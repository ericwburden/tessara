[CmdletBinding()]
param(
    [switch]$SelfTest,
    [Parameter(DontShow)][switch]$ParallelWorker,
    [Parameter(DontShow)][string]$WorkerModulePath,
    [Parameter(DontShow)][string]$WorkerAdapterPath,
    [Parameter(DontShow)][string]$WorkerEvidenceRoot,
    [Parameter(DontShow)][string]$WorkerCandidateFingerprint,
    [Parameter(DontShow)][string]$WorkerResultPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($ParallelWorker) {
    foreach ($workerArgument in @(
            $WorkerModulePath,
            $WorkerAdapterPath,
            $WorkerEvidenceRoot,
            $WorkerCandidateFingerprint,
            $WorkerResultPath
        )) {
        if ([string]::IsNullOrWhiteSpace($workerArgument)) {
            throw "Parallel certification worker arguments must be non-empty."
        }
    }

    Import-Module $WorkerModulePath -Force
    $workerResult = Invoke-TessaraValidationLane `
        -AdapterPath $WorkerAdapterPath `
        -LaneId "local-success" `
        -CandidateFingerprint $WorkerCandidateFingerprint `
        -EvidenceRoot $WorkerEvidenceRoot
    $workerResultBytes = [Text.UTF8Encoding]::new($false).GetBytes(
        ($workerResult | ConvertTo-Json -Depth 100 -Compress)
    )
    $workerResultStream = [IO.FileStream]::new(
        [IO.Path]::GetFullPath($WorkerResultPath),
        [IO.FileMode]::CreateNew,
        [IO.FileAccess]::Write,
        [IO.FileShare]::None
    )
    try {
        $workerResultStream.Write($workerResultBytes, 0, $workerResultBytes.Length)
        $workerResultStream.Flush($true)
    } finally {
        $workerResultStream.Dispose()
    }
    return
}

if (-not $SelfTest) {
    throw "This is a validation-platform certification suite. Invoke it with -SelfTest."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$modulePath = Join-Path $PSScriptRoot "tessara-validation-platform.psm1"
$lifecycleModulePath = Join-Path $PSScriptRoot `
    "validation-platform/tessara-validation-lifecycle.psm1"
$finalizerModulePath = Join-Path $PSScriptRoot `
    "validation-platform/tessara-validation-finalizer.psm1"
$adapterPath = Join-Path $PSScriptRoot "validation-platform/fixtures/synthetic-adapter.json"
$dockerShimPath = Join-Path $PSScriptRoot "validation-platform/fixtures/docker-shim.ps1"
$certificationHarnessRelativePaths = @(
    "scripts/test-tessara-validation-platform.ps1"
    "scripts/validation-platform/fixtures/assert-synthetic-environment.ps1"
    "scripts/validation-platform/fixtures/docker-shim.ps1"
    "scripts/validation-platform/fixtures/synthetic-acceptance.json"
    "scripts/validation-platform/fixtures/synthetic-action.ps1"
    "scripts/validation-platform/fixtures/synthetic-adapter.json"
    "scripts/validation-platform/fixtures/synthetic-compose.yaml"
    "scripts/validation-platform/fixtures/synthetic-service.ps1"
    "scripts/validation-platform/fixtures/synthetic-validation-contract.json"
)

function Get-CertificationHarnessSnapshot {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string[]]$RelativePaths
    )
    if ($RelativePaths.Count -ne 9 -or
        [string]$RelativePaths[0] -cne "scripts/test-tessara-validation-platform.ps1" -or
        @($RelativePaths | Where-Object {
                ([string]$_).StartsWith(
                    "scripts/validation-platform/fixtures/",
                    [StringComparison]::Ordinal
                )
            }).Count -ne 8 -or
        @($RelativePaths | Sort-Object -Unique).Count -ne $RelativePaths.Count) {
        throw "Certification harness inventory must be the certification script plus exactly eight fixtures."
    }

    $resolvedRoot = [IO.Path]::GetFullPath($RepositoryRoot).TrimEnd(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar
    )
    $rootEntry = Get-Item -Force -LiteralPath $resolvedRoot
    if (-not $rootEntry.PSIsContainer -or
        ($rootEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Certification repository root is not one ordinary directory."
    }
    $comparison = if ([OperatingSystem]::IsWindows()) {
        [StringComparison]::OrdinalIgnoreCase
    } else {
        [StringComparison]::Ordinal
    }
    $rootPrefix = $resolvedRoot + [IO.Path]::DirectorySeparatorChar
    $maximumFileBytes = 4MB
    $maximumAggregateBytes = 16MB
    [long]$aggregateBytes = 0
    $files = [Collections.Generic.List[object]]::new()

    foreach ($relativePath in $RelativePaths) {
        $canonicalPath = ([string]$relativePath).Replace('\', '/')
        if ($canonicalPath -cne [string]$relativePath -or
            [IO.Path]::IsPathRooted($canonicalPath) -or
            @($canonicalPath.Split('/') | Where-Object { $_ -ceq ".." }).Count -ne 0) {
            throw "Certification harness path is not canonical: $relativePath"
        }
        $resolvedPath = [IO.Path]::GetFullPath((Join-Path $resolvedRoot $canonicalPath))
        if (-not $resolvedPath.StartsWith($rootPrefix, $comparison)) {
            throw "Certification harness path escapes the repository: $canonicalPath"
        }

        $currentPath = $resolvedRoot
        $entry = $null
        foreach ($segment in $canonicalPath.Split('/')) {
            $currentPath = Join-Path $currentPath $segment
            $entry = Get-Item -Force -LiteralPath $currentPath
            if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
                ($entry.PSObject.Properties.Name -contains "LinkType" -and
                    -not [string]::IsNullOrEmpty([string]$entry.LinkType))) {
                throw "Certification harness path crosses a reparse point: $canonicalPath"
            }
        }
        if ($null -eq $entry -or $entry.PSIsContainer -or
            $entry -isnot [IO.FileInfo]) {
            throw "Certification harness entry is not one ordinary file: $canonicalPath"
        }
        if ($entry.PSObject.Properties.Name -contains "UnixMode" -and
            -not ([string]$entry.UnixMode).StartsWith("-", [StringComparison]::Ordinal)) {
            throw "Certification harness entry is not one ordinary file: $canonicalPath"
        }
        if ([long]$entry.Length -gt $maximumFileBytes) {
            throw "Certification harness file exceeds the 4-MiB limit: $canonicalPath"
        }
        $aggregateBytes += [long]$entry.Length
        if ($aggregateBytes -gt $maximumAggregateBytes) {
            throw "Certification harness exceeds the 16-MiB aggregate limit."
        }
        $files.Add([pscustomobject][ordered]@{
            path = $canonicalPath
            sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $resolvedPath).Hash.ToLowerInvariant()
        })
    }

    $identityText = "tessara.validation.certification-harness.v1`n" +
        (@($files | ForEach-Object {
                    "$(([string]$_.path).Length):$($_.path):$($_.sha256)`n"
                }) -join "")
    $identityBytes = [Text.UTF8Encoding]::new($false).GetBytes($identityText)
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.certification-harness"
        fingerprint = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($identityBytes)
        ).ToLowerInvariant()
        files = @($files)
    }
}

$certificationHarnessSnapshot = Get-CertificationHarnessSnapshot `
    -RepositoryRoot $repoRoot -RelativePaths $certificationHarnessRelativePaths
Import-Module $modulePath -Force

$platformModule = Get-Module | Where-Object {
    [IO.Path]::GetFullPath([string]$_.Path) -ceq [IO.Path]::GetFullPath($modulePath)
} | Select-Object -First 1
if ($null -eq $platformModule) {
    throw "Validation-platform module did not load for private parser certification."
}
$nulPathText = "line-one`nline-two" + [char]0 + 'literal\backslash' + [char]0
$nulPathRecords = @(& $platformModule {
    param([string]$Text)
    ConvertFrom-TessaraPlatformGitNulRecords -Text $Text -Label "NUL parser probe"
} $nulPathText)
if ($nulPathRecords.Count -ne 2 -or
    [string]$nulPathRecords[0] -cne "line-one`nline-two" -or
    [string]$nulPathRecords[1] -cne 'literal\backslash') {
    throw "Git NUL-path parsing did not preserve newline or literal-backslash path text."
}
$h1 = "1" * 64
$h2 = "2" * 64
$h3 = "3" * 64
$collisionA = [pscustomobject][ordered]@{ entries = @(
    [pscustomobject][ordered]@{ path = "p`n$h1"; sha256 = $h2 }
    [pscustomobject][ordered]@{ path = "q"; sha256 = $h3 }
) }
$collisionB = [pscustomobject][ordered]@{ entries = @(
    [pscustomobject][ordered]@{ path = "p"; sha256 = $h1 }
    [pscustomobject][ordered]@{ path = "$h2`nq"; sha256 = $h3 }
) }
$legacyA = @($collisionA.entries | ForEach-Object {
    "$([string]$_.path)`n$([string]$_.sha256)"
}) -join "`n"
$legacyB = @($collisionB.entries | ForEach-Object {
    "$([string]$_.path)`n$([string]$_.sha256)"
}) -join "`n"
if ($legacyA -cne $legacyB) {
    throw "Git pathname collision regression fixture is not structurally equivalent."
}
$canonicalCollisionA = & $platformModule {
    param($Value)
    Get-TessaraPlatformCanonicalJsonSha256 -Value $Value
} $collisionA
$canonicalCollisionB = & $platformModule {
    param($Value)
    Get-TessaraPlatformCanonicalJsonSha256 -Value $Value
} $collisionB
if ([string]$canonicalCollisionA -ceq [string]$canonicalCollisionB) {
    throw "Canonical dependency inventory hashing retained a newline repartition collision."
}

function Assert-Equal {
    param(
        [AllowNull()][object]$Actual,
        [AllowNull()][object]$Expected,
        [Parameter(Mandatory)][string]$Label
    )
    if ($Actual -cne $Expected) {
        throw "$Label expected '$Expected' but received '$Actual'."
    }
}

function Assert-Rejected {
    param(
        [Parameter(Mandatory)][scriptblock]$Action,
        [Parameter(Mandatory)][string]$ExpectedMessage
    )
    try {
        & $Action
    } catch {
        if ($_.Exception.Message -notlike "*$ExpectedMessage*") {
            throw "Expected rejection containing '$ExpectedMessage', received: $($_.Exception.Message)"
        }
        return
    }
    throw "Expected rejection containing '$ExpectedMessage' at certification call line $($MyInvocation.ScriptLineNumber)."
}

function Copy-JsonValue {
    param([Parameter(Mandatory)]$Value)
    $Value | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
}

function Write-JsonFile {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)]$Document)
    [IO.File]::WriteAllText(
        $Path,
        (($Document | ConvertTo-Json -Depth 100 -Compress) + "`n"),
        [Text.UTF8Encoding]::new($false)
    )
}

function Write-TestAdapterWithMatchingContract {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    $contractSource = Join-Path $RepositoryRoot ([string]$Document.validation_contract_path)
    $contract = Get-Content -Raw -LiteralPath $contractSource | ConvertFrom-Json -Depth 100
    foreach ($lane in @($Document.lanes)) {
        foreach ($action in @($lane.actions | Where-Object {
                    [string]$_.stage -ceq "assertion"
                })) {
            $target = @($contract.implementation_targets | Where-Object {
                    [string]$_.id -ceq [string]$action.implementation_target
                })[0]
            if ($null -eq $target) {
                throw "Test adapter assertion target '$($action.implementation_target)' is absent from its source contract."
            }
            $actionCommand = [pscustomobject][ordered]@{
                program = [string]$action.program
                arguments = @($action.arguments)
                input_paths = @($action.input_paths)
                tools = @($action.tools)
            }
            $targetCommandJson = $target.command | ConvertTo-Json -Depth 50 -Compress
            $actionCommandJson = $actionCommand | ConvertTo-Json -Depth 50 -Compress
            if ($targetCommandJson -ceq $actionCommandJson) { continue }

            $variant = Copy-JsonValue $target
            $variantDigest = [Convert]::ToHexString(
                [Security.Cryptography.SHA256]::HashData(
                    [Text.UTF8Encoding]::new($false).GetBytes($actionCommandJson)
                )
            ).ToLowerInvariant().Substring(0, 12)
            $variant.id = "test-$([string]$lane.id)-$variantDigest"
            $variant.command = $actionCommand
            $contract.implementation_targets += $variant
            foreach ($requirement in @($contract.requirements | Where-Object {
                        @($_.implementation_targets) -ccontains [string]$target.id -and
                        @($_.validation_lanes) -ccontains [string]$lane.id
                    })) {
                if (@($requirement.validation_lanes).Count -eq 1) {
                    $requirement.implementation_targets = @(
                        $requirement.implementation_targets | ForEach-Object {
                            if ([string]$_ -ceq [string]$target.id) {
                                [string]$variant.id
                            } else { [string]$_ }
                        }
                    )
                } else {
                    $requirement.validation_lanes = @($requirement.validation_lanes |
                        Where-Object { [string]$_ -cne [string]$lane.id })
                    $requirementVariant = Copy-JsonValue $requirement
                    $requirementVariant.id = "test-requirement-$([string]$lane.id)-$variantDigest"
                    $requirementVariant.validation_lanes = @([string]$lane.id)
                    $requirementVariant.implementation_targets = @(
                        $requirement.implementation_targets | ForEach-Object {
                            if ([string]$_ -ceq [string]$target.id) {
                                [string]$variant.id
                            } else { [string]$_ }
                        }
                    )
                    $contract.requirements += $requirementVariant
                }
            }
            $action.implementation_target = [string]$variant.id
        }
    }
    $contractPath = [IO.Path]::ChangeExtension($Path, ".contract.json")
    Write-JsonFile -Path $contractPath -Document $contract
    $Document.validation_contract_path = [IO.Path]::GetRelativePath(
        $RepositoryRoot, $contractPath
    ).Replace('\', '/')
    Write-JsonFile -Path $Path -Document $Document
}

function Get-LaneResult {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$LaneId
    )
    $result = @(Get-ChildItem -LiteralPath (Join-Path $EvidenceRoot "lanes/$LaneId/attempts") `
        -Filter "lane-result.json" -File -Recurse | Sort-Object FullName | Select-Object -Last 1)
    if ($result.Count -ne 1) { throw "Lane '$LaneId' did not retain one discoverable result." }
    $document = Get-Content -Raw -LiteralPath $result[0].FullName |
        ConvertFrom-Json -Depth 100
    $document | Add-Member -NotePropertyName evidence_path `
        -NotePropertyValue $result[0].FullName -Force
    $document
}

function Get-DockerShimLiveResidue {
    param([Parameter(Mandatory)][string]$Root)

    @(
        Get-ChildItem -LiteralPath $Root -Force -ErrorAction SilentlyContinue |
            Where-Object { [string]$_.Name -cne "docker-transcript.jsonl" }
    )
}

function Invoke-CompatibilityPlannerSet {
    param(
        [Parameter(Mandatory)][string]$AdapterPath,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$CandidateFingerprint
    )
    $CandidateFingerprint = [string](Get-TessaraValidationCandidateIdentity `
        -AdapterPath $AdapterPath -RepositoryRoot $RepositoryRoot).candidate_fingerprint
    $owner = Invoke-TessaraValidationLane -AdapterPath $AdapterPath `
        -LaneId "no-topology" -CandidateFingerprint $CandidateFingerprint `
        -EvidenceRoot $EvidenceRoot -RepositoryRoot $RepositoryRoot
    $independent = Invoke-TessaraValidationLane -AdapterPath $AdapterPath `
        -LaneId "local-success" -CandidateFingerprint $CandidateFingerprint `
        -EvidenceRoot $EvidenceRoot -RepositoryRoot $RepositoryRoot
    $dependent = Invoke-TessaraValidationLane -AdapterPath $AdapterPath `
        -LaneId "dependent-no-topology" -CandidateFingerprint $CandidateFingerprint `
        -EvidenceRoot $EvidenceRoot -RepositoryRoot $RepositoryRoot `
        -PrerequisiteResultPaths @($owner.evidence_path)
    [pscustomobject][ordered]@{
        owner = [string]$owner.lane_compatibility_fingerprint
        dependent = [string]$dependent.lane_compatibility_fingerprint
        independent = [string]$independent.lane_compatibility_fingerprint
    }
}

function Assert-ScopedPlannerMutation {
    param(
        [Parameter(Mandatory)]$Baseline,
        [Parameter(Mandatory)]$Mutated,
        [Parameter(Mandatory)][string]$Label
    )
    if ([string]$Mutated.owner -ceq [string]$Baseline.owner -or
        [string]$Mutated.dependent -ceq [string]$Baseline.dependent) {
        throw "$Label did not invalidate the owning lane and its dependent closure."
    }
    Assert-Equal $Mutated.independent $Baseline.independent `
        "$Label independent lane compatibility"
}

function Assert-EvidencePairs {
    param([Parameter(Mandatory)][string]$AttemptRoot)
    foreach ($file in @(Get-ChildItem -LiteralPath $AttemptRoot -File -Recurse |
            Where-Object { $_.Name -notlike "*.sha256" })) {
        $sidecar = "$($file.FullName).sha256"
        if (-not (Test-Path -LiteralPath $sidecar -PathType Leaf)) {
            throw "Evidence file has no SHA-256 sidecar: $($file.FullName)"
        }
        $expected = (Get-Content -Raw -LiteralPath $sidecar).Split(' ', 2)[0]
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $file.FullName).Hash.ToLowerInvariant()
        Assert-Equal $actual $expected "Evidence digest"
    }
}

function Assert-NoFinalizationPublication {
    param([Parameter(Mandatory)][string]$AttemptRoot)
    foreach ($name in @(
            "lane-result.json", "lane-result.json.sha256",
            "attempt-index.json", "attempt-index.json.sha256"
        )) {
        if (Test-Path -LiteralPath (Join-Path $AttemptRoot $name)) {
            throw "Rejected finalization published '$name'."
        }
    }
    foreach ($attestation in @(Get-ChildItem -LiteralPath $AttemptRoot -File -Force |
            Where-Object {
                $_.Name -cmatch '^finalization-attestation\.[0-9a-f]{64}\.json(?:\.sha256)?$'
            })) {
        throw "Rejected finalization published '$($attestation.Name)'."
    }
}

function Test-PortClosed {
    param([Parameter(Mandatory)][int]$Port)
    $client = [Net.Sockets.TcpClient]::new()
    try {
        $task = $client.ConnectAsync([Net.IPAddress]::Loopback, $Port)
        if ($task.Wait(500) -and $client.Connected) { return $false }
        $true
    } catch { $true } finally { $client.Dispose() }
}

function Assert-CertificationRootChain {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    $resolved = [IO.Path]::GetFullPath($Path)
    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $allowed = [IO.Path]::GetFullPath((Join-Path $repository `
        "artifacts/validation-platform-certification"))
    $relative = [IO.Path]::GetRelativePath($allowed, $resolved)
    if ($relative -eq "." -or $relative -eq ".." -or [IO.Path]::IsPathRooted($relative) -or
        $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal)) {
        throw "Refusing to remove certification root outside its repository artifact boundary: $resolved"
    }
    $current = $repository
    $repositoryEntry = Get-Item -Force -LiteralPath $current
    if (-not $repositoryEntry.PSIsContainer -or
        ($repositoryEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Certification repository root is not an ordinary directory: $current"
    }
    $relativeFromRepository = [IO.Path]::GetRelativePath($repository, $resolved)
    foreach ($segment in @($relativeFromRepository -split '[\\/]' | Where-Object {
            -not [string]::IsNullOrWhiteSpace($_)
        })) {
        $current = Join-Path $current $segment
        $entry = Get-Item -Force -LiteralPath $current
        if (-not $entry.PSIsContainer -or
            ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Certification root crosses a non-ordinary directory: $current"
        }
    }
    $resolved
}

function New-CertificationRoot {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Capability
    )
    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $repositoryEntry = Get-Item -Force -LiteralPath $repository
    if (-not $repositoryEntry.PSIsContainer -or
        ($repositoryEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Certification repository root is not an ordinary directory: $repository"
    }
    $current = $repository
    foreach ($segment in @("artifacts", "validation-platform-certification")) {
        $current = Join-Path $current $segment
        if (-not [IO.Directory]::Exists($current)) {
            [IO.Directory]::CreateDirectory($current) | Out-Null
        }
        $entry = Get-Item -Force -LiteralPath $current
        if (-not $entry.PSIsContainer -or
            ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Certification root crosses a non-ordinary directory: $current"
        }
    }
    $root = Join-Path $current ([guid]::NewGuid().ToString('N'))
    [IO.Directory]::CreateDirectory($root) | Out-Null
    $root = Assert-CertificationRootChain -Path $root -RepositoryRoot $repository
    $markerPath = Join-Path $root ".tessara-certification-root.json"
    $markerStream = [IO.File]::Open(
        $markerPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read
    )
    try {
        $markerBytes = [Text.UTF8Encoding]::new($false).GetBytes((
            ([pscustomobject][ordered]@{
                schema_version = 1
                contract = "tessara.validation.certification-root"
                root = $root
                capability = $Capability
            } | ConvertTo-Json -Depth 10 -Compress) + "`n"
        ))
        $markerStream.Write($markerBytes, 0, $markerBytes.Length)
        $markerStream.Flush($true)
    } finally {
        $markerStream.Dispose()
    }
    $root
}

function Remove-CertificationRoot {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Capability
    )
    if (-not (Test-Path -LiteralPath $Path)) { return }
    $resolved = Assert-CertificationRootChain -Path $Path -RepositoryRoot $RepositoryRoot
    $markerPath = Join-Path $resolved ".tessara-certification-root.json"
    if (-not (Test-Path -LiteralPath $markerPath -PathType Leaf)) {
        throw "Refusing to remove certification root without its private ownership marker."
    }
    $marker = Get-Content -Raw -LiteralPath $markerPath | ConvertFrom-Json -Depth 10
    if ([int]$marker.schema_version -ne 1 -or
        [string]$marker.contract -cne "tessara.validation.certification-root" -or
        [string]$marker.root -cne $resolved -or
        [string]$marker.capability -cne $Capability) {
        throw "Refusing to remove certification root with an invalid ownership capability."
    }
    $resolved = Assert-CertificationRootChain -Path $resolved -RepositoryRoot $RepositoryRoot
    Remove-Item -LiteralPath $resolved -Recurse -Force
}

$rootCapability = [Convert]::ToHexString(
    [Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
).ToLowerInvariant()
$root = New-CertificationRoot -RepositoryRoot $repoRoot -Capability $rootCapability
$previousSentinel = [Environment]::GetEnvironmentVariable("TESSARA_VP_SENTINEL", "Process")
$sentinelWasPresent = [Environment]::GetEnvironmentVariables("Process").Contains("TESSARA_VP_SENTINEL")
$shimEnvironmentNames = @(
    "TESSARA_VP_DOCKER_SHIM_ROOT", "TESSARA_VP_DOCKER_SHIM_FOREIGN_CONFIG",
    "TESSARA_VP_DOCKER_SHIM_FOREIGN_LEASE", "TESSARA_VP_DOCKER_SHIM_RETAIN"
)
$shimEnvironmentBefore = [ordered]@{}
foreach ($name in $shimEnvironmentNames) {
    $shimEnvironmentBefore[$name] = [pscustomobject][ordered]@{
        present = [Environment]::GetEnvironmentVariables("Process").Contains($name)
        value = [Environment]::GetEnvironmentVariable($name, "Process")
    }
}
$candidate = [string](Get-TessaraValidationCandidateIdentity `
    -AdapterPath $adapterPath -RepositoryRoot $repoRoot).candidate_fingerprint
$dockerCommand = @("pwsh", "-NoProfile", "-NonInteractive", "-File", $dockerShimPath)
$dockerCommandInputPaths = @(
    "scripts/validation-platform/fixtures/docker-shim.ps1"
)
$laneASourceBefore = [Environment]::GetEnvironmentVariable("TESSARA_VP_LANE_A_SOURCE", "Process")
$laneASourceWasPresent = [Environment]::GetEnvironmentVariables("Process").Contains(
    "TESSARA_VP_LANE_A_SOURCE"
)
$certificationEnvironmentNames = @(
    "TESSARA_VP_REQUIRED_SOURCE",
    "TESSARA_VP_WHITESPACE_SOURCE",
    "TESSARA_VP_SECRET_SOURCE",
    "TESSARA_VP_UNDECLARED"
)
$certificationEnvironmentBefore = [ordered]@{}
foreach ($name in $certificationEnvironmentNames) {
    $certificationEnvironmentBefore[$name] = [pscustomobject][ordered]@{
        present = [Environment]::GetEnvironmentVariables("Process").Contains($name)
        value = [Environment]::GetEnvironmentVariable($name, "Process")
    }
}
Remove-Item Env:TESSARA_VP_LANE_A_SOURCE -ErrorAction SilentlyContinue
Remove-Item Env:TESSARA_VP_REQUIRED_SOURCE -ErrorAction SilentlyContinue
Remove-Item Env:TESSARA_VP_WHITESPACE_SOURCE -ErrorAction SilentlyContinue
Remove-Item Env:TESSARA_VP_SECRET_SOURCE -ErrorAction SilentlyContinue

try {
    $cleanupProbeRoot = Join-Path $root "cleanup-probe"
    [IO.Directory]::CreateDirectory($cleanupProbeRoot) | Out-Null
    $cleanupProbeCapability = [Convert]::ToHexString(
        [Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
    ).ToLowerInvariant()
    [IO.File]::WriteAllText(
        (Join-Path $cleanupProbeRoot ".tessara-certification-root.json"),
        (([pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.certification-root"
            root = [IO.Path]::GetFullPath($cleanupProbeRoot)
            capability = $cleanupProbeCapability
        } | ConvertTo-Json -Depth 10 -Compress) + "`n"),
        [Text.UTF8Encoding]::new($false)
    )
    $cleanupOutsideSentinel = Join-Path $cleanupProbeRoot "must-survive-rejection.txt"
    [IO.File]::WriteAllText(
        $cleanupOutsideSentinel,
        "sentinel`n",
        [Text.UTF8Encoding]::new($false)
    )
    Assert-Rejected -Action {
        Remove-CertificationRoot -Path $cleanupProbeRoot `
            -RepositoryRoot $repoRoot -Capability ("0" * 64)
    } -ExpectedMessage "invalid ownership capability"
    if (-not (Test-Path -LiteralPath $cleanupOutsideSentinel -PathType Leaf)) {
        throw "Certification cleanup removed a sentinel without its ownership capability."
    }
    Remove-CertificationRoot -Path $cleanupProbeRoot `
        -RepositoryRoot $repoRoot -Capability $cleanupProbeCapability

    $cleanupSafetyRepository = Join-Path $root "cleanup-safety-repository"
    $cleanupSafetyArtifacts = Join-Path $cleanupSafetyRepository "artifacts"
    $cleanupSafetyTarget = Join-Path $root "cleanup-safety-outside-target"
    $cleanupSafetyLink = Join-Path $cleanupSafetyArtifacts `
        "validation-platform-certification"
    [IO.Directory]::CreateDirectory($cleanupSafetyArtifacts) | Out-Null
    [IO.Directory]::CreateDirectory($cleanupSafetyTarget) | Out-Null
    if ($IsWindows) {
        New-Item -ItemType Junction -Path $cleanupSafetyLink `
            -Target $cleanupSafetyTarget | Out-Null
    } else {
        [IO.Directory]::CreateSymbolicLink($cleanupSafetyLink, $cleanupSafetyTarget) | Out-Null
    }
    try {
        Assert-Rejected -Action {
            New-CertificationRoot -RepositoryRoot $cleanupSafetyRepository `
                -Capability ("f" * 64) | Out-Null
        } -ExpectedMessage "non-ordinary directory"
        Assert-Equal @(Get-ChildItem -LiteralPath $cleanupSafetyTarget -Force).Count 0 `
            "Certification parent reparse write containment"
    } finally {
        [IO.Directory]::Delete($cleanupSafetyLink)
    }

    $identity = Get-TessaraValidationPlatformIdentity
    $identityAgain = Get-TessaraValidationPlatformIdentity
    Assert-Equal $identity.contract "tessara.validation.platform" "Platform contract"
    Assert-Equal $identity.release_version "2.0.0" "Platform release"
    Assert-Equal $identity.components.Count 4 "Platform component count"
    Assert-Equal $identity.boundary_inputs.Count 21 "Platform boundary-input count"
    if ([string]$identity.execution_fingerprint -cnotmatch '^[0-9a-f]{64}$') {
        throw "Platform did not publish a lane execution fingerprint."
    }
    if ([string]$identity.finalization_fingerprint -cnotmatch '^[0-9a-f]{64}$') {
        throw "Platform did not publish an independent finalization fingerprint."
    }
    Assert-Equal $identity.platform_fingerprint $identityAgain.platform_fingerprint `
        "Deterministic platform fingerprint"
    foreach ($component in @($identity.components)) {
        $full = Join-Path $repoRoot ([string]$component.path)
        Assert-Equal $component.sha256 `
            (Get-FileHash -Algorithm SHA256 -LiteralPath $full).Hash.ToLowerInvariant() `
            "Platform component hash"
    }
    foreach ($boundaryInput in @($identity.boundary_inputs)) {
        $full = Join-Path $repoRoot ([string]$boundaryInput.path)
        Assert-Equal $boundaryInput.sha256 `
            (Get-FileHash -Algorithm SHA256 -LiteralPath $full).Hash.ToLowerInvariant() `
            "Platform boundary-input hash"
    }
    Import-Module $modulePath -Force
    Get-TessaraValidationPlatformIdentity | Out-Null

    $validated = Assert-TessaraValidationAdapter -AdapterPath $adapterPath
    if ([string]$validated.adapter_fingerprint -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$validated.acceptance_fingerprint -cnotmatch '^[0-9a-f]{64}$' -or
        [string]$validated.validation_contract_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        @($validated.lane_identities).Count -ne 5 -or
        @($validated.lane_identities | Where-Object {
            [string]$_.adapter_fingerprint -cnotmatch '^[0-9a-f]{64}$' -or
            [string]$_.acceptance_fingerprint -cnotmatch '^[0-9a-f]{64}$'
        }).Count -ne 0) {
        throw "Validated adapter did not publish exact fingerprints."
    }
    $adapter = Get-Content -Raw -LiteralPath $adapterPath | ConvertFrom-Json -Depth 100

    $unknownField = Copy-JsonValue $adapter
    $unknownField | Add-Member -NotePropertyName callback -NotePropertyValue "Invoke-Anything"
    $unknownFieldPath = Join-Path $root "unknown-field.json"
    Write-JsonFile -Path $unknownFieldPath -Document $unknownField
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $unknownFieldPath | Out-Null
    } -ExpectedMessage "does not satisfy contract v2"

    $unknownToken = Copy-JsonValue $adapter
    $unknownToken.lanes[0].actions[0].arguments[0] = '${unknown}'
    $unknownTokenPath = Join-Path $root "unknown-token.json"
    Write-JsonFile -Path $unknownTokenPath -Document $unknownToken
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $unknownTokenPath | Out-Null
    } -ExpectedMessage "unknown token"

    $duplicateAction = Copy-JsonValue $adapter
    $duplicateAction.lanes[0].actions += Copy-JsonValue $duplicateAction.lanes[0].actions[0]
    $duplicateActionPath = Join-Path $root "duplicate-action.json"
    Write-JsonFile -Path $duplicateActionPath -Document $duplicateAction
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $duplicateActionPath | Out-Null
    } -ExpectedMessage "duplicate action IDs"

    $prerequisiteMismatch = Copy-JsonValue $adapter
    $prerequisiteMismatch.lanes[0].prerequisites = @("compose-consume")
    $prerequisiteMismatchPath = Join-Path $root "prerequisite-mismatch.json"
    Write-JsonFile -Path $prerequisiteMismatchPath -Document $prerequisiteMismatch
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $prerequisiteMismatchPath | Out-Null
    } -ExpectedMessage "prerequisites do not exactly match"

    $handoffToLocalSuccess = Copy-JsonValue $adapter
    $handoffToLocalSuccessProducer = @($handoffToLocalSuccess.lanes | Where-Object {
        [string]$_.id -ceq "compose-create"
    })[0]
    $handoffToLocalSuccessProducer.topology.handoff_to = "local-success"
    $handoffToLocalSuccessPath = Join-Path $root "handoff-to-local-success.json"
    Write-JsonFile -Path $handoffToLocalSuccessPath -Document $handoffToLocalSuccess
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $handoffToLocalSuccessPath | Out-Null
    } -ExpectedMessage "does not target one matching consume successor"

    $handoffToSelf = Copy-JsonValue $adapter
    $handoffToSelfProducer = @($handoffToSelf.lanes | Where-Object {
        [string]$_.id -ceq "compose-create"
    })[0]
    $handoffToSelfProducer.topology.handoff_to = "compose-create"
    $handoffToSelfPath = Join-Path $root "handoff-to-self.json"
    Write-JsonFile -Path $handoffToSelfPath -Document $handoffToSelf
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $handoffToSelfPath | Out-Null
    } -ExpectedMessage "does not target one matching consume successor"

    $mismatchedHandoffSource = Copy-JsonValue $adapter
    $mismatchedHandoffConsumer = @($mismatchedHandoffSource.lanes | Where-Object {
        [string]$_.id -ceq "compose-consume"
    })[0]
    $mismatchedHandoffConsumer.topology.from_lane = "no-topology"
    $mismatchedHandoffSourcePath = Join-Path $root "mismatched-handoff-source.json"
    Write-JsonFile -Path $mismatchedHandoffSourcePath -Document $mismatchedHandoffSource
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $mismatchedHandoffSourcePath | Out-Null
    } -ExpectedMessage "does not target one matching consume successor"

    $destroyedHandoffSource = Copy-JsonValue $adapter
    $destroyedHandoffProducer = @($destroyedHandoffSource.lanes | Where-Object {
        [string]$_.id -ceq "compose-create"
    })[0]
    $destroyedHandoffProducer.topology.on_success = "destroy"
    $destroyedHandoffProducer.actions[0].stage = "assertion"
    $destroyedHandoffProducer.actions[0] | Add-Member `
        -NotePropertyName implementation_target `
        -NotePropertyValue "synthetic-compose-certification"
    $destroyedHandoffProducer.actions[0] | Add-Member `
        -NotePropertyName proof_classes -NotePropertyValue @("runner-selftest")
    $destroyedHandoffSourcePath = Join-Path $root "destroyed-handoff-source.json"
    Write-JsonFile -Path $destroyedHandoffSourcePath -Document $destroyedHandoffSource
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $destroyedHandoffSourcePath | Out-Null
    } -ExpectedMessage "does not consume one declared prerequisite handoff"

    $cycleRepository = Join-Path $root "cycle-repository"
    [IO.Directory]::CreateDirectory($cycleRepository) | Out-Null
    $cycle = Copy-JsonValue $adapter
    $cycleContract = Get-Content -Raw -LiteralPath `
        (Join-Path $repoRoot "scripts/validation-platform/fixtures/synthetic-validation-contract.json") |
        ConvertFrom-Json -Depth 100
    $cycle.lanes[0].prerequisites = @("compose-consume")
    $cycle.lanes[3].prerequisites = @("no-topology", "compose-create")
    $cycleContract.lanes[0].prerequisites = @("compose-consume")
    $cycleContract.lanes[3].prerequisites = @("no-topology", "compose-create")
    $cycle.validation_contract_path = "contract.json"
    $cycle.acceptance_inputs = @([pscustomobject][ordered]@{
        path = "acceptance.json"
        lanes = @($cycle.lanes.id)
    })
    $cycle.execution_inputs = @([pscustomobject][ordered]@{
        path = "action.ps1"
        lanes = @($cycle.lanes.id)
    })
    $cyclePath = Join-Path $cycleRepository "cycle.json"
    Write-JsonFile -Path $cyclePath -Document $cycle
    Write-JsonFile -Path (Join-Path $cycleRepository "contract.json") -Document $cycleContract
    Write-JsonFile -Path (Join-Path $cycleRepository "acceptance.json") -Document `
        ([pscustomobject]@{ expected = "synthetic" })
    [IO.File]::WriteAllText((Join-Path $cycleRepository "action.ps1"), "exit 0`n")
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $cyclePath `
            -RepositoryRoot $cycleRepository | Out-Null
    } -ExpectedMessage "prerequisites contain a cycle"

    $interpreterBypasses = @(
        @("-NoProfile", "-Command", "exit 0"),
        @("-NoProfile", "-command", "exit 0"),
        @("-NoProfile", "/Command", "exit 0"),
        @("-NoProfile", "-c", "exit 0"),
        @("-NoProfile", "-EncodedCommand", "ZQB4AGkAdAAgADAA")
    )
    for ($bypassIndex = 0; $bypassIndex -lt $interpreterBypasses.Count; $bypassIndex++) {
        $shellString = Copy-JsonValue $adapter
        $shellString.lanes[0].actions[0].arguments = @($interpreterBypasses[$bypassIndex])
        $shellStringPath = Join-Path $root "shell-string-$bypassIndex.json"
        Write-JsonFile -Path $shellStringPath -Document $shellString
        Assert-Rejected -Action {
            Assert-TessaraValidationAdapter -AdapterPath $shellStringPath | Out-Null
        } -ExpectedMessage "must use exactly one exact '-File' switch"
    }
    foreach ($combinedShellProgram in @("bash", "bash.exe")) {
        $combinedShell = Copy-JsonValue $adapter
        $combinedShell.lanes[0].actions[0].program = $combinedShellProgram
        $combinedShell.lanes[0].actions[0].arguments = @("-lc", "exit 0")
        $combinedShellPath = Join-Path $root `
            "combined-shell-string-$($combinedShellProgram.Replace('.', '-')).json"
        Write-JsonFile -Path $combinedShellPath -Document $combinedShell
        Assert-Rejected -Action {
            Assert-TessaraValidationAdapter -AdapterPath $combinedShellPath | Out-Null
        } -ExpectedMessage "must name one declared input directly as its first argument"
    }

    $undeclaredCommandInput = Copy-JsonValue $adapter
    $undeclaredInputLane = @($undeclaredCommandInput.lanes | Where-Object {
        [string]$_.id -ceq "compose-create"
    })[0]
    $undeclaredInputLane.actions[0].input_paths += `
        "scripts/validation-platform/fixtures/assert-synthetic-environment.ps1"
    $undeclaredCommandInputPath = Join-Path $root "undeclared-command-input.json"
    Write-JsonFile -Path $undeclaredCommandInputPath -Document $undeclaredCommandInput
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $undeclaredCommandInputPath | Out-Null
    } -ExpectedMessage "outside its execution-input inventory"

    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath -LaneId "no-topology" `
            -CandidateFingerprint $candidate -EvidenceRoot (Join-Path $root "docker-shell-string") `
            -DockerCommand @("pwsh", "-NoProfile", "-Command", "exit 0") `
            -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
    } -ExpectedMessage "must use exactly one exact '-File' switch"

    $junctionEvidenceRoot = Join-Path $root "junction-evidence-root"
    $junctionOutsideTarget = Join-Path $root "junction-outside-target"
    $junctionLanesPath = Join-Path $junctionEvidenceRoot "lanes"
    [IO.Directory]::CreateDirectory($junctionEvidenceRoot) | Out-Null
    [IO.Directory]::CreateDirectory($junctionOutsideTarget) | Out-Null
    $junctionOutsideSentinel = Join-Path $junctionOutsideTarget "must-survive.txt"
    [IO.File]::WriteAllText(
        $junctionOutsideSentinel,
        "no out-of-root evidence may be created here`n",
        [Text.UTF8Encoding]::new($false)
    )
    $junctionOutsideSentinelHash = (Get-FileHash -Algorithm SHA256 `
        -LiteralPath $junctionOutsideSentinel).Hash
    try {
        if ($IsWindows) {
            $null = New-Item -ItemType Junction -Path $junctionLanesPath `
                -Target $junctionOutsideTarget
        } else {
            $null = New-Item -ItemType SymbolicLink -Path $junctionLanesPath `
                -Target $junctionOutsideTarget
        }
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $junctionEvidenceRoot | Out-Null
        } -ExpectedMessage "contains a reparse entry outside platform ownership"
    } finally {
        if (Test-Path -LiteralPath $junctionLanesPath) {
            Remove-Item -LiteralPath $junctionLanesPath -Force
        }
    }
    if ((Test-Path -LiteralPath $junctionLanesPath) -or
        -not (Test-Path -LiteralPath $junctionOutsideSentinel -PathType Leaf) -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $junctionOutsideSentinel).Hash -cne
            $junctionOutsideSentinelHash -or
        @(Get-ChildItem -LiteralPath $junctionOutsideTarget -Force).Count -ne 1) {
        throw "A rejected preexisting lanes junction created or changed out-of-root evidence."
    }

    $overBudgetActions = Copy-JsonValue $adapter
    $overBudgetActionTemplate = Copy-JsonValue $overBudgetActions.lanes[0].actions[0]
    $overBudgetActionList = [Collections.Generic.List[object]]::new()
    for ($actionIndex = 0; $actionIndex -lt 25; $actionIndex++) {
        $overBudgetAction = Copy-JsonValue $overBudgetActionTemplate
        $overBudgetAction.id = "budget-action-$actionIndex"
        $overBudgetActionList.Add($overBudgetAction)
    }
    $overBudgetActions.lanes[0].actions = @($overBudgetActionList)
    $overBudgetActionsPath = Join-Path $root "over-budget-actions.json"
    Write-JsonFile -Path $overBudgetActionsPath -Document $overBudgetActions
    $overBudgetEvidenceRoot = Join-Path $root "over-budget-actions-evidence"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $overBudgetActionsPath `
            -LaneId ([string]$overBudgetActions.lanes[0].id) `
            -CandidateFingerprint $candidate -EvidenceRoot $overBudgetEvidenceRoot | Out-Null
    } -ExpectedMessage "Validation adapter does not satisfy contract v2"
    if (Test-Path -LiteralPath $overBudgetEvidenceRoot) {
        throw "An adapter with more than 24 actions created an attempt or evidence root."
    }

    $overTimeBudget = Copy-JsonValue $adapter
    $overTimeTemplate = Copy-JsonValue $overTimeBudget.lanes[0].actions[0]
    $overTimeActions = [Collections.Generic.List[object]]::new()
    for ($actionIndex = 0; $actionIndex -lt 3; $actionIndex++) {
        $overTimeAction = Copy-JsonValue $overTimeTemplate
        $overTimeAction.id = "time-budget-action-$actionIndex"
        $overTimeAction.timeout_seconds = 5000
        $overTimeActions.Add($overTimeAction)
    }
    $overTimeBudget.lanes[0].actions = @($overTimeActions)
    $overTimeBudgetPath = Join-Path $root "over-time-budget-actions.json"
    Write-JsonFile -Path $overTimeBudgetPath -Document $overTimeBudget
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $overTimeBudgetPath | Out-Null
    } -ExpectedMessage "14,400-second aggregate action budget"

    $mismatchedHandoffPorts = Copy-JsonValue $adapter
    $mismatchedHandoffConsumer = @($mismatchedHandoffPorts.lanes | Where-Object {
        [string]$_.id -ceq "compose-consume"
    })[0]
    $mismatchedHandoffConsumer.topology.ports[0].name = "consumer-http"
    $mismatchedHandoffConsumer.topology.readiness.port = "consumer-http"
    $mismatchedHandoffPortsPath = Join-Path $root "mismatched-handoff-ports.json"
    Write-JsonFile -Path $mismatchedHandoffPortsPath -Document $mismatchedHandoffPorts
    $mismatchedHandoffPortsRoot = Join-Path $root "mismatched-handoff-ports-evidence"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $mismatchedHandoffPortsPath `
            -LaneId "compose-create" -CandidateFingerprint $candidate `
            -EvidenceRoot $mismatchedHandoffPortsRoot | Out-Null
    } -ExpectedMessage "does not target one matching consume successor"
    if (Test-Path -LiteralPath $mismatchedHandoffPortsRoot) {
        throw "A producer/consumer port-name mismatch created an attempt or evidence root."
    }

    $overBudgetPorts = Copy-JsonValue $adapter
    $overBudgetPortLane = @($overBudgetPorts.lanes | Where-Object {
        [string]$_.id -ceq "local-success"
    })[0]
    for ($portIndex = 1; $portIndex -le 8; $portIndex++) {
        $overBudgetPortLane.topology.ports += [pscustomobject][ordered]@{
            name = "extra-$portIndex"
            environment = "TESSARA_VP_EXTRA_PORT_$portIndex"
        }
    }
    $overBudgetPortsPath = Join-Path $root "over-budget-ports.json"
    Write-JsonFile -Path $overBudgetPortsPath -Document $overBudgetPorts
    $overBudgetPortsRoot = Join-Path $root "over-budget-ports-evidence"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $overBudgetPortsPath `
            -LaneId "local-success" -CandidateFingerprint $candidate `
            -EvidenceRoot $overBudgetPortsRoot | Out-Null
    } -ExpectedMessage "Validation adapter does not satisfy contract v2"
    if (Test-Path -LiteralPath $overBudgetPortsRoot) {
        throw "An adapter with more than eight topology ports created an attempt or evidence root."
    }

    $nonMonotonic = Copy-JsonValue $adapter
    $setupAfterAssertion = Copy-JsonValue $nonMonotonic.lanes[0].actions[0]
    $setupAfterAssertion.id = "late-setup"
    $setupAfterAssertion.stage = "setup"
    $setupAfterAssertion.PSObject.Properties.Remove("implementation_target")
    $setupAfterAssertion.PSObject.Properties.Remove("proof_classes")
    $nonMonotonic.lanes[0].actions += $setupAfterAssertion
    $nonMonotonicPath = Join-Path $root "non-monotonic.json"
    Write-JsonFile -Path $nonMonotonicPath -Document $nonMonotonic
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $nonMonotonicPath | Out-Null
    } -ExpectedMessage "action stages are not monotonic"

    $reservedEnvironment = Copy-JsonValue $adapter
    $reservedEnvironmentLane = @($reservedEnvironment.lanes | Where-Object {
        $_.id -ceq "no-topology"
    })[0]
    $reservedEnvironmentLane.environment += [pscustomobject][ordered]@{
        name = "PATH"
        value = "synthetic-path-override"
    }
    $reservedEnvironmentPath = Join-Path $root "reserved-environment-adapter.json"
    Write-JsonFile -Path $reservedEnvironmentPath -Document $reservedEnvironment
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $reservedEnvironmentPath | Out-Null
    } -ExpectedMessage "platform-reserved environment variable"

    $gitPoisonEnvironmentNames = @("GIT_DIR", "GIT_INDEX_FILE")
    $gitPoisonEnvironmentBefore = [ordered]@{}
    foreach ($name in $gitPoisonEnvironmentNames) {
        $gitPoisonEnvironmentBefore[$name] = [pscustomobject][ordered]@{
            present = [Environment]::GetEnvironmentVariables("Process").Contains($name)
            value = [Environment]::GetEnvironmentVariable($name, "Process")
        }
    }
    try {
        [Environment]::SetEnvironmentVariable(
            "GIT_DIR", (Join-Path $root "poison-git-directory"), "Process"
        )
        [Environment]::SetEnvironmentVariable(
            "GIT_INDEX_FILE", (Join-Path $root "poison-git-index"), "Process"
        )
        $isolatedGitRoot = Join-Path $root "isolated-git"
        $isolatedGitResult = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $isolatedGitRoot
        Assert-Equal $isolatedGitResult.state "passed" "Poisoned caller Git environment lane state"
        Assert-EvidencePairs -AttemptRoot (Split-Path -Parent $isolatedGitResult.evidence_path)
    } finally {
        foreach ($name in $gitPoisonEnvironmentNames) {
            if ([bool]$gitPoisonEnvironmentBefore[$name].present) {
                [Environment]::SetEnvironmentVariable(
                    $name, [string]$gitPoisonEnvironmentBefore[$name].value, "Process"
                )
            } else {
                Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
            }
        }
    }
    foreach ($name in $gitPoisonEnvironmentNames) {
        $restoredPresent = [Environment]::GetEnvironmentVariables("Process").Contains($name)
        $restoredValue = [Environment]::GetEnvironmentVariable($name, "Process")
        if ($restoredPresent -ne [bool]$gitPoisonEnvironmentBefore[$name].present -or
            ($restoredPresent -and
                $restoredValue -cne [string]$gitPoisonEnvironmentBefore[$name].value)) {
            throw "Certification did not exactly restore caller environment variable '$name'."
        }
    }

    $noTopologyRoot = Join-Path $root "no-topology"
    $noTopology = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $noTopologyRoot
    Assert-Equal $noTopology.state "passed" "No-topology lane state"
    Assert-Equal $noTopology.assertions_started $true "No-topology assertion start"
    Assert-Equal $noTopology.assertions_completed $true "No-topology assertion completion"
    Assert-EvidencePairs -AttemptRoot (Split-Path -Parent $noTopology.evidence_path)

    $noTopologyAttemptRoot = Split-Path -Parent $noTopology.evidence_path
    $checkpointPath = Join-Path $noTopologyAttemptRoot "execution-complete.json"
    $executionIntegrityPath = Join-Path $noTopologyAttemptRoot `
        "execution-integrity-verified.json"
    $executionIntegrity = Get-Content -Raw -LiteralPath $executionIntegrityPath |
        ConvertFrom-Json -Depth 20
    Assert-Equal $executionIntegrity.state "verified" `
        "Positive post-checkpoint integrity state"
    Assert-Equal $executionIntegrity.execution_checkpoint.path $checkpointPath `
        "Positive post-checkpoint integrity checkpoint binding"
    $finalizationAttestationPath = Join-Path $noTopologyAttemptRoot `
        "finalization-attestation.$([string]$identity.finalization_fingerprint).json"
    $actionLogPath = [string]$noTopology.actions[0].stdout.path
    $actionLogHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $actionLogPath).Hash
    foreach ($publicationPath in @(
            (Join-Path $noTopologyAttemptRoot "lane-result.json"),
            (Join-Path $noTopologyAttemptRoot "lane-result.json.sha256"),
            (Join-Path $noTopologyAttemptRoot "attempt-index.json"),
            (Join-Path $noTopologyAttemptRoot "attempt-index.json.sha256"),
            $finalizationAttestationPath,
            "$finalizationAttestationPath.sha256"
        )) {
        Remove-Item -LiteralPath $publicationPath -Force
    }

    $lateExtraPath = Join-Path $noTopologyAttemptRoot "late-extra.txt"
    try {
        [IO.File]::WriteAllText(
            $lateExtraPath,
            "unlisted late evidence",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "inventory contains unlisted file: late-extra.txt"
        Assert-NoFinalizationPublication -AttemptRoot $noTopologyAttemptRoot
    } finally {
        Remove-Item -LiteralPath $lateExtraPath -Force -ErrorAction SilentlyContinue
    }

    $orphanSidecarPath = Join-Path $noTopologyAttemptRoot "orphan.txt.sha256"
    try {
        [IO.File]::WriteAllText(
            $orphanSidecarPath,
            "not a declared pair`n",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "orphan or unlisted digest sidecar: orphan.txt.sha256"
        Assert-NoFinalizationPublication -AttemptRoot $noTopologyAttemptRoot
    } finally {
        Remove-Item -LiteralPath $orphanSidecarPath -Force -ErrorAction SilentlyContinue
    }

    $oversizedPath = Join-Path $noTopologyAttemptRoot "oversized.bin"
    try {
        $oversizedStream = [IO.File]::Open(
            $oversizedPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None
        )
        try {
            $oversizedStream.SetLength(16MB + 1)
        } finally {
            $oversizedStream.Dispose()
        }
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "exceeds the 16777216-byte limit: oversized.bin"
        Assert-NoFinalizationPublication -AttemptRoot $noTopologyAttemptRoot
    } finally {
        Remove-Item -LiteralPath $oversizedPath -Force -ErrorAction SilentlyContinue
    }

    $directoryFloodRoot = Join-Path $noTopologyAttemptRoot "too-many-directories"
    try {
        [IO.Directory]::CreateDirectory($directoryFloodRoot) | Out-Null
        for ($directoryIndex = 0; $directoryIndex -lt 64; $directoryIndex++) {
            [IO.Directory]::CreateDirectory(
                (Join-Path $directoryFloodRoot "directory-$directoryIndex")
            ) | Out-Null
        }
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "exceeds the 64-directory limit"
        Assert-NoFinalizationPublication -AttemptRoot $noTopologyAttemptRoot
    } finally {
        Remove-Item -LiteralPath $directoryFloodRoot -Recurse -Force `
            -ErrorAction SilentlyContinue
    }

    $deepDirectoryRoot = Join-Path $noTopologyAttemptRoot "too-deep"
    try {
        $deepDirectoryPath = $deepDirectoryRoot
        [IO.Directory]::CreateDirectory($deepDirectoryPath) | Out-Null
        for ($depthIndex = 1; $depthIndex -le 8; $depthIndex++) {
            $deepDirectoryPath = Join-Path $deepDirectoryPath "level-$depthIndex"
            [IO.Directory]::CreateDirectory($deepDirectoryPath) | Out-Null
        }
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "exceeds the 8-level depth limit"
        Assert-NoFinalizationPublication -AttemptRoot $noTopologyAttemptRoot
    } finally {
        Remove-Item -LiteralPath $deepDirectoryRoot -Recurse -Force `
            -ErrorAction SilentlyContinue
    }

    $longRelativePathRoot = Join-Path $noTopologyAttemptRoot ("p" * 104)
    try {
        $longRelativePath = $longRelativePathRoot
        for ($pathIndex = 1; $pathIndex -lt 5; $pathIndex++) {
            $longRelativePath = Join-Path $longRelativePath ("p" * 104)
        }
        [IO.Directory]::CreateDirectory($longRelativePath) | Out-Null
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "exceeds the 512-character relative-path limit"
        Assert-NoFinalizationPublication -AttemptRoot $noTopologyAttemptRoot
    } finally {
        Remove-Item -LiteralPath $longRelativePathRoot -Recurse -Force `
            -ErrorAction SilentlyContinue
    }

    $reparseTarget = Join-Path $root "finalizer-reparse-target"
    $reparsePath = Join-Path $noTopologyAttemptRoot "late-reparse"
    [IO.Directory]::CreateDirectory($reparseTarget) | Out-Null
    try {
        if ($IsWindows) {
            $null = New-Item -ItemType Junction -Path $reparsePath -Target $reparseTarget
        } else {
            $null = New-Item -ItemType SymbolicLink -Path $reparsePath -Target $reparseTarget
        }
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "contains a reparse entry outside platform ownership"
        Assert-NoFinalizationPublication -AttemptRoot $noTopologyAttemptRoot
    } finally {
        if (Test-Path -LiteralPath $reparsePath) {
            Remove-Item -LiteralPath $reparsePath -Force
        }
    }

    Assert-Equal (Get-FileHash -Algorithm SHA256 -LiteralPath $actionLogPath).Hash `
        $actionLogHash "Rejected finalization inventory action immutability"
    $recoveredFinalization = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath
    Assert-Equal $recoveredFinalization.attempt_id $noTopology.attempt_id `
        "Finalization recovery attempt identity"
    Assert-Equal $recoveredFinalization.authoritative $true `
        "Finalization recovery authority"
    Assert-Equal (Get-FileHash -Algorithm SHA256 -LiteralPath $actionLogPath).Hash `
        $actionLogHash "Finalization recovery action immutability"
    $attemptIndexPath = Join-Path $noTopologyAttemptRoot "attempt-index.json"
    $stableIndexHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $attemptIndexPath).Hash
    $finalizerTestModule = @(Get-Module | Where-Object {
            -not [string]::IsNullOrWhiteSpace([string]$_.Path) -and
            [IO.Path]::GetFullPath([string]$_.Path) -ceq
                [IO.Path]::GetFullPath($finalizerModulePath)
        })
    if ($finalizerTestModule.Count -eq 0) {
        $finalizerTestModule = @(
            Import-Module $finalizerModulePath -Force -PassThru
        )
    }
    if ($finalizerTestModule.Count -ne 1) {
        throw "Could not identify the loaded finalizer module for writer-fault certification."
    }
    $writerFaultPublicationPaths = @(
        (Join-Path $noTopologyAttemptRoot "lane-result.json.sha256"),
        (Join-Path $noTopologyAttemptRoot "lane-result.json"),
        (Join-Path $noTopologyAttemptRoot "attempt-index.json.sha256"),
        $attemptIndexPath,
        "$finalizationAttestationPath.sha256",
        $finalizationAttestationPath
    )
    $writerFaultCases = @(
        [pscustomobject]@{ id = "result-data"; target_name = "lane-result.json" }
        [pscustomobject]@{ id = "result-sidecar"; target_name = "lane-result.json.sha256" }
        [pscustomobject]@{ id = "index-data"; target_name = "attempt-index.json" }
        [pscustomobject]@{ id = "index-sidecar"; target_name = "attempt-index.json.sha256" }
        [pscustomobject]@{
            id = "attestation-data"
            target_name = [IO.Path]::GetFileName($finalizationAttestationPath)
        }
        [pscustomobject]@{
            id = "attestation-sidecar"
            target_name = [IO.Path]::GetFileName("$finalizationAttestationPath.sha256")
        }
    )
    foreach ($writerFaultCase in $writerFaultCases) {
        foreach ($publicationPath in $writerFaultPublicationPaths) {
            Remove-Item -LiteralPath $publicationPath -Force
        }
        $faultTargetPath = Join-Path $noTopologyAttemptRoot `
            ([string]$writerFaultCase.target_name)
        Assert-Rejected -Action {
            & $finalizerTestModule[0] {
                param([Parameter(Mandatory)][string]$TargetPath)
                Write-TessaraFinalizerCreateNewBytes -Path $TargetPath `
                    -Bytes ([byte[]](1, 2, 3)) -Label "Injected finalizer writer" `
                    -AfterCreate { throw "Injected finalizer create-new write failure." }
            } $faultTargetPath
        } -ExpectedMessage "Injected finalizer create-new write failure"
        if (Test-Path -LiteralPath $faultTargetPath) {
            throw ("Finalizer writer fault '$($writerFaultCase.id)' retained its " +
                "invocation-owned partial member.")
        }
        $writerFaultRecovery = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath
        Assert-Equal $writerFaultRecovery.attempt_id $noTopology.attempt_id `
            "Finalizer writer fault $($writerFaultCase.id) recovery attempt identity"
        Assert-Equal $writerFaultRecovery.authoritative $true `
            "Finalizer writer fault $($writerFaultCase.id) recovery authority"
        Assert-Equal (Get-FileHash -Algorithm SHA256 -LiteralPath $actionLogPath).Hash `
            $actionLogHash "Finalizer writer fault $($writerFaultCase.id) action immutability"
        Assert-Equal (Get-FileHash -Algorithm SHA256 -LiteralPath $attemptIndexPath).Hash `
            $stableIndexHash "Finalizer writer fault $($writerFaultCase.id) stable index"
        Assert-EvidencePairs -AttemptRoot $noTopologyAttemptRoot
    }
    $priorFinalizationFingerprint = if (
        [string]$identity.finalization_fingerprint -cne ("0" * 64)
    ) { "0" * 64 } else { "f" * 64 }
    $priorAttestationPath = Join-Path $noTopologyAttemptRoot `
        "finalization-attestation.$priorFinalizationFingerprint.json"
    $priorAttestation = Get-Content -Raw -LiteralPath $finalizationAttestationPath |
        ConvertFrom-Json -Depth 100
    $priorAttestation.finalization_platform_fingerprint = $priorFinalizationFingerprint
    Write-JsonFile -Path $priorAttestationPath -Document $priorAttestation
    $priorAttestationHash = (Get-FileHash -Algorithm SHA256 `
        -LiteralPath $priorAttestationPath).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText(
        "$priorAttestationPath.sha256",
        "$priorAttestationHash  $([IO.Path]::GetFileName($priorAttestationPath))`n",
        [Text.UTF8Encoding]::new($false)
    )
    Remove-Item -LiteralPath "$finalizationAttestationPath.sha256" -Force
    Remove-Item -LiteralPath $finalizationAttestationPath -Force
    $reattestedFinalization = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath
    Assert-Equal $reattestedFinalization.attempt_id $noTopology.attempt_id `
        "Current-finalizer re-attestation attempt identity"
    Assert-Equal (Get-FileHash -Algorithm SHA256 -LiteralPath $attemptIndexPath).Hash `
        $stableIndexHash "Current-finalizer re-attestation stable index"
    Assert-Equal (Get-FileHash -Algorithm SHA256 -LiteralPath $actionLogPath).Hash `
        $actionLogHash "Current-finalizer re-attestation action immutability"
    foreach ($attestationPairPath in @(
            $priorAttestationPath, "$priorAttestationPath.sha256",
            $finalizationAttestationPath, "$finalizationAttestationPath.sha256"
        )) {
        if (-not (Test-Path -LiteralPath $attestationPairPath -PathType Leaf)) {
            throw "Finalization re-attestation did not retain exact append-only pairs."
        }
    }

    Remove-Item -LiteralPath "$finalizationAttestationPath.sha256" -Force
    $postAttestationExtraPath = Join-Path $noTopologyAttemptRoot `
        "post-attestation-extra.txt"
    try {
        [IO.File]::WriteAllText(
            $postAttestationExtraPath,
            "injected after finalization attestation data publication",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath | Out-Null
        } -ExpectedMessage "inventory contains unlisted file: post-attestation-extra.txt"
        if (Test-Path -LiteralPath "$finalizationAttestationPath.sha256") {
            throw "Rejected post-attestation finalization published its completion sidecar."
        }
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot `
                -PrerequisiteResultPaths @($noTopology.evidence_path) | Out-Null
        } -ExpectedMessage "Prerequisite finalization attestation pair is missing"
        $postAttestationDependent = Get-LaneResult -EvidenceRoot $noTopologyRoot `
            -LaneId "dependent-no-topology"
        if ([string]$postAttestationDependent.state -ceq "passed" -or
            [bool]$postAttestationDependent.assertions_started) {
            throw "A consumer accepted an attempt without its current finalization attestation."
        }
    } finally {
        Remove-Item -LiteralPath $postAttestationExtraPath -Force `
            -ErrorAction SilentlyContinue
    }
    $idempotentFinalization = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $checkpointPath
    Assert-Equal $idempotentFinalization.attempt_id $noTopology.attempt_id `
        "Idempotent finalization recovery"

    $currentAttestationBytes = [IO.File]::ReadAllBytes($finalizationAttestationPath)
    $currentAttestationSidecarBytes = [IO.File]::ReadAllBytes(
        "$finalizationAttestationPath.sha256"
    )
    try {
        $tamperedAttestationBytes = [byte[]]$currentAttestationBytes.Clone()
        $tamperedAttestationBytes[0] = [byte]($tamperedAttestationBytes[0] -bxor 1)
        [IO.File]::WriteAllBytes($finalizationAttestationPath, $tamperedAttestationBytes)
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot `
                -PrerequisiteResultPaths @($noTopology.evidence_path) | Out-Null
        } -ExpectedMessage "Prerequisite finalization attestation digest sidecar is invalid"
        $tamperedAttestationDependent = Get-LaneResult -EvidenceRoot $noTopologyRoot `
            -LaneId "dependent-no-topology"
        if ([string]$tamperedAttestationDependent.state -ceq "passed" -or
            [bool]$tamperedAttestationDependent.assertions_started) {
            throw "A consumer accepted a digest-tampered finalization attestation."
        }
    } finally {
        [IO.File]::WriteAllBytes($finalizationAttestationPath, $currentAttestationBytes)
        [IO.File]::WriteAllBytes(
            "$finalizationAttestationPath.sha256", $currentAttestationSidecarBytes
        )
    }

    try {
        $staleAttestation = [Text.UTF8Encoding]::new($false, $true).GetString(
            $currentAttestationBytes
        ) | ConvertFrom-Json -Depth 100
        $staleAttestation.finalization_platform_fingerprint = `
            $priorFinalizationFingerprint
        $staleAttestationBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (($staleAttestation | ConvertTo-Json -Depth 100 -Compress) + "`n")
        )
        [IO.File]::WriteAllBytes($finalizationAttestationPath, $staleAttestationBytes)
        $staleAttestationHash = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($staleAttestationBytes)
        ).ToLowerInvariant()
        [IO.File]::WriteAllText(
            "$finalizationAttestationPath.sha256",
            "$staleAttestationHash  $([IO.Path]::GetFileName($finalizationAttestationPath))`n",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot `
                -PrerequisiteResultPaths @($noTopology.evidence_path) | Out-Null
        } -ExpectedMessage "Prerequisite finalization commit authentication failed"
        $staleAttestationDependent = Get-LaneResult -EvidenceRoot $noTopologyRoot `
            -LaneId "dependent-no-topology"
        if ([string]$staleAttestationDependent.state -ceq "passed" -or
            [bool]$staleAttestationDependent.assertions_started) {
            throw "A consumer accepted a stale-finalizer completion attestation."
        }
    } finally {
        [IO.File]::WriteAllBytes($finalizationAttestationPath, $currentAttestationBytes)
        [IO.File]::WriteAllBytes(
            "$finalizationAttestationPath.sha256", $currentAttestationSidecarBytes
        )
    }

    $currentIndexBytes = [IO.File]::ReadAllBytes($attemptIndexPath)
    $currentIndexSidecarBytes = [IO.File]::ReadAllBytes("$attemptIndexPath.sha256")
    try {
        $tamperedIndex = [Text.UTF8Encoding]::new($false, $true).GetString(
            $currentIndexBytes
        ) | ConvertFrom-Json -Depth 100
        $tamperedIndex.files = @()
        $tamperedIndexBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (($tamperedIndex | ConvertTo-Json -Depth 100 -Compress) + "`n")
        )
        [IO.File]::WriteAllBytes($attemptIndexPath, $tamperedIndexBytes)
        $tamperedIndexHash = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($tamperedIndexBytes)
        ).ToLowerInvariant()
        [IO.File]::WriteAllText(
            "$attemptIndexPath.sha256",
            "$tamperedIndexHash  $([IO.Path]::GetFileName($attemptIndexPath))`n",
            [Text.UTF8Encoding]::new($false)
        )
        $tamperedIndexAttestation = `
            [Text.UTF8Encoding]::new($false, $true).GetString(
                $currentAttestationBytes
            ) | ConvertFrom-Json -Depth 100
        $tamperedIndexAttestation.attempt_index.sha256 = $tamperedIndexHash
        $tamperedIndexAttestationBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (($tamperedIndexAttestation | ConvertTo-Json -Depth 100 -Compress) + "`n")
        )
        [IO.File]::WriteAllBytes(
            $finalizationAttestationPath, $tamperedIndexAttestationBytes
        )
        $tamperedIndexAttestationHash = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($tamperedIndexAttestationBytes)
        ).ToLowerInvariant()
        [IO.File]::WriteAllText(
            "$finalizationAttestationPath.sha256",
            "$tamperedIndexAttestationHash  $([IO.Path]::GetFileName($finalizationAttestationPath))`n",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot `
                -PrerequisiteResultPaths @($noTopology.evidence_path) | Out-Null
        } -ExpectedMessage "Prerequisite finalization commit authentication failed"
        $tamperedIndexDependent = Get-LaneResult -EvidenceRoot $noTopologyRoot `
            -LaneId "dependent-no-topology"
        if ([string]$tamperedIndexDependent.state -ceq "passed" -or
            [bool]$tamperedIndexDependent.assertions_started) {
            throw "A consumer accepted a semantically tampered attempt index."
        }
    } finally {
        [IO.File]::WriteAllBytes($attemptIndexPath, $currentIndexBytes)
        [IO.File]::WriteAllBytes("$attemptIndexPath.sha256", $currentIndexSidecarBytes)
        [IO.File]::WriteAllBytes($finalizationAttestationPath, $currentAttestationBytes)
        [IO.File]::WriteAllBytes(
            "$finalizationAttestationPath.sha256", $currentAttestationSidecarBytes
        )
    }

    $finalPrerequisiteRecheckLine = @(Select-String `
        -LiteralPath $finalizerModulePath -SimpleMatch `
        'if ($null -eq $ResultReference -or')
    if ($finalPrerequisiteRecheckLine.Count -ne 1) {
        throw "Could not identify the exact final prerequisite recheck boundary."
    }
    $dependentAttemptsRoot = Join-Path $noTopologyRoot `
        "lanes/dependent-no-topology/attempts"

    $beforeRevocationCheckpoints = @(
        Get-ChildItem -LiteralPath $dependentAttemptsRoot `
            -Filter "execution-complete.json" -File -Recurse -ErrorAction SilentlyContinue |
            ForEach-Object { $_.FullName }
    )
    $lateRevocationBreakpoint = Set-PSBreakpoint -Script $finalizerModulePath `
        -Line $finalPrerequisiteRecheckLine[0].LineNumber -Action {
            $global:TessaraFinalPrerequisiteCallCount++
            if ($global:TessaraFinalPrerequisiteCallCount -eq 2) {
                $sourceAttemptRoot = $global:TessaraFinalPrerequisiteSourceAttemptRoot
                $sourceRevocationPath = Join-Path $sourceAttemptRoot "execution-revoked.json"
                $sourceRevocationText = `
                    '{"schema_version":1,"contract":"tessara.validation.execution-revocation","state":"revoked","reason":"injected-late-prerequisite-revocation"}' + "`n"
                [IO.File]::WriteAllText(
                    $sourceRevocationPath,
                    $sourceRevocationText,
                    [Text.UTF8Encoding]::new($false)
                )
                $sourceRevocationHash = (Get-FileHash -Algorithm SHA256 `
                    -LiteralPath $sourceRevocationPath).Hash.ToLowerInvariant()
                [IO.File]::WriteAllText(
                    "$sourceRevocationPath.sha256",
                    "$sourceRevocationHash  execution-revoked.json`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
    $global:TessaraFinalPrerequisiteCallCount = 0
    $global:TessaraFinalPrerequisiteSourceAttemptRoot = $noTopologyAttemptRoot
    try {
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot `
                -PrerequisiteResultPaths @($noTopology.evidence_path) | Out-Null
        } -ExpectedMessage "revoked before dependent finalization"
    } finally {
        Remove-PSBreakpoint -Breakpoint $lateRevocationBreakpoint
        Remove-Variable -Name TessaraFinalPrerequisiteCallCount -Scope Global `
            -ErrorAction SilentlyContinue
        Remove-Variable -Name TessaraFinalPrerequisiteSourceAttemptRoot `
            -Scope Global -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath (Join-Path $noTopologyAttemptRoot `
                "execution-revoked.json.sha256") -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath (Join-Path $noTopologyAttemptRoot `
                "execution-revoked.json") -Force -ErrorAction SilentlyContinue
    }
    $revocationCheckpoints = @(
        Get-ChildItem -LiteralPath $dependentAttemptsRoot `
            -Filter "execution-complete.json" -File -Recurse |
            Where-Object { $_.FullName -notin $beforeRevocationCheckpoints }
    )
    if ($revocationCheckpoints.Count -ne 1) {
        throw "Late prerequisite revocation did not retain one dependent checkpoint."
    }
    $revocationCheckpointPath = $revocationCheckpoints[0].FullName
    $revocationCheckpoint = Get-Content -Raw -LiteralPath $revocationCheckpointPath |
        ConvertFrom-Json -Depth 100
    $revocationAttemptRoot = Split-Path -Parent $revocationCheckpointPath
    $revocationAttestationPath = Join-Path $revocationAttemptRoot `
        "finalization-attestation.$([string]$identity.finalization_fingerprint).json"
    if (Test-Path -LiteralPath "$revocationAttestationPath.sha256") {
        throw "Late prerequisite revocation committed dependent finalization."
    }
    $revocationActionLogPath = [string]$revocationCheckpoint.result.actions[0].stdout.path
    $revocationActionLogHash = (Get-FileHash -Algorithm SHA256 `
        -LiteralPath $revocationActionLogPath).Hash
    $revocationRecovered = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $revocationCheckpointPath
    Assert-Equal $revocationRecovered.attempt_id `
        ([string]$revocationCheckpoint.result.attempt_id) `
        "Late prerequisite revocation finalization-only recovery"
    Assert-Equal (Get-FileHash -Algorithm SHA256 `
            -LiteralPath $revocationActionLogPath).Hash $revocationActionLogHash `
        "Late prerequisite revocation action immutability"

    $sourceIndexSidecarPath = "$attemptIndexPath.sha256"
    $sourceIndexSidecarBytes = [IO.File]::ReadAllBytes($sourceIndexSidecarPath)
    $beforeIndexLossCheckpoints = @(
        Get-ChildItem -LiteralPath $dependentAttemptsRoot `
            -Filter "execution-complete.json" -File -Recurse |
            ForEach-Object { $_.FullName }
    )
    $lateIndexLossBreakpoint = Set-PSBreakpoint -Script $finalizerModulePath `
        -Line $finalPrerequisiteRecheckLine[0].LineNumber -Action {
            $global:TessaraFinalPrerequisiteCallCount++
            if ($global:TessaraFinalPrerequisiteCallCount -eq 2) {
                $sourceAttemptRoot = $global:TessaraFinalPrerequisiteSourceAttemptRoot
                [IO.File]::Delete((Join-Path $sourceAttemptRoot `
                    "attempt-index.json.sha256"))
            }
        }
    $global:TessaraFinalPrerequisiteCallCount = 0
    $global:TessaraFinalPrerequisiteSourceAttemptRoot = $noTopologyAttemptRoot
    try {
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $noTopologyRoot `
                -PrerequisiteResultPaths @($noTopology.evidence_path) | Out-Null
        } -ExpectedMessage "Final prerequisite recheck attempt index pair is missing"
    } finally {
        Remove-PSBreakpoint -Breakpoint $lateIndexLossBreakpoint
        Remove-Variable -Name TessaraFinalPrerequisiteCallCount -Scope Global `
            -ErrorAction SilentlyContinue
        Remove-Variable -Name TessaraFinalPrerequisiteSourceAttemptRoot `
            -Scope Global -ErrorAction SilentlyContinue
        [IO.File]::WriteAllBytes($sourceIndexSidecarPath, $sourceIndexSidecarBytes)
    }
    $indexLossCheckpoints = @(
        Get-ChildItem -LiteralPath $dependentAttemptsRoot `
            -Filter "execution-complete.json" -File -Recurse |
            Where-Object { $_.FullName -notin $beforeIndexLossCheckpoints }
    )
    if ($indexLossCheckpoints.Count -ne 1) {
        throw "Late prerequisite index loss did not retain one dependent checkpoint."
    }
    $indexLossCheckpointPath = $indexLossCheckpoints[0].FullName
    $indexLossCheckpoint = Get-Content -Raw -LiteralPath $indexLossCheckpointPath |
        ConvertFrom-Json -Depth 100
    $indexLossAttemptRoot = Split-Path -Parent $indexLossCheckpointPath
    $indexLossAttestationPath = Join-Path $indexLossAttemptRoot `
        "finalization-attestation.$([string]$identity.finalization_fingerprint).json"
    if (Test-Path -LiteralPath "$indexLossAttestationPath.sha256") {
        throw "Late prerequisite index loss committed dependent finalization."
    }
    $indexLossActionLogPath = [string]$indexLossCheckpoint.result.actions[0].stdout.path
    $indexLossActionLogHash = (Get-FileHash -Algorithm SHA256 `
        -LiteralPath $indexLossActionLogPath).Hash
    $indexLossRecovered = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $noTopologyRoot -FinalizeCheckpointPath $indexLossCheckpointPath
    Assert-Equal $indexLossRecovered.attempt_id `
        ([string]$indexLossCheckpoint.result.attempt_id) `
        "Late prerequisite index-loss finalization-only recovery"
    Assert-Equal (Get-FileHash -Algorithm SHA256 `
            -LiteralPath $indexLossActionLogPath).Hash $indexLossActionLogHash `
        "Late prerequisite index-loss action immutability"

    [Environment]::SetEnvironmentVariable("TESSARA_VP_SENTINEL", "caller-sentinel", "Process")
    [Environment]::SetEnvironmentVariable("TESSARA_VP_UNDECLARED", "poison", "Process")
    $localRoot = Join-Path $root "local"
    $local = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "local-success" -CandidateFingerprint $candidate -EvidenceRoot $localRoot
    Assert-Equal $local.state "passed" "Local-process lane state"
    Assert-Equal $local.cleanup_restoration.mode "local-process-tree-destroyed" `
        "Local-process cleanup"
    Assert-Equal $local.cleanup_restoration.environment_restoration.state "passed" `
        "Local-process environment restoration state"
    Assert-Equal $env:TESSARA_VP_SENTINEL "caller-sentinel" "Caller environment restoration"
    Assert-Equal $env:TESSARA_VP_UNDECLARED "poison" "Undeclared caller environment preservation"
    if (-not (Test-PortClosed -Port ([int]$local.ports.http))) {
        throw "Local-process topology retained its leased port after cleanup."
    }
    Assert-EvidencePairs -AttemptRoot (Split-Path -Parent $local.evidence_path)

    $localAcquisitionRoot = Join-Path $root "local-acquisition-failure"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "local-success" -CandidateFingerprint $candidate `
            -EvidenceRoot $localAcquisitionRoot `
            -CertificationFault "local-topology-after-start" | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $localAcquisitionResult = Get-LaneResult -EvidenceRoot $localAcquisitionRoot `
        -LaneId "local-success"
    if ([bool]$localAcquisitionResult.assertions_started -or
        [string]$localAcquisitionResult.failure.message -notlike
            "*Injected failure after local topology process start.*" -or
        -not (Test-PortClosed -Port ([int]$localAcquisitionResult.ports.http))) {
        throw "A post-start local topology acquisition fault escaped cleanup containment."
    }

    $actionAcquisitionAdapter = Copy-JsonValue $adapter
    $actionAcquisitionLane = @($actionAcquisitionAdapter.lanes | Where-Object {
        [string]$_.id -ceq "no-topology"
    })[0]
    $actionAcquisitionPath = Join-Path $root "action-acquisition-adapter.json"
    Write-JsonFile -Path $actionAcquisitionPath -Document $actionAcquisitionAdapter
    $actionAcquisitionRoot = Join-Path $root "action-acquisition-failure"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $actionAcquisitionPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $actionAcquisitionRoot `
            -CertificationFault "assertion-after-process-acquisition" | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $actionAcquisitionResult = Get-LaneResult -EvidenceRoot $actionAcquisitionRoot `
        -LaneId "no-topology"
    if (-not [bool]$actionAcquisitionResult.assertions_started -or
        [string]$actionAcquisitionResult.failure_stage -cne "assertion" -or
        [string]$actionAcquisitionResult.failure.message -notlike
            "*Injected failure after assertion process start.*") {
        throw "A post-start assertion acquisition fault escaped cleanup or start accounting."
    }

    $postSetupToolAdapter = Copy-JsonValue $adapter
    $postSetupToolLane = @($postSetupToolAdapter.lanes | Where-Object {
        [string]$_.id -ceq "local-success"
    })[0]
    @($postSetupToolAdapter.execution_inputs | Where-Object {
        [string]$_.path -ceq
            "scripts/validation-platform/fixtures/synthetic-action.ps1"
    })[0].lanes += "local-success"
    $postSetupToolLane.actions = @(
        [pscustomobject][ordered]@{
            id = "pre-topology-setup"
            stage = "setup"
            program = "pwsh"
            arguments = @(
                "-NoProfile", "-NonInteractive", "-File",
                '${repository_root}/scripts/validation-platform/fixtures/synthetic-action.ps1',
                "-Mode", "success"
            )
            input_paths = @(
                "scripts/validation-platform/fixtures/synthetic-action.ps1"
            )
            timeout_seconds = 10
            tools = @()
        }
        $postSetupToolLane.actions[0]
    )
    $postSetupToolPath = Join-Path $root "post-setup-topology-tool-adapter.json"
    Write-JsonFile -Path $postSetupToolPath -Document $postSetupToolAdapter
    $postSetupToolRoot = Join-Path $root "post-setup-topology-tool"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $postSetupToolPath `
            -LaneId "local-success" -CandidateFingerprint $candidate `
            -EvidenceRoot $postSetupToolRoot `
            -CertificationFault "after-post-setup-verification" | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $postSetupToolResult = Get-LaneResult -EvidenceRoot $postSetupToolRoot `
        -LaneId "local-success"
    Assert-Equal $postSetupToolResult.assertions_started $false `
        "Post-setup topology tool drift assertion state"
    if ([string]$postSetupToolResult.failure.message -notlike
            "*identity changed immediately before invocation*" -or
        (Test-Path -LiteralPath (Join-Path (Split-Path -Parent `
                $postSetupToolResult.evidence_path) "topology-ownership-acquired.json")) -or
        -not (Test-PortClosed -Port ([int]$postSetupToolResult.ports.http))) {
        throw "Post-setup topology tool drift was not contained before topology ownership/start."
    }

    $blockedDependentRoot = $noTopologyRoot
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $blockedDependentRoot | Out-Null
    } -ExpectedMessage "finished 'blocked'"
    $blockedDependent = Get-LaneResult -EvidenceRoot $blockedDependentRoot `
        -LaneId "dependent-no-topology"
    Assert-Equal $blockedDependent.assertions_started $false `
        "Blocked prerequisite assertion state"
    $dependent = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $blockedDependentRoot `
        -PrerequisiteResultPaths @($noTopology.evidence_path)
    Assert-Equal $dependent.state "passed" "Prerequisite-authorized lane state"
    Assert-Equal @($dependent.prerequisite_results).Count 1 `
        "Prerequisite result count"

    $env:TESSARA_VP_LANE_A_SOURCE = "changed-lane-a-environment"
    $impactRoot = Join-Path $root "selective-impact"
    $changedA = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "no-topology" -CandidateFingerprint $candidate -EvidenceRoot $impactRoot
    $unchangedB = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "local-success" -CandidateFingerprint $candidate -EvidenceRoot $impactRoot
    $changedC = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "dependent-no-topology" -CandidateFingerprint $candidate `
        -EvidenceRoot $impactRoot -PrerequisiteResultPaths @($changedA.evidence_path)
    if ([string]$changedA.lane_compatibility_fingerprint -ceq
            [string]$noTopology.lane_compatibility_fingerprint -or
        [string]$changedC.lane_compatibility_fingerprint -ceq
            [string]$dependent.lane_compatibility_fingerprint) {
        throw "A declared environment change did not invalidate its lane and dependent closure."
    }
    Assert-Equal $unchangedB.lane_compatibility_fingerprint `
        $local.lane_compatibility_fingerprint `
        "Independent lane compatibility after unrelated environment change"
    Remove-Item Env:TESSARA_VP_LANE_A_SOURCE -ErrorAction SilentlyContinue

    $plannerRepository = Join-Path $root "planner-mutation-repository"
    foreach ($directory in @(
            "acceptance", "fixtures", "harness", "domain", "artifacts/evidence"
        )) {
        [IO.Directory]::CreateDirectory((Join-Path $plannerRepository $directory)) |
            Out-Null
    }
    & git -C $plannerRepository init --quiet
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to initialize the disposable planner-mutation repository."
    }
    & git -C $plannerRepository config core.autocrlf false
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to pin line-ending behavior in the disposable planner repository."
    }
    $plannerContract = [pscustomobject][ordered]@{
        schema_version = 2
        contract = "tessara.validation-contract"
        policy_version = "tessara-validation-v2"
        sprint = "sprint-0a-planner-mutation"
        implementation_profile = [pscustomobject][ordered]@{ kind = "standard" }
        requirements = @(
            [pscustomobject][ordered]@{
                id = "owner-requirement"
                implementation_targets = @("owner-target")
                validation_lanes = @("no-topology")
            }
            [pscustomobject][ordered]@{
                id = "independent-requirement"
                implementation_targets = @("independent-target")
                validation_lanes = @("local-success")
            }
            [pscustomobject][ordered]@{
                id = "dependent-requirement"
                implementation_targets = @("dependent-target")
                validation_lanes = @("dependent-no-topology")
            }
            [pscustomobject][ordered]@{
                id = "neutral-requirement"
                implementation_targets = @("neutral-target")
                validation_lanes = @(
                    "rehearsal-neutral", "preflight-neutral", "sit-neutral"
                )
            }
        )
        dependency_domains = @(
            [pscustomobject][ordered]@{
                name = "owner-domain"
                tracked_inputs = @("domain/owner.txt")
                environment_sections = @()
            }
            [pscustomobject][ordered]@{
                name = "independent-domain"
                tracked_inputs = @("domain/independent.txt")
                environment_sections = @()
            }
            [pscustomobject][ordered]@{
                name = "dependent-domain"
                tracked_inputs = @("domain/dependent.txt")
                environment_sections = @()
            }
            [pscustomobject][ordered]@{
                name = "neutral-domain"
                tracked_inputs = @("domain/neutral.txt")
                environment_sections = @()
            }
        )
        implementation_targets = @(
            [pscustomobject][ordered]@{
                id = "owner-target"
                command = "planner owner target"
                dependency_domains = @("owner-domain")
                proof_classes = @("runner-selftest")
                required = $true
                clean_environment = $true
            }
            [pscustomobject][ordered]@{
                id = "independent-target"
                command = "planner independent target"
                dependency_domains = @("independent-domain")
                proof_classes = @("runner-selftest")
                required = $true
                clean_environment = $true
            }
            [pscustomobject][ordered]@{
                id = "dependent-target"
                command = "planner dependent target"
                dependency_domains = @("dependent-domain")
                proof_classes = @("runner-selftest")
                required = $true
                clean_environment = $true
            }
            [pscustomobject][ordered]@{
                id = "neutral-target"
                command = "planner neutral target"
                dependency_domains = @("neutral-domain")
                proof_classes = @("runner-selftest")
                required = $true
                clean_environment = $true
            }
        )
        lanes = @(
            [pscustomobject][ordered]@{
                id = "no-topology"
                phase = "validation-readiness"
                dependency_domains = @("owner-domain")
                prerequisites = @()
                touches_live_state = $false
            }
            [pscustomobject][ordered]@{
                id = "local-success"
                phase = "validation-readiness"
                dependency_domains = @("independent-domain")
                prerequisites = @()
                touches_live_state = $false
            }
            [pscustomobject][ordered]@{
                id = "dependent-no-topology"
                phase = "uat"
                dependency_domains = @("dependent-domain")
                prerequisites = @("no-topology")
                touches_live_state = $false
            }
            [pscustomobject][ordered]@{
                id = "rehearsal-neutral"
                phase = "candidate-rehearsal"
                dependency_domains = @("neutral-domain")
                prerequisites = @()
                touches_live_state = $false
            }
            [pscustomobject][ordered]@{
                id = "preflight-neutral"
                phase = "validation-preflight"
                dependency_domains = @("neutral-domain")
                prerequisites = @()
                touches_live_state = $false
            }
            [pscustomobject][ordered]@{
                id = "sit-neutral"
                phase = "sit"
                dependency_domains = @("neutral-domain")
                prerequisites = @()
                touches_live_state = $false
            }
        )
        evidence_policy = [pscustomobject][ordered]@{
            root = "artifacts/evidence"
            tracked = $false
            successful_raw = "retained_cold"
            phase_local_indexes = $true
            final_full_integrity_audit = $true
        }
    }
    $plannerAdapter = [pscustomobject][ordered]@{
        schema_version = 2
        contract = "tessara.validation.adapter"
        adapter_id = "planner-mutation"
        validation_contract_path = "contract.json"
        acceptance_inputs = @(
            [pscustomobject][ordered]@{
                path = "acceptance/no-topology.json"
                lanes = @("no-topology")
            }
            [pscustomobject][ordered]@{
                path = "acceptance/local-success.json"
                lanes = @("local-success")
            }
            [pscustomobject][ordered]@{
                path = "acceptance/dependent-no-topology.json"
                lanes = @("dependent-no-topology")
            }
            [pscustomobject][ordered]@{
                path = "acceptance/neutral.json"
                lanes = @("rehearsal-neutral", "preflight-neutral", "sit-neutral")
            }
        )
        fixture_inputs = @(
            [pscustomobject][ordered]@{
                path = "fixtures/no-topology.txt"
                lanes = @("no-topology")
            }
        )
        execution_inputs = @(
            [pscustomobject][ordered]@{
                path = "harness/no-topology.ps1"
                lanes = @("no-topology")
            }
            [pscustomobject][ordered]@{
                path = "harness/owner-config.txt"
                lanes = @("no-topology")
            }
            [pscustomobject][ordered]@{
                path = "harness/owner-response.rsp"
                lanes = @("no-topology")
            }
            [pscustomobject][ordered]@{
                path = "harness/local-success.ps1"
                lanes = @("local-success")
            }
            [pscustomobject][ordered]@{
                path = "harness/dependent-no-topology.ps1"
                lanes = @("dependent-no-topology")
            }
            [pscustomobject][ordered]@{
                path = "harness/neutral.ps1"
                lanes = @("rehearsal-neutral", "preflight-neutral", "sit-neutral")
            }
        )
        lanes = @(
            [pscustomobject][ordered]@{
                id = "no-topology"
                prerequisites = @()
                environment = @()
                blocked_environment = @("DATABASE_URL")
                topology = [pscustomobject][ordered]@{
                    mode = "none"
                    provider = "none"
                    on_success = "destroy"
                }
                actions = @([pscustomobject][ordered]@{
                    id = "owner-assertion"
                    stage = "assertion"
                    program = "pwsh"
                    arguments = @(
                        "-NoProfile", "-NonInteractive", "-File",
                        '${repository_root}/harness/no-topology.ps1',
                        '--config=${repository_root}/harness/owner-config.txt',
                        '@${repository_root}/harness/owner-response.rsp'
                    )
                    input_paths = @(
                        "harness/no-topology.ps1",
                        "harness/owner-config.txt",
                        "harness/owner-response.rsp"
                    )
                    timeout_seconds = 10
                })
            }
            [pscustomobject][ordered]@{
                id = "local-success"
                prerequisites = @()
                environment = @()
                blocked_environment = @("DATABASE_URL")
                topology = [pscustomobject][ordered]@{
                    mode = "none"
                    provider = "none"
                    on_success = "destroy"
                }
                actions = @([pscustomobject][ordered]@{
                    id = "independent-assertion"
                    stage = "assertion"
                    program = "pwsh"
                    arguments = @(
                        "-NoProfile", "-NonInteractive", "-File",
                        '${repository_root}/harness/local-success.ps1'
                    )
                    input_paths = @("harness/local-success.ps1")
                    timeout_seconds = 10
                })
            }
            [pscustomobject][ordered]@{
                id = "dependent-no-topology"
                prerequisites = @("no-topology")
                environment = @()
                blocked_environment = @("DATABASE_URL")
                topology = [pscustomobject][ordered]@{
                    mode = "none"
                    provider = "none"
                    on_success = "destroy"
                }
                actions = @([pscustomobject][ordered]@{
                    id = "dependent-assertion"
                    stage = "assertion"
                    program = "pwsh"
                    arguments = @(
                        "-NoProfile", "-NonInteractive", "-File",
                        '${repository_root}/harness/dependent-no-topology.ps1'
                    )
                    input_paths = @("harness/dependent-no-topology.ps1")
                    timeout_seconds = 10
                })
            }
            [pscustomobject][ordered]@{
                id = "rehearsal-neutral"
                prerequisites = @()
                environment = @()
                blocked_environment = @("DATABASE_URL")
                topology = [pscustomobject][ordered]@{
                    mode = "none"
                    provider = "none"
                    on_success = "destroy"
                }
                actions = @([pscustomobject][ordered]@{
                    id = "rehearsal-neutral-assertion"
                    stage = "assertion"
                    program = "pwsh"
                    arguments = @(
                        "-NoProfile", "-NonInteractive", "-File", '${repository_root}/harness/neutral.ps1'
                    )
                    input_paths = @("harness/neutral.ps1")
                    timeout_seconds = 10
                })
            }
            [pscustomobject][ordered]@{
                id = "preflight-neutral"
                prerequisites = @()
                environment = @()
                blocked_environment = @("DATABASE_URL")
                topology = [pscustomobject][ordered]@{
                    mode = "none"
                    provider = "none"
                    on_success = "destroy"
                }
                actions = @([pscustomobject][ordered]@{
                    id = "preflight-neutral-assertion"
                    stage = "assertion"
                    program = "pwsh"
                    arguments = @(
                        "-NoProfile", "-NonInteractive", "-File", '${repository_root}/harness/neutral.ps1'
                    )
                    input_paths = @("harness/neutral.ps1")
                    timeout_seconds = 10
                })
            }
            [pscustomobject][ordered]@{
                id = "sit-neutral"
                prerequisites = @()
                environment = @()
                blocked_environment = @("DATABASE_URL")
                topology = [pscustomobject][ordered]@{
                    mode = "none"
                    provider = "none"
                    on_success = "destroy"
                }
                actions = @([pscustomobject][ordered]@{
                    id = "sit-neutral-assertion"
                    stage = "assertion"
                    program = "pwsh"
                    arguments = @(
                        "-NoProfile", "-NonInteractive", "-File", '${repository_root}/harness/neutral.ps1'
                    )
                    input_paths = @("harness/neutral.ps1")
                    timeout_seconds = 10
                })
            }
        )
    }
    $plannerAdapter | Add-Member -NotePropertyName execution_policy `
        -NotePropertyValue ([pscustomobject][ordered]@{
            max_parallel_lanes = 2
            max_attempts_per_lane = 32
            max_evidence_bytes = 268435456
        })
    $plannerTargetByLane = @{
        "no-topology" = "owner-target"
        "local-success" = "independent-target"
        "dependent-no-topology" = "dependent-target"
        "rehearsal-neutral" = "neutral-target"
        "preflight-neutral" = "neutral-target"
        "sit-neutral" = "neutral-target"
    }
    foreach ($plannerLane in $plannerAdapter.lanes) {
        $plannerLane | Add-Member -NotePropertyName deadline_seconds -NotePropertyValue 60
        foreach ($plannerAction in $plannerLane.actions) {
            $plannerAction | Add-Member -NotePropertyName implementation_target `
                -NotePropertyValue ([string]$plannerTargetByLane[[string]$plannerLane.id])
            $plannerAction | Add-Member -NotePropertyName proof_classes `
                -NotePropertyValue @("runner-selftest")
            $plannerAction | Add-Member -NotePropertyName tools -NotePropertyValue @()
        }
    }
    foreach ($plannerTarget in $plannerContract.implementation_targets) {
        $representativeLane = @($plannerAdapter.lanes | Where-Object {
            [string]$plannerTargetByLane[[string]$_.id] -ceq [string]$plannerTarget.id
        })[0]
        $representativeAction = @($representativeLane.actions)[0]
        $plannerTarget.command = [pscustomobject][ordered]@{
            program = [string]$representativeAction.program
            arguments = @($representativeAction.arguments)
            input_paths = @($representativeAction.input_paths)
            tools = @()
        }
    }
    $plannerTextInputs = [ordered]@{
        ".gitignore" = "/artifacts/evidence/`n"
        "acceptance/no-topology.json" = "{`"expected`":`"owner`"}`n"
        "acceptance/local-success.json" = "{`"expected`":`"independent`"}`n"
        "acceptance/dependent-no-topology.json" = "{`"expected`":`"dependent`"}`n"
        "acceptance/neutral.json" = "{`"expected`":`"neutral`"}`n"
        "fixtures/no-topology.txt" = "owner fixture`n"
        "harness/no-topology.ps1" = "exit 0`n"
        "harness/owner-config.txt" = "owner config`n"
        "harness/owner-response.rsp" = "owner response`n"
        "harness/local-success.ps1" = "exit 0`n"
        "harness/dependent-no-topology.ps1" = "exit 0`n"
        "harness/neutral.ps1" = "exit 0`n"
        "domain/owner.txt" = "owner domain`n"
        "domain/independent.txt" = "independent domain`n"
        "domain/dependent.txt" = "dependent domain`n"
        "domain/neutral.txt" = "neutral domain`n"
    }
    $resetPlannerRepository = {
        Write-JsonFile -Path (Join-Path $plannerRepository "contract.json") `
            -Document $plannerContract
        Write-JsonFile -Path (Join-Path $plannerRepository "adapter.json") `
            -Document $plannerAdapter
        foreach ($entry in $plannerTextInputs.GetEnumerator()) {
            [IO.File]::WriteAllText(
                (Join-Path $plannerRepository ([string]$entry.Key)),
                [string]$entry.Value,
                [Text.UTF8Encoding]::new($false)
            )
        }
    }
    & $resetPlannerRepository
    & git -C $plannerRepository add --all
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to stage the disposable planner repository."
    }
    & git -C $plannerRepository -c user.name=tessara-validation `
        -c user.email=validation@invalid commit --quiet -m baseline
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to commit the disposable planner repository baseline."
    }
    $plannerAdapterPath = Join-Path $plannerRepository "adapter.json"
    $plannerEvidenceRoot = Join-Path $plannerRepository "artifacts/evidence"
    $plannerBaseline = Invoke-CompatibilityPlannerSet -AdapterPath $plannerAdapterPath `
        -RepositoryRoot $plannerRepository -EvidenceRoot (Join-Path $plannerEvidenceRoot "baseline") `
        -CandidateFingerprint $candidate
    $plannerMutationCases = @(
        [pscustomobject][ordered]@{
            id = "contract-slice"
            apply = {
                $mutated = Copy-JsonValue $plannerContract
                $ownerRequirement = @($mutated.requirements | Where-Object {
                    [string]$_.id -ceq "owner-requirement"
                })[0]
                $ownerRequirement.id = "owner-requirement-mutated"
                Write-JsonFile -Path (Join-Path $plannerRepository "contract.json") `
                    -Document $mutated
            }
        }
        [pscustomobject][ordered]@{
            id = "acceptance"
            apply = {
                [IO.File]::WriteAllText(
                    (Join-Path $plannerRepository "acceptance/no-topology.json"),
                    "{`"expected`":`"owner-mutated`"}`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        [pscustomobject][ordered]@{
            id = "fixture"
            apply = {
                [IO.File]::WriteAllText(
                    (Join-Path $plannerRepository "fixtures/no-topology.txt"),
                    "owner fixture mutated`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        [pscustomobject][ordered]@{
            id = "harness"
            apply = {
                [IO.File]::WriteAllText(
                    (Join-Path $plannerRepository "harness/no-topology.ps1"),
                    "# mutated owner harness`nexit 0`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        [pscustomobject][ordered]@{
            id = "embedded-config-input"
            apply = {
                [IO.File]::WriteAllText(
                    (Join-Path $plannerRepository "harness/owner-config.txt"),
                    "owner config mutated`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        [pscustomobject][ordered]@{
            id = "response-file-input"
            apply = {
                [IO.File]::WriteAllText(
                    (Join-Path $plannerRepository "harness/owner-response.rsp"),
                    "owner response mutated`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
        [pscustomobject][ordered]@{
            id = "lane-adapter"
            apply = {
                $mutated = Copy-JsonValue $plannerAdapter
                $ownerLane = @($mutated.lanes | Where-Object {
                    [string]$_.id -ceq "no-topology"
                })[0]
                $ownerLane.blocked_environment += "TESSARA_VP_MUTATION_SENTINEL"
                Write-JsonFile -Path (Join-Path $plannerRepository "adapter.json") `
                    -Document $mutated
            }
        }
        [pscustomobject][ordered]@{
            id = "dependency-domain"
            apply = {
                [IO.File]::WriteAllText(
                    (Join-Path $plannerRepository "domain/owner.txt"),
                    "owner domain mutated`n",
                    [Text.UTF8Encoding]::new($false)
                )
            }
        }
    )
    foreach ($mutation in $plannerMutationCases) {
        & $resetPlannerRepository
        & $mutation.apply
        $mutatedPlannerSet = Invoke-CompatibilityPlannerSet `
            -AdapterPath $plannerAdapterPath -RepositoryRoot $plannerRepository `
            -EvidenceRoot (Join-Path $plannerEvidenceRoot ([string]$mutation.id)) `
            -CandidateFingerprint $candidate
        Assert-ScopedPlannerMutation -Baseline $plannerBaseline -Mutated $mutatedPlannerSet `
            -Label "Planner $($mutation.id) mutation"
    }

    & $resetPlannerRepository
    $backslashContract = Copy-JsonValue $plannerContract
    @($backslashContract.dependency_domains | Where-Object {
        [string]$_.name -ceq "owner-domain"
    })[0].tracked_inputs = @(' domain\owner.txt ')
    Write-JsonFile -Path (Join-Path $plannerRepository "contract.json") `
        -Document $backslashContract
    $backslashBaselinePlan = Invoke-CompatibilityPlannerSet `
        -AdapterPath $plannerAdapterPath -RepositoryRoot $plannerRepository `
        -EvidenceRoot (Join-Path $plannerEvidenceRoot "backslash-baseline") `
        -CandidateFingerprint $candidate
    [IO.File]::WriteAllText(
        (Join-Path $plannerRepository "domain/owner.txt"),
        "backslash-pattern mutation`n",
        [Text.UTF8Encoding]::new($false)
    )
    $backslashChangedPlan = Invoke-CompatibilityPlannerSet `
        -AdapterPath $plannerAdapterPath -RepositoryRoot $plannerRepository `
        -EvidenceRoot (Join-Path $plannerEvidenceRoot "backslash-changed") `
        -CandidateFingerprint $candidate
    Assert-ScopedPlannerMutation -Baseline $backslashBaselinePlan `
        -Mutated $backslashChangedPlan -Label "Backslash dependency-pattern mutation"

    & $resetPlannerRepository
    & git -C $plannerRepository add -- "domain/owner.txt"
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to stage the disposable cached-deletion fixture."
    }
    Remove-Item -LiteralPath (Join-Path $plannerRepository "domain/owner.txt") -Force
    $cachedDeletionPlan = Invoke-CompatibilityPlannerSet `
        -AdapterPath $plannerAdapterPath -RepositoryRoot $plannerRepository `
        -EvidenceRoot (Join-Path $plannerEvidenceRoot "cached-regular-deletion") `
        -CandidateFingerprint $candidate
    if ([string]$cachedDeletionPlan.owner -ceq [string]$plannerBaseline.owner) {
        throw "A cached deleted ordinary file did not invalidate its owner lane."
    }
    & git -C $plannerRepository update-index --force-remove -- "domain/owner.txt"
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to clear the disposable cached-deletion fixture."
    }

    & $resetPlannerRepository
    $linkBlob = (& git -C $plannerRepository hash-object -w -- "domain/owner.txt").Trim()
    if ($LASTEXITCODE -ne 0 -or $linkBlob -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "Unable to create the disposable tracked-link blob."
    }
    & git -C $plannerRepository update-index --add --cacheinfo `
        "120000,$linkBlob,domain/owner.txt"
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to create the disposable tracked-link index entry."
    }
    try {
        Assert-Rejected -Action {
            Invoke-CompatibilityPlannerSet -AdapterPath $plannerAdapterPath `
                -RepositoryRoot $plannerRepository `
                -EvidenceRoot (Join-Path $plannerEvidenceRoot "tracked-link") `
                -CandidateFingerprint $candidate | Out-Null
        } -ExpectedMessage "unsupported tracked entry mode '120000'"
    } finally {
        & git -C $plannerRepository update-index --force-remove -- "domain/owner.txt"
        if ($LASTEXITCODE -ne 0) {
            throw "Unable to clear the disposable tracked-link index entry."
        }
    }

    & $resetPlannerRepository
    $unicodeRelativePath = "domain/naïve.txt"
    $unicodeFullPath = Join-Path $plannerRepository $unicodeRelativePath
    [IO.File]::WriteAllText(
        $unicodeFullPath, "unicode baseline`n", [Text.UTF8Encoding]::new($false)
    )
    $unicodeContract = Copy-JsonValue $plannerContract
    @($unicodeContract.dependency_domains | Where-Object {
        [string]$_.name -ceq "owner-domain"
    })[0].tracked_inputs += $unicodeRelativePath
    Write-JsonFile -Path (Join-Path $plannerRepository "contract.json") `
        -Document $unicodeContract
    $unicodeBaselinePlan = Invoke-CompatibilityPlannerSet `
        -AdapterPath $plannerAdapterPath -RepositoryRoot $plannerRepository `
        -EvidenceRoot (Join-Path $plannerEvidenceRoot "unicode-baseline") `
        -CandidateFingerprint $candidate
    [IO.File]::WriteAllText(
        $unicodeFullPath, "unicode changed`n", [Text.UTF8Encoding]::new($false)
    )
    $unicodeChangedPlan = Invoke-CompatibilityPlannerSet `
        -AdapterPath $plannerAdapterPath -RepositoryRoot $plannerRepository `
        -EvidenceRoot (Join-Path $plannerEvidenceRoot "unicode-changed") `
        -CandidateFingerprint $candidate
    if ([string]$unicodeBaselinePlan.owner -ceq
        [string]$unicodeChangedPlan.owner) {
        throw "A non-ASCII Git pathname content change did not invalidate its owner lane."
    }

    $missingSourceAdapter = Copy-JsonValue $adapter
    $missingSourceLane = @($missingSourceAdapter.lanes | Where-Object { $_.id -ceq "no-topology" })[0]
    $missingSourceLane.environment += [pscustomobject][ordered]@{
        name = "TESSARA_VP_REQUIRED"
        source = "TESSARA_VP_REQUIRED_SOURCE"
    }
    $missingSourcePath = Join-Path $root "missing-source-adapter.json"
    Write-JsonFile -Path $missingSourcePath -Document $missingSourceAdapter
    $missingSourceRoot = Join-Path $root "missing-source"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $missingSourcePath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $missingSourceRoot | Out-Null
    } -ExpectedMessage "Required source environment variable"
    $missingSourceResult = Get-LaneResult -EvidenceRoot $missingSourceRoot `
        -LaneId "no-topology"
    Assert-Equal $missingSourceResult.failure_stage "setup" "Missing-source failure stage"
    Assert-Equal $missingSourceResult.assertions_started $false `
        "Missing-source assertions-started state"
    Assert-EvidencePairs -AttemptRoot (Split-Path -Parent (
        @(Get-ChildItem -LiteralPath (Join-Path $missingSourceRoot "lanes/no-topology/attempts") `
            -Filter "lane-result.json" -File -Recurse | Select-Object -Last 1).FullName
    ))

    [Environment]::SetEnvironmentVariable(
        "TESSARA_VP_WHITESPACE_SOURCE", " `t ", "Process"
    )
    $whitespaceSourceAdapter = Copy-JsonValue $adapter
    $whitespaceSourceLane = @($whitespaceSourceAdapter.lanes | Where-Object {
        $_.id -ceq "local-success"
    })[0]
    $whitespaceSourceLane.environment += [pscustomobject][ordered]@{
        name = "TESSARA_VP_WHITESPACE"
        source = "TESSARA_VP_WHITESPACE_SOURCE"
    }
    $whitespaceSourcePath = Join-Path $root "whitespace-source-adapter.json"
    Write-JsonFile -Path $whitespaceSourcePath -Document $whitespaceSourceAdapter
    $whitespaceSourceRoot = Join-Path $root "whitespace-source"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $whitespaceSourcePath `
            -LaneId "local-success" -CandidateFingerprint $candidate `
            -EvidenceRoot $whitespaceSourceRoot | Out-Null
    } -ExpectedMessage "Source environment variable 'TESSARA_VP_WHITESPACE_SOURCE' is empty"
    $whitespaceSourceResult = Get-LaneResult -EvidenceRoot $whitespaceSourceRoot `
        -LaneId "local-success"
    Assert-Equal $whitespaceSourceResult.failure_stage "setup" `
        "Whitespace-source failure stage"
    Assert-Equal $whitespaceSourceResult.assertions_started $false `
        "Whitespace-source assertions-started state"
    Assert-Equal @($whitespaceSourceResult.actions).Count 0 `
        "Whitespace-source action count"
    if (@(Get-ChildItem -LiteralPath $whitespaceSourceRoot `
            -Filter "topology-ownership-acquired.json" -File -Recurse).Count -ne 0) {
        throw "Whitespace-source setup failure acquired topology ownership."
    }
    if (-not (Test-PortClosed -Port ([int]$whitespaceSourceResult.ports.http))) {
        throw "Whitespace-source setup failure retained its leased topology port."
    }

    $secretValue = "tessara-vp-secret-$([guid]::NewGuid().ToString('N'))"
    [Environment]::SetEnvironmentVariable("TESSARA_VP_SECRET_SOURCE", $secretValue, "Process")
    $secretAdapter = Copy-JsonValue $adapter
    $secretLane = @($secretAdapter.lanes | Where-Object { $_.id -ceq "no-topology" })[0]
    $secretLane.environment += [pscustomobject][ordered]@{
        name = "TESSARA_VP_SECRET"
        source = "TESSARA_VP_SECRET_SOURCE"
    }
    $secretLane.actions[0].arguments[-1] = "echo-secret"
    $secretPath = Join-Path $root "secret-source-adapter.json"
    Write-TestAdapterWithMatchingContract -Path $secretPath `
        -Document $secretAdapter -RepositoryRoot $repoRoot
    $secretRoot = Join-Path $root "secret-source"
    $secretResult = Invoke-TessaraValidationLane -AdapterPath $secretPath `
        -LaneId "no-topology" -CandidateFingerprint $candidate -EvidenceRoot $secretRoot
    Assert-Equal $secretResult.state "passed" "Secret-source lane state"
    $secretAttemptRoot = Split-Path -Parent $secretResult.evidence_path
    foreach ($streamName in @("stdout", "stderr")) {
        $streamText = Get-Content -Raw -LiteralPath (
            Join-Path $secretAttemptRoot "actions/no-topology-check.$streamName.log"
        )
        if (-not $streamText.Contains("[REDACTED:TESSARA_SOURCE_ENV]")) {
            throw "Source-secret $streamName evidence did not prove active platform redaction."
        }
    }
    foreach ($file in @(Get-ChildItem -LiteralPath $secretRoot -File -Recurse)) {
        if ([IO.File]::ReadAllText($file.FullName).IndexOf(
                $secretValue, [StringComparison]::Ordinal
            ) -ge 0) {
            throw "A retained validation artifact disclosed the unique source secret."
        }
    }
    Assert-EvidencePairs -AttemptRoot $secretAttemptRoot

    $shortSecretValue = "Q~"
    [Environment]::SetEnvironmentVariable(
        "TESSARA_VP_SECRET_SOURCE", $shortSecretValue, "Process"
    )
    $noisySecretAdapter = Copy-JsonValue $adapter
    $noisySecretLane = @($noisySecretAdapter.lanes | Where-Object {
        $_.id -ceq "no-topology"
    })[0]
    $noisySecretLane.environment += [pscustomobject][ordered]@{
        name = "TESSARA_VP_SECRET"
        source = "TESSARA_VP_SECRET_SOURCE"
    }
    $noisySecretLane.actions[0].arguments[-1] = "noisy-secret"
    $noisySecretPath = Join-Path $root "noisy-secret-source-adapter.json"
    Write-TestAdapterWithMatchingContract -Path $noisySecretPath `
        -Document $noisySecretAdapter -RepositoryRoot $repoRoot
    $noisySecretRoot = Join-Path $root "noisy-secret-source"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $noisySecretPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $noisySecretRoot | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $noisySecretResult = Get-LaneResult -EvidenceRoot $noisySecretRoot `
        -LaneId "no-topology"
    Assert-Equal $noisySecretResult.actions[0].terminal "output_overflow" `
        "Noisy secret action terminal"
    Assert-Equal $noisySecretResult.actions[0].stdout_overflow $true `
        "Noisy secret stdout overflow"
    Assert-Equal $noisySecretResult.actions[0].stderr_overflow $true `
        "Noisy secret stderr overflow"
    Assert-Equal ([long]$noisySecretResult.actions[0].output_limit_bytes) 1048576 `
        "Noisy secret output byte limit"
    if ([string]$noisySecretResult.failure.message -notlike "*bounded output limit*") {
        throw "Noisy secret action did not fail at the bounded output boundary."
    }
    $noisySecretAttemptRoot = Split-Path -Parent $noisySecretResult.evidence_path
    foreach ($streamName in @("stdout", "stderr")) {
        $streamPath = Join-Path $noisySecretAttemptRoot `
            "actions/no-topology-check.$streamName.log"
        $streamEntry = Get-Item -LiteralPath $streamPath
        if ([long]$streamEntry.Length -gt 1048576) {
            throw "Post-redaction $streamName evidence exceeded the 1-MiB durable limit."
        }
        $streamText = Get-Content -Raw -LiteralPath $streamPath
        if (-not $streamText.Contains("[REDACTED:TESSARA_SOURCE_ENV]") -or
            -not $streamText.Contains("[TESSARA_REDACTED_OUTPUT_TRUNCATED]") -or
            $streamText.IndexOf($shortSecretValue, [StringComparison]::Ordinal) -ge 0) {
            throw "Bounded $streamName evidence did not completely redact the short secret."
        }
    }
    foreach ($file in @(Get-ChildItem -LiteralPath $noisySecretRoot -File -Recurse)) {
        if ([IO.File]::ReadAllText($file.FullName).IndexOf(
                $shortSecretValue, [StringComparison]::Ordinal
            ) -ge 0) {
            throw "A retained noisy-output artifact disclosed the short source secret."
        }
    }
    Assert-EvidencePairs -AttemptRoot $noisySecretAttemptRoot

    $invocationInputPath = Join-Path $root "embedded-command-input.txt"
    $invocationMutatorPath = Join-Path $root "embedded-input-mutator.ps1"
    $invocationVerifierPath = Join-Path $root "embedded-input-verifier.ps1"
    $invocationMarkerPath = Join-Path $root "embedded-input-verifier.invoked"
    [IO.File]::WriteAllText(
        $invocationInputPath, "planned input`n", [Text.UTF8Encoding]::new($false)
    )
    [IO.File]::WriteAllText(
        $invocationMutatorPath,
        @'
$configArgument = [string]$args[0]
if (-not $configArgument.StartsWith('--config=', [StringComparison]::Ordinal)) { exit 31 }
[IO.File]::AppendAllText($configArgument.Substring(9), "changed-during-execution`n")
'@,
        [Text.UTF8Encoding]::new($false)
    )
    [IO.File]::WriteAllText(
        $invocationVerifierPath,
        @'
if (-not ([string]$args[0]).StartsWith('@', [StringComparison]::Ordinal)) { exit 32 }
[IO.File]::WriteAllText([string]$args[1], "invoked`n")
'@,
        [Text.UTF8Encoding]::new($false)
    )
    $invocationInputRelative = [IO.Path]::GetRelativePath(
        $repoRoot, $invocationInputPath
    ).Replace('\', '/')
    $invocationMutatorRelative = [IO.Path]::GetRelativePath(
        $repoRoot, $invocationMutatorPath
    ).Replace('\', '/')
    $invocationVerifierRelative = [IO.Path]::GetRelativePath(
        $repoRoot, $invocationVerifierPath
    ).Replace('\', '/')
    $invocationMarkerRelative = [IO.Path]::GetRelativePath(
        $repoRoot, $invocationMarkerPath
    ).Replace('\', '/')
    $invocationAdapter = Copy-JsonValue $adapter
    $invocationAdapter.execution_inputs += @(
        [pscustomobject][ordered]@{
            path = $invocationInputRelative
            lanes = @("no-topology")
        }
        [pscustomobject][ordered]@{
            path = $invocationMutatorRelative
            lanes = @("no-topology")
        }
        [pscustomobject][ordered]@{
            path = $invocationVerifierRelative
            lanes = @("no-topology")
        }
    )
    $invocationLane = @($invocationAdapter.lanes | Where-Object {
        [string]$_.id -ceq "no-topology"
    })[0]
    $invocationLane.actions = @(
        [pscustomobject][ordered]@{
            id = "mutate-embedded-input"
            stage = "setup"
            program = "pwsh"
            arguments = @(
                "-NoProfile", "-NonInteractive", "-File", "`${repository_root}/$invocationMutatorRelative",
                "--config=$invocationInputRelative"
            )
            input_paths = @($invocationMutatorRelative, $invocationInputRelative)
            timeout_seconds = 10
            tools = @()
        }
        (Copy-JsonValue (@($adapter.lanes | Where-Object {
            [string]$_.id -ceq "no-topology"
        })[0].actions[0]))
    )
    $invocationAdapterPath = Join-Path $root "embedded-command-input-adapter.json"
    Write-JsonFile -Path $invocationAdapterPath -Document $invocationAdapter
    $invocationEvidenceRoot = Join-Path $root "embedded-command-input"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $invocationAdapterPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $invocationEvidenceRoot | Out-Null
    } -ExpectedMessage "checkpoint was revoked and the lane must rerun"
    $invocationCheckpointPath = @(
        Get-ChildItem -LiteralPath (Join-Path $invocationEvidenceRoot `
            "lanes/no-topology/attempts") -Filter "execution-complete.json" `
            -File -Recurse | Sort-Object FullName | Select-Object -Last 1
    )[0].FullName
    $invocationAttemptRoot = Split-Path -Parent $invocationCheckpointPath
    $invocationCheckpoint = Get-Content -Raw -LiteralPath $invocationCheckpointPath |
        ConvertFrom-Json -Depth 100
    $invocationResult = $invocationCheckpoint.result
    Assert-Equal @($invocationResult.actions).Count 1 `
        "Mutated embedded input completed-action count"
    if ((Test-Path -LiteralPath $invocationMarkerPath) -or
        [string]$invocationResult.failure.message -notlike
            "*Lane-owned input*changed during execution*" -or
        -not [bool]$invocationResult.source_integrity_failure -or
        -not (Test-Path -LiteralPath (Join-Path $invocationAttemptRoot `
                "execution-revoked.json") -PathType Leaf) -or
        (Test-Path -LiteralPath (Join-Path $invocationAttemptRoot "lane-result.json")) -or
        (Test-Path -LiteralPath (Join-Path $invocationAttemptRoot "attempt-index.json"))) {
        throw "An embedded declared input mutation was not rejected before the next command."
    }

    $driftInputPath = Join-Path $root "execution-drift-input.txt"
    [IO.File]::WriteAllText($driftInputPath, "planned`n", [Text.UTF8Encoding]::new($false))
    $driftInputRelative = [IO.Path]::GetRelativePath($repoRoot, $driftInputPath).Replace('\', '/')
    $driftAdapter = Copy-JsonValue $adapter
    $driftAdapter.execution_inputs += [pscustomobject][ordered]@{
        path = $driftInputRelative
        lanes = @("no-topology")
    }
    $driftLane = @($driftAdapter.lanes | Where-Object { $_.id -ceq "no-topology" })[0]
    $driftLane.actions[0].arguments = @(
        "-NoProfile", "-NonInteractive", "-File",
        '${repository_root}/scripts/validation-platform/fixtures/synthetic-action.ps1',
        "-Mode", "mutate-file", "-Path", "`${repository_root}/$driftInputRelative"
    )
    $driftLane.actions[0].input_paths = @(
        "scripts/validation-platform/fixtures/synthetic-action.ps1",
        $driftInputRelative
    )
    $driftAdapterPath = Join-Path $root "execution-drift-adapter.json"
    Write-TestAdapterWithMatchingContract -Path $driftAdapterPath `
        -Document $driftAdapter -RepositoryRoot $repoRoot
    $driftEvidenceRoot = Join-Path $root "execution-drift"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $driftAdapterPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $driftEvidenceRoot | Out-Null
    } -ExpectedMessage "checkpoint was revoked and the lane must rerun"
    $driftCheckpointPath = @(
        Get-ChildItem -LiteralPath (Join-Path $driftEvidenceRoot `
            "lanes/no-topology/attempts") -Filter "execution-complete.json" `
            -File -Recurse | Sort-Object FullName | Select-Object -Last 1
    )[0].FullName
    $driftAttemptRoot = Split-Path -Parent $driftCheckpointPath
    $driftCheckpoint = Get-Content -Raw -LiteralPath $driftCheckpointPath |
        ConvertFrom-Json -Depth 100
    $driftResult = $driftCheckpoint.result
    Assert-Equal $driftResult.failure_stage "source-integrity" `
        "Changed execution-input failure stage"
    Assert-Equal $driftResult.assertions_completed $true `
        "Changed execution-input assertion completion"
    if ([string]::IsNullOrWhiteSpace([string]$driftResult.source_integrity_failure)) {
        throw "Changed execution input did not retain its integrity failure."
    }
    $driftRevocation = Get-Content -Raw -LiteralPath (
        Join-Path $driftAttemptRoot "execution-revoked.json"
    ) | ConvertFrom-Json -Depth 20
    Assert-Equal $driftRevocation.state "revoked" "Changed execution-input revocation state"
    Assert-Equal $driftRevocation.reason "source-integrity-changed-after-checkpoint" `
        "Changed execution-input revocation reason"
    if ((Test-Path -LiteralPath (Join-Path $driftAttemptRoot "lane-result.json")) -or
        (Test-Path -LiteralPath (Join-Path $driftAttemptRoot "attempt-index.json"))) {
        throw "A revoked execution-input checkpoint published authoritative evidence."
    }
    Assert-EvidencePairs -AttemptRoot $driftAttemptRoot

    # A negative revocation is diagnostic; eligibility depends on the positive
    # integrity commit. Even when revocation publication itself fails and the
    # mutated input is later restored, a raw checkpoint that recorded source
    # drift must remain permanently ineligible for finalization.
    [IO.File]::WriteAllText(
        $driftInputPath, "planned`n", [Text.UTF8Encoding]::new($false)
    )
    $revocationFailureRoot = Join-Path $root "execution-drift-revocation-failure"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $driftAdapterPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $revocationFailureRoot `
            -CertificationFault "execution-revocation-publication" | Out-Null
    } -ExpectedMessage "Revocation publication failed: Injected execution-revocation publication failure."
    $revocationFailureCheckpointPath = @(
        Get-ChildItem -LiteralPath (Join-Path $revocationFailureRoot `
            "lanes/no-topology/attempts") -Filter "execution-complete.json" `
            -File -Recurse | Sort-Object FullName | Select-Object -Last 1
    )[0].FullName
    $revocationFailureAttemptRoot = Split-Path -Parent $revocationFailureCheckpointPath
    foreach ($forbiddenPublication in @(
            "execution-revoked.json", "execution-revoked.json.sha256",
            "execution-integrity-verified.json",
            "execution-integrity-verified.json.sha256",
            "lane-result.json", "lane-result.json.sha256",
            "attempt-index.json", "attempt-index.json.sha256"
        )) {
        if (Test-Path -LiteralPath (Join-Path $revocationFailureAttemptRoot `
                $forbiddenPublication)) {
            throw "Failed revocation published ineligible evidence '$forbiddenPublication'."
        }
    }
    [IO.File]::WriteAllText(
        $driftInputPath, "planned`n", [Text.UTF8Encoding]::new($false)
    )
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $driftAdapterPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $revocationFailureRoot `
            -FinalizeCheckpointPath $revocationFailureCheckpointPath | Out-Null
    } -ExpectedMessage "cannot receive a positive integrity commit"
    if ((Test-Path -LiteralPath (Join-Path $revocationFailureAttemptRoot `
            "lane-result.json")) -or
        (Test-Path -LiteralPath (Join-Path $revocationFailureAttemptRoot `
            "attempt-index.json"))) {
        throw "A checkpoint with failed revocation publication was promoted after input restoration."
    }

    $lateDriftAdapter = Copy-JsonValue $driftAdapter
    $lateDriftLane = @($lateDriftAdapter.lanes | Where-Object {
        [string]$_.id -ceq "no-topology"
    })[0]
    $lateDriftLane.actions[0].arguments = @(
        "-NoProfile", "-NonInteractive", "-File",
        '${repository_root}/scripts/validation-platform/fixtures/synthetic-action.ps1',
        "-Mode", "success"
    )
    $lateDriftLane.actions[0].input_paths = @(
        "scripts/validation-platform/fixtures/synthetic-action.ps1"
    )
    $lateDriftAdapterPath = Join-Path $root "late-execution-drift-adapter.json"
    Write-TestAdapterWithMatchingContract -Path $lateDriftAdapterPath `
        -Document $lateDriftAdapter -RepositoryRoot $repoRoot
    [IO.File]::WriteAllText(
        $driftInputPath, "planned`n", [Text.UTF8Encoding]::new($false)
    )
    $postCheckpointIntegrityLines = @(Select-String `
        -LiteralPath $lifecycleModulePath -SimpleMatch `
        'Assert-TessaraLifecycleLaneInputsUnchanged -LaneIdentity $LaneIdentity')
    if ($postCheckpointIntegrityLines.Count -lt 2) {
        throw "Could not identify the post-checkpoint source-integrity boundary."
    }
    $postCheckpointIntegrityLine = @(
        $postCheckpointIntegrityLines | Sort-Object LineNumber | Select-Object -Last 1
    )[0]
    $lateDriftRoot = Join-Path $root "late-execution-drift-revocation-failure"
    $global:TessaraCertificationLateDriftPath = $driftInputPath
    $lateDriftBreakpoint = Set-PSBreakpoint -Script $lifecycleModulePath `
        -Line $postCheckpointIntegrityLine.LineNumber -Action {
            [IO.File]::WriteAllText(
                $global:TessaraCertificationLateDriftPath,
                "changed-after-checkpoint`n",
                [Text.UTF8Encoding]::new($false)
            )
        }
    try {
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $lateDriftAdapterPath `
                -LaneId "no-topology" -CandidateFingerprint $candidate `
                -EvidenceRoot $lateDriftRoot `
                -CertificationFault "execution-revocation-publication" | Out-Null
        } -ExpectedMessage "Revocation publication failed: Injected execution-revocation publication failure."
    } finally {
        Remove-PSBreakpoint -Breakpoint $lateDriftBreakpoint
        Remove-Variable -Name TessaraCertificationLateDriftPath -Scope Global `
            -ErrorAction SilentlyContinue
    }
    $lateDriftCheckpointPath = @(
        Get-ChildItem -LiteralPath (Join-Path $lateDriftRoot `
            "lanes/no-topology/attempts") -Filter "execution-complete.json" `
            -File -Recurse | Sort-Object FullName | Select-Object -Last 1
    )[0].FullName
    $lateDriftAttemptRoot = Split-Path -Parent $lateDriftCheckpointPath
    $lateDriftCheckpoint = Get-Content -Raw -LiteralPath $lateDriftCheckpointPath |
        ConvertFrom-Json -Depth 100
    if ($null -ne $lateDriftCheckpoint.result.source_integrity_failure) {
        throw "Late post-checkpoint drift was incorrectly recorded as pre-checkpoint source failure."
    }
    foreach ($forbiddenPublication in @(
            "execution-revoked.json", "execution-revoked.json.sha256",
            "execution-integrity-verified.json",
            "execution-integrity-verified.json.sha256",
            "lane-result.json", "lane-result.json.sha256",
            "attempt-index.json", "attempt-index.json.sha256"
        )) {
        if (Test-Path -LiteralPath (Join-Path $lateDriftAttemptRoot `
                $forbiddenPublication)) {
            throw "Late drift with failed revocation published '$forbiddenPublication'."
        }
    }
    [IO.File]::WriteAllText(
        $driftInputPath, "planned`n", [Text.UTF8Encoding]::new($false)
    )
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $lateDriftAdapterPath `
            -LaneId "no-topology" -CandidateFingerprint $candidate `
            -EvidenceRoot $lateDriftRoot `
            -FinalizeCheckpointPath $lateDriftCheckpointPath | Out-Null
    } -ExpectedMessage "finalization-only recovery cannot create it"
    if ((Test-Path -LiteralPath (Join-Path $lateDriftAttemptRoot `
            "execution-integrity-verified.json")) -or
        (Test-Path -LiteralPath (Join-Path $lateDriftAttemptRoot `
            "lane-result.json")) -or
        (Test-Path -LiteralPath (Join-Path $lateDriftAttemptRoot `
            "attempt-index.json"))) {
        throw "Late drift was promoted after its source was restored."
    }

    $failureAdapter = Copy-JsonValue $adapter
    $failureLane = @($failureAdapter.lanes | Where-Object { $_.id -ceq "local-success" })[0]
    @($failureAdapter.execution_inputs | Where-Object {
        $_.path -ceq "scripts/validation-platform/fixtures/synthetic-action.ps1"
    })[0].lanes += "local-success"
    $failureLane.actions[0].arguments = @(
        "-NoProfile", "-NonInteractive", "-File",
        '${repository_root}/scripts/validation-platform/fixtures/synthetic-action.ps1',
        "-Mode", "fail", "-ExitCode", "17"
    )
    $failureLane.actions[0].input_paths = @(
        "scripts/validation-platform/fixtures/synthetic-action.ps1"
    )
    $failurePath = Join-Path $root "failure-adapter.json"
    Write-TestAdapterWithMatchingContract -Path $failurePath `
        -Document $failureAdapter -RepositoryRoot $repoRoot
    $failureRoot = Join-Path $root "failure"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $failurePath -LaneId "local-success" `
            -CandidateFingerprint $candidate -EvidenceRoot $failureRoot | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $failureResult = Get-LaneResult -EvidenceRoot $failureRoot -LaneId "local-success"
    Assert-Equal $failureResult.actions[0].exit_code 17 "Failed action exit code"
    Assert-Equal $failureResult.cleanup_restoration.state "passed" "Failed-lane cleanup"
    Assert-Equal $failureResult.failure_stage "assertion" "Failed action stage"
    Assert-Equal $failureResult.assertions_started $true "Failed action assertions-started state"
    Assert-EvidencePairs -AttemptRoot (Split-Path -Parent (
        @(Get-ChildItem -LiteralPath (Join-Path $failureRoot "lanes/local-success/attempts") `
            -Filter "lane-result.json" -File -Recurse | Select-Object -Last 1).FullName
    ))

    $timeoutAdapter = Copy-JsonValue $adapter
    $timeoutLane = @($timeoutAdapter.lanes | Where-Object { $_.id -ceq "local-success" })[0]
    @($timeoutAdapter.execution_inputs | Where-Object {
        $_.path -ceq "scripts/validation-platform/fixtures/synthetic-action.ps1"
    })[0].lanes += "local-success"
    $timeoutLane.actions[0].arguments = @(
        "-NoProfile", "-NonInteractive", "-File",
        '${repository_root}/scripts/validation-platform/fixtures/synthetic-action.ps1',
        "-Mode", "wait", "-Seconds", "30"
    )
    $timeoutLane.actions[0].input_paths = @(
        "scripts/validation-platform/fixtures/synthetic-action.ps1"
    )
    $timeoutLane.actions[0].timeout_seconds = 1
    $timeoutPath = Join-Path $root "timeout-adapter.json"
    Write-TestAdapterWithMatchingContract -Path $timeoutPath `
        -Document $timeoutAdapter -RepositoryRoot $repoRoot
    $timeoutRoot = Join-Path $root "timeout"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $timeoutPath -LaneId "local-success" `
            -CandidateFingerprint $candidate -EvidenceRoot $timeoutRoot | Out-Null
    } -ExpectedMessage "timed out"
    $timeoutResult = Get-LaneResult -EvidenceRoot $timeoutRoot -LaneId "local-success"
    Assert-Equal $timeoutResult.actions[0].terminal "timed_out" "Timed-out action terminal"
    Assert-Equal $timeoutResult.cleanup_restoration.state "passed" "Timed-out lane cleanup"
    Assert-Equal $timeoutResult.failure_stage "assertion" "Timed-out action stage"

    $interruptRoot = Join-Path $root "interrupt"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $timeoutPath -LaneId "local-success" `
            -CandidateFingerprint $candidate -EvidenceRoot $interruptRoot `
            -CertificationFault `
                "assertion-cancellation-after-process-acquisition" | Out-Null
    } -ExpectedMessage "finished 'interrupted'"
    $interruptResult = Get-LaneResult -EvidenceRoot $interruptRoot -LaneId "local-success"
    Assert-Equal $interruptResult.state "interrupted" "Interrupted lane state"
    Assert-Equal $interruptResult.cleanup_restoration.state "passed" "Interrupted lane cleanup"

    $externalFinalizationAdapter = Copy-JsonValue $adapter
    $externalFinalizationLane = @($externalFinalizationAdapter.lanes | Where-Object {
        $_.id -ceq "no-topology"
    })[0]
    $externalFinalizationLane.actions[0].stage = "finalization"
    $externalFinalizationPath = Join-Path $root "external-finalization-adapter.json"
    Write-JsonFile -Path $externalFinalizationPath -Document $externalFinalizationAdapter
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $externalFinalizationPath | Out-Null
    } -ExpectedMessage "does not satisfy contract v2"

    $noAssertionAdapter = Copy-JsonValue $adapter
    $noAssertionLane = @($noAssertionAdapter.lanes | Where-Object {
        $_.id -ceq "no-topology"
    })[0]
    $noAssertionLane.actions[0].stage = "setup"
    $noAssertionLane.actions[0].PSObject.Properties.Remove("implementation_target")
    $noAssertionLane.actions[0].PSObject.Properties.Remove("proof_classes")
    $noAssertionPath = Join-Path $root "no-assertion-adapter.json"
    Write-JsonFile -Path $noAssertionPath -Document $noAssertionAdapter
    Assert-Rejected -Action {
        Assert-TessaraValidationAdapter -AdapterPath $noAssertionPath | Out-Null
    } -ExpectedMessage "must declare at least one product assertion action"

    $parallelRoots = @(
        (Join-Path $root "parallel-a")
        (Join-Path $root "parallel-b")
    )
    $parallelResultPaths = @(
        (Join-Path $root "parallel-a-result.json")
        (Join-Path $root "parallel-b-result.json")
    )
    $hostExecutable = (Get-Process -Id $PID).Path
    $workers = [Collections.Generic.List[object]]::new()
    try {
        for ($parallelIndex = 0; $parallelIndex -lt $parallelRoots.Count; $parallelIndex++) {
            $startInfo = [Diagnostics.ProcessStartInfo]::new()
            $startInfo.FileName = $hostExecutable
            $startInfo.WorkingDirectory = $repoRoot
            $startInfo.UseShellExecute = $false
            $startInfo.CreateNoWindow = $true
            $startInfo.RedirectStandardOutput = $true
            $startInfo.RedirectStandardError = $true
            foreach ($argument in @(
                    "-NoProfile",
                    "-File", $PSCommandPath,
                    "-ParallelWorker",
                    "-WorkerModulePath", $modulePath,
                    "-WorkerAdapterPath", $adapterPath,
                    "-WorkerEvidenceRoot", $parallelRoots[$parallelIndex],
                    "-WorkerCandidateFingerprint", $candidate,
                    "-WorkerResultPath", $parallelResultPaths[$parallelIndex]
                )) {
                $startInfo.ArgumentList.Add([string]$argument)
            }

            $process = [Diagnostics.Process]::new()
            $process.StartInfo = $startInfo
            if (-not $process.Start()) {
                throw "Parallel certification worker could not be started."
            }
            $workers.Add([pscustomobject][ordered]@{
                    process = $process
                    stdout = $process.StandardOutput.ReadToEndAsync()
                    stderr = $process.StandardError.ReadToEndAsync()
                })
        }

        $parallelDeadline = [DateTime]::UtcNow.AddSeconds(120)
        while (@($workers | Where-Object { -not $_.process.HasExited }).Count -ne 0) {
            if ([DateTime]::UtcNow -ge $parallelDeadline) {
                throw "Parallel certification workers exceeded the bounded 120-second deadline."
            }
            Start-Sleep -Milliseconds 50
        }

        $parallelResults = [Collections.Generic.List[object]]::new()
        for ($parallelIndex = 0; $parallelIndex -lt $workers.Count; $parallelIndex++) {
            $worker = $workers[$parallelIndex]
            $workerOutput = $worker.stdout.GetAwaiter().GetResult()
            $workerError = $worker.stderr.GetAwaiter().GetResult()
            if ($worker.process.ExitCode -ne 0) {
                throw "Parallel certification worker $parallelIndex failed with exit code $($worker.process.ExitCode): $workerError $workerOutput"
            }
            if (-not (Test-Path -LiteralPath $parallelResultPaths[$parallelIndex] `
                    -PathType Leaf)) {
                throw "Parallel certification worker $parallelIndex did not publish its result."
            }
            $parallelResults.Add((Get-Content -Raw `
                        -LiteralPath $parallelResultPaths[$parallelIndex] | ConvertFrom-Json))
        }
        if (@($parallelResults.attempt_id | Sort-Object -Unique).Count -ne 2 -or
            @($parallelResults.ports.http | Sort-Object -Unique).Count -ne 2) {
            throw "Parallel-safe lane invocations did not retain unique identities and ports."
        }
    } finally {
        foreach ($worker in $workers) {
            try {
                if (-not $worker.process.HasExited) {
                    $worker.process.Kill($true)
                    $worker.process.WaitForExit(5000) | Out-Null
                }
            } finally {
                $worker.process.Dispose()
            }
        }
    }

    $shimRoot = Join-Path $root "docker-shim"
    [IO.Directory]::CreateDirectory($shimRoot) | Out-Null
    $env:TESSARA_VP_DOCKER_SHIM_ROOT = $shimRoot

    $toolSwapShimPath = Join-Path $root "tool-swap-docker-shim.ps1"
    Copy-Item -LiteralPath $dockerShimPath -Destination $toolSwapShimPath
    $toolSwapAdapter = Copy-JsonValue $adapter
    $toolSwapRelative = [IO.Path]::GetRelativePath(
        $repoRoot, $toolSwapShimPath
    ).Replace('\', '/')
    $toolSwapAdapter.execution_inputs += [pscustomobject][ordered]@{
        path = $toolSwapRelative
        lanes = @("compose-create")
    }
    $toolSwapLane = @($toolSwapAdapter.lanes | Where-Object {
        $_.id -ceq "compose-create"
    })[0]
    $toolSwapLane.actions[0].arguments = @(
        "-NoProfile", "-NonInteractive", "-File",
        '${repository_root}/scripts/validation-platform/fixtures/synthetic-action.ps1',
        "-Mode", "mutate-file", "-Path", "`${repository_root}/$toolSwapRelative"
    )
    $toolSwapAdapterPath = Join-Path $root "tool-swap-adapter.json"
    Write-JsonFile -Path $toolSwapAdapterPath -Document $toolSwapAdapter
    $toolSwapEvidenceRoot = Join-Path $root "tool-swap"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $toolSwapAdapterPath `
            -LaneId "compose-create" -CandidateFingerprint $candidate `
            -EvidenceRoot $toolSwapEvidenceRoot `
            -DockerCommand @("pwsh", "-NoProfile", "-NonInteractive", "-File", $toolSwapShimPath) `
            -DockerCommandInputPaths @($toolSwapRelative) |
            Out-Null
    } -ExpectedMessage "checkpoint was revoked and the lane must rerun"
    $toolSwapCheckpointPath = @(
        Get-ChildItem -LiteralPath (Join-Path $toolSwapEvidenceRoot `
            "lanes/compose-create/attempts") -Filter "execution-complete.json" `
            -File -Recurse | Sort-Object FullName | Select-Object -Last 1
    )[0].FullName
    $toolSwapAttemptRoot = Split-Path -Parent $toolSwapCheckpointPath
    $toolSwapCheckpoint = Get-Content -Raw -LiteralPath $toolSwapCheckpointPath |
        ConvertFrom-Json -Depth 100
    $toolSwapResult = $toolSwapCheckpoint.result
    Assert-Equal $toolSwapResult.failure_stage "setup" "Tool substitution failure stage"
    $toolSwapFailureMessage = [string]$toolSwapResult.failure.message
    if ($toolSwapFailureMessage -notlike "*identity changed immediately*" -and
        $toolSwapFailureMessage -notlike "*Lane-owned input*changed during execution*") {
        throw "Tool substitution was not rejected by an exact pre-topology identity gate: $toolSwapFailureMessage"
    }
    if (-not [bool]$toolSwapResult.source_integrity_failure -or
        -not (Test-Path -LiteralPath (Join-Path $toolSwapAttemptRoot `
                "execution-revoked.json") -PathType Leaf) -or
        (Test-Path -LiteralPath (Join-Path $toolSwapAttemptRoot "lane-result.json")) -or
        (Test-Path -LiteralPath (Join-Path $toolSwapAttemptRoot "attempt-index.json")) -or
        (Test-Path -LiteralPath (Join-Path $toolSwapAttemptRoot `
                "topology-ownership-acquired.json"))) {
        throw "Tool substitution did not retain only failed, revoked, non-authoritative evidence."
    }
    Assert-EvidencePairs -AttemptRoot $toolSwapAttemptRoot
    Assert-Equal @(Get-DockerShimLiveResidue -Root $shimRoot).Count 0 `
        "Tool substitution Docker execution"

    $consumeSetupFailureAdapter = Copy-JsonValue $adapter
    $consumeSetupFailureLane = @($consumeSetupFailureAdapter.lanes | Where-Object {
        [string]$_.id -ceq "compose-consume"
    })[0]
    $consumeSetupFailureAction = Copy-JsonValue $consumeSetupFailureLane.actions[0]
    $consumeSetupFailureAction.id = "consume-setup-failure"
    $consumeSetupFailureAction.stage = "setup"
    $consumeSetupFailureAction.PSObject.Properties.Remove("implementation_target")
    $consumeSetupFailureAction.PSObject.Properties.Remove("proof_classes")
    $consumeSetupFailureAction.arguments = @(
        "-NoProfile", "-NonInteractive", "-File",
        '${repository_root}/scripts/validation-platform/fixtures/synthetic-action.ps1',
        "-Mode", "fail", "-ExitCode", "41"
    )
    $consumeSetupFailureLane.actions = @($consumeSetupFailureAction) +
        @($consumeSetupFailureLane.actions)
    $consumeSetupFailurePath = Join-Path $root "consume-setup-failure-adapter.json"
    Write-JsonFile -Path $consumeSetupFailurePath -Document $consumeSetupFailureAdapter
    $consumeSetupFailureRoot = Join-Path $root "consume-setup-failure"
    $consumeSetupFailureCreate = Invoke-TessaraValidationLane `
        -AdapterPath $consumeSetupFailurePath -LaneId "compose-create" `
        -CandidateFingerprint $candidate -EvidenceRoot $consumeSetupFailureRoot `
        -DockerCommand $dockerCommand -DockerCommandInputPaths $dockerCommandInputPaths
    $consumeSetupFailureReceiptPath = `
        [string]$consumeSetupFailureCreate.cleanup_restoration.receipt.path
    $consumeSetupFailureReceipt = Get-Content -Raw `
        -LiteralPath $consumeSetupFailureReceiptPath | ConvertFrom-Json -Depth 50
    $consumeSetupFailureClaimPath = Join-Path $consumeSetupFailureRoot (
        "topology-claims/$([string]$consumeSetupFailureReceipt.topology_lease_sha256).json"
    )
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $consumeSetupFailurePath `
            -LaneId "compose-consume" -CandidateFingerprint $candidate `
            -EvidenceRoot $consumeSetupFailureRoot `
            -TopologyReceiptPath $consumeSetupFailureReceiptPath `
            -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $consumeSetupFailureResult = Get-LaneResult -EvidenceRoot $consumeSetupFailureRoot `
        -LaneId "compose-consume"
    Assert-Equal $consumeSetupFailureResult.failure_stage "setup" `
        "Claimed consumer setup-action failure stage"
    Assert-Equal $consumeSetupFailureResult.assertions_started $false `
        "Claimed consumer setup-action assertion state"
    Assert-Equal $consumeSetupFailureResult.cleanup_restoration.mode "destroyed" `
        "Claimed consumer setup-action topology teardown"
    if (-not (Test-Path -LiteralPath $consumeSetupFailureClaimPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath "$consumeSetupFailureClaimPath.sha256" -PathType Leaf)) {
        throw "Consumer setup-action failure did not retain its authenticated one-time claim."
    }
    Assert-Equal @(Get-DockerShimLiveResidue -Root $shimRoot).Count 0 `
        "Claimed consumer setup-action Docker residue"

    $handoffTransferRoot = Join-Path $root "handoff-transfer-failure"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "compose-create" -CandidateFingerprint $candidate `
            -EvidenceRoot $handoffTransferRoot -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths `
            -CertificationFault "after-handoff-finalization" | Out-Null
    } -ExpectedMessage "Handoff evidence finalization failed"
    $handoffTransferResult = Get-LaneResult -EvidenceRoot $handoffTransferRoot `
        -LaneId "compose-create"
    $handoffTransferAttemptRoot = Split-Path -Parent $handoffTransferResult.evidence_path
    $handoffTransferRevocationPath = Join-Path $handoffTransferAttemptRoot `
        "execution-revoked.json"
    $handoffTransferReceiptPath = `
        [string]$handoffTransferResult.cleanup_restoration.receipt.path
    $handoffTransferReceipt = Get-Content -Raw `
        -LiteralPath $handoffTransferReceiptPath | ConvertFrom-Json -Depth 50
    $handoffTransferClaimPath = Join-Path $handoffTransferRoot (
        "topology-claims/$([string]$handoffTransferReceipt.topology_lease_sha256).json"
    )
    if (-not [bool]$handoffTransferResult.authoritative -or
        -not (Test-Path -LiteralPath $handoffTransferRevocationPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath "$handoffTransferRevocationPath.sha256" -PathType Leaf) -or
        @(Get-DockerShimLiveResidue -Root $shimRoot).Count -ne 0) {
        throw "A post-finalizer handoff fault did not revoke authority before teardown."
    }
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "compose-consume" -CandidateFingerprint $candidate `
            -EvidenceRoot $handoffTransferRoot `
            -TopologyReceiptPath $handoffTransferReceiptPath `
            -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $handoffTransferConsumer = Get-LaneResult -EvidenceRoot $handoffTransferRoot `
        -LaneId "compose-consume"
    if ([bool]$handoffTransferConsumer.assertions_started -or
        [string]$handoffTransferConsumer.failure.message -notlike
            "*source attempt was revoked*" -or
        (Test-Path -LiteralPath $handoffTransferClaimPath) -or
        (Test-Path -LiteralPath "$handoffTransferClaimPath.sha256")) {
        throw "A consumer accepted the revoked post-finalizer handoff."
    }

    $claimOwnershipRoot = Join-Path $root "claim-ownership-transfer"
    $claimOwnershipCreate = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "compose-create" -CandidateFingerprint $candidate `
        -EvidenceRoot $claimOwnershipRoot -DockerCommand $dockerCommand `
        -DockerCommandInputPaths $dockerCommandInputPaths
    $claimOwnershipReceiptPath = `
        [string]$claimOwnershipCreate.cleanup_restoration.receipt.path
    $claimOwnershipReceipt = Get-Content -Raw -LiteralPath $claimOwnershipReceiptPath |
        ConvertFrom-Json -Depth 50
    $claimOwnershipPath = Join-Path $claimOwnershipRoot (
        "topology-claims/$([string]$claimOwnershipReceipt.topology_lease_sha256).json"
    )
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "compose-consume" -CandidateFingerprint $candidate `
            -EvidenceRoot $claimOwnershipRoot `
            -TopologyReceiptPath $claimOwnershipReceiptPath `
            -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths `
            -CertificationFault "after-consumer-claim" | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $claimOwnershipResult = Get-LaneResult -EvidenceRoot $claimOwnershipRoot `
        -LaneId "compose-consume"
    if ([bool]$claimOwnershipResult.assertions_started -or
        [string]$claimOwnershipResult.failure.message -notlike
            "*Injected failure after complete topology claim publication.*" -or
        [string]$claimOwnershipResult.cleanup_restoration.mode -cne "destroyed" -or
        -not (Test-Path -LiteralPath $claimOwnershipPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath "$claimOwnershipPath.sha256" -PathType Leaf) -or
        @(Get-DockerShimLiveResidue -Root $shimRoot).Count -ne 0) {
        throw "A complete topology claim did not retain cleanup ownership through failure."
    }

    $composeRoot = Join-Path $root "compose"
    $create = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "compose-create" -CandidateFingerprint $candidate `
        -EvidenceRoot $composeRoot -DockerCommand $dockerCommand `
        -DockerCommandInputPaths $dockerCommandInputPaths
    Assert-Equal $create.cleanup_restoration.mode "retained-for-handoff" "Compose handoff"
    $receiptPath = [string]$create.cleanup_restoration.receipt.path
    $composeSourceAttemptRoot = Split-Path -Parent $create.evidence_path
    $composeSourceAttestationPath = Join-Path $composeSourceAttemptRoot `
        "finalization-attestation.$([string]$identity.finalization_fingerprint).json"
    $composeAttestationBytes = [IO.File]::ReadAllBytes($composeSourceAttestationPath)
    $composeAttestationSidecarBytes = [IO.File]::ReadAllBytes(
        "$composeSourceAttestationPath.sha256"
    )
    $composeReceipt = Get-Content -Raw -LiteralPath $receiptPath |
        ConvertFrom-Json -Depth 100
    $composeClaimPath = Join-Path $composeRoot `
        "topology-claims/$([string]$composeReceipt.topology_lease_sha256).json"
    try {
        $staleComposeAttestation = [Text.UTF8Encoding]::new($false, $true).GetString(
            $composeAttestationBytes
        ) | ConvertFrom-Json -Depth 100
        $staleComposeAttestation.finalization_platform_fingerprint = `
            $priorFinalizationFingerprint
        $staleComposeAttestationBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (($staleComposeAttestation | ConvertTo-Json -Depth 100 -Compress) + "`n")
        )
        [IO.File]::WriteAllBytes(
            $composeSourceAttestationPath, $staleComposeAttestationBytes
        )
        $staleComposeAttestationHash = [Convert]::ToHexString(
            [Security.Cryptography.SHA256]::HashData($staleComposeAttestationBytes)
        ).ToLowerInvariant()
        [IO.File]::WriteAllText(
            "$composeSourceAttestationPath.sha256",
            "$staleComposeAttestationHash  $([IO.Path]::GetFileName($composeSourceAttestationPath))`n",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath `
                -LaneId "compose-consume" -CandidateFingerprint $candidate `
                -EvidenceRoot $composeRoot -TopologyReceiptPath $receiptPath `
                -DockerCommand $dockerCommand `
                -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
        } -ExpectedMessage "Topology source finalization commit authentication failed"
        $staleHandoffResult = Get-LaneResult -EvidenceRoot $composeRoot `
            -LaneId "compose-consume"
        if ([string]$staleHandoffResult.state -ceq "passed" -or
            [bool]$staleHandoffResult.assertions_started -or
            (Test-Path -LiteralPath $composeClaimPath) -or
            (Test-Path -LiteralPath "$composeClaimPath.sha256")) {
            throw "A stale-finalizer topology source reached assertion or ownership claim."
        }
    } finally {
        [IO.File]::WriteAllBytes($composeSourceAttestationPath, $composeAttestationBytes)
        [IO.File]::WriteAllBytes(
            "$composeSourceAttestationPath.sha256", $composeAttestationSidecarBytes
        )
    }

    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "compose-consume" -CandidateFingerprint $candidate `
            -EvidenceRoot $composeRoot -TopologyReceiptPath $receiptPath `
            -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths `
            -CertificationFault "consumer-compose-config-substitution" | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $driftedConsumerResult = Get-LaneResult -EvidenceRoot $composeRoot `
        -LaneId "compose-consume"
    if ([string]$driftedConsumerResult.failure.message -notlike
        "*configuration substituted project*" -or
        [bool]$driftedConsumerResult.assertions_started -or
        (Test-Path -LiteralPath $composeClaimPath) -or
        (Test-Path -LiteralPath "$composeClaimPath.sha256")) {
        throw "Consumer configuration drift was not contained before the reusable handoff claim."
    }

    $tamperedReceipt = Join-Path $composeRoot "tampered-topology.json"
    $tampered = Get-Content -Raw -LiteralPath $receiptPath | ConvertFrom-Json -Depth 50
    $tampered.handoff_to = "local-success"
    Write-JsonFile -Path $tamperedReceipt -Document $tampered
    $tamperedHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $tamperedReceipt).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText("$tamperedReceipt.sha256", "$tamperedHash  tampered-topology.json`n")
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath -LaneId "compose-consume" `
            -CandidateFingerprint $candidate -EvidenceRoot $composeRoot `
            -TopologyReceiptPath $tamperedReceipt -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
    } -ExpectedMessage "canonical source-attempt receipt"

    $copiedComposeRoot = Join-Path $root "copied-compose"
    [IO.Directory]::CreateDirectory($copiedComposeRoot) | Out-Null
    Copy-Item -LiteralPath (Join-Path $composeRoot "lanes") `
        -Destination $copiedComposeRoot -Recurse
    $copiedReceiptPath = Join-Path $copiedComposeRoot (
        [IO.Path]::GetRelativePath($composeRoot, $receiptPath)
    )
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath -LaneId "compose-consume" `
            -CandidateFingerprint $candidate -EvidenceRoot $copiedComposeRoot `
            -TopologyReceiptPath $copiedReceiptPath -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
    } -ExpectedMessage "does not bind the exact source"

    $consume = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "compose-consume" -CandidateFingerprint $candidate `
        -EvidenceRoot $composeRoot -TopologyReceiptPath $receiptPath `
        -DockerCommand $dockerCommand -DockerCommandInputPaths $dockerCommandInputPaths
    Assert-Equal $consume.cleanup_restoration.mode "destroyed" "Compose final teardown"
    Assert-Equal @(Get-DockerShimLiveResidue -Root $shimRoot).Count 0 `
        "Compose shim residue"
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "compose-consume" -CandidateFingerprint $candidate `
            -EvidenceRoot $composeRoot -TopologyReceiptPath $receiptPath `
            -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $replayResult = Get-LaneResult -EvidenceRoot $composeRoot -LaneId "compose-consume"
    Assert-Equal $replayResult.failure_stage "setup" "Replayed handoff failure stage"
    Assert-Equal $replayResult.assertions_started $false "Replayed handoff assertion state"

    $claimFailureRoot = Join-Path $root "claim-publication-failure"
    $claimFailureCreate = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "compose-create" -CandidateFingerprint $candidate `
        -EvidenceRoot $claimFailureRoot -DockerCommand $dockerCommand `
        -DockerCommandInputPaths $dockerCommandInputPaths
    $claimFailureReceiptPath = [string]$claimFailureCreate.cleanup_restoration.receipt.path
    $claimFailureReceipt = Get-Content -Raw -LiteralPath $claimFailureReceiptPath |
        ConvertFrom-Json -Depth 50
    $claimFailurePath = Join-Path $claimFailureRoot (
        "topology-claims/$([string]$claimFailureReceipt.topology_lease_sha256).json"
    )
    Assert-Rejected -Action {
        Invoke-TessaraValidationLane -AdapterPath $adapterPath `
            -LaneId "compose-consume" -CandidateFingerprint $candidate `
            -EvidenceRoot $claimFailureRoot `
            -TopologyReceiptPath $claimFailureReceiptPath `
            -DockerCommand $dockerCommand `
            -DockerCommandInputPaths $dockerCommandInputPaths `
            -CertificationFault "partial-claim-publication" | Out-Null
    } -ExpectedMessage "finished 'failed'"
    $claimFailureResult = Get-LaneResult -EvidenceRoot $claimFailureRoot `
        -LaneId "compose-consume"
    Assert-Equal $claimFailureResult.failure_stage "setup" `
        "Partial claim publication failure stage"
    Assert-Equal $claimFailureResult.assertions_started $false `
        "Partial claim publication assertion state"
    Assert-Equal $claimFailureResult.cleanup_restoration.mode "destroyed" `
        "Partial claim publication topology teardown"
    if ([string]$claimFailureResult.failure.message -notlike
        "*Injected failure after topology claim data creation.*") {
        throw "Partial claim publication certification did not reach its injected boundary."
    }
    if (-not (Test-Path -LiteralPath $claimFailurePath -PathType Leaf) -or
        (Test-Path -LiteralPath "$claimFailurePath.sha256")) {
        throw "Partial claim publication did not retain exactly one durable claim tombstone."
    }
    Assert-Equal @(Get-DockerShimLiveResidue -Root $shimRoot).Count 0 `
        "Partial claim publication Docker residue"

    $claimRetryCreate = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "compose-create" -CandidateFingerprint $candidate `
        -EvidenceRoot $claimFailureRoot -DockerCommand $dockerCommand `
        -DockerCommandInputPaths $dockerCommandInputPaths
    $claimRetryReceiptPath = [string]$claimRetryCreate.cleanup_restoration.receipt.path
    $claimRetryReceipt = Get-Content -Raw -LiteralPath $claimRetryReceiptPath |
        ConvertFrom-Json -Depth 50
    $claimRetryPath = Join-Path $claimFailureRoot (
        "topology-claims/$([string]$claimRetryReceipt.topology_lease_sha256).json"
    )
    $claimRetry = Invoke-TessaraValidationLane -AdapterPath $adapterPath `
        -LaneId "compose-consume" -CandidateFingerprint $candidate `
        -EvidenceRoot $claimFailureRoot -TopologyReceiptPath $claimRetryReceiptPath `
        -DockerCommand $dockerCommand -DockerCommandInputPaths $dockerCommandInputPaths
    Assert-Equal $claimRetry.state "passed" "Claim publication retry"
    if (-not (Test-Path -LiteralPath $claimRetryPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath "$claimRetryPath.sha256" -PathType Leaf)) {
        throw "Claim publication retry did not produce a complete claim pair."
    }
    Assert-Equal @(Get-DockerShimLiveResidue -Root $shimRoot).Count 0 `
        "Claim publication retry Docker residue"

    $env:TESSARA_VP_DOCKER_SHIM_FOREIGN_CONFIG = "1"
    try {
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath -LaneId "compose-create" `
                -CandidateFingerprint $candidate -EvidenceRoot (Join-Path $root "foreign-compose") `
                -DockerCommand $dockerCommand `
                -DockerCommandInputPaths $dockerCommandInputPaths | Out-Null
        } -ExpectedMessage "substituted project"
    } finally {
        Remove-Item Env:TESSARA_VP_DOCKER_SHIM_FOREIGN_CONFIG -ErrorAction SilentlyContinue
    }

    $env:TESSARA_VP_DOCKER_SHIM_RETAIN = "1"
    try {
        Assert-Rejected -Action {
            Invoke-TessaraValidationLane -AdapterPath $adapterPath -LaneId "compose-create" `
                -CandidateFingerprint $candidate -EvidenceRoot (Join-Path $root "residue-compose") `
                -DockerCommand $dockerCommand `
                -DockerCommandInputPaths $dockerCommandInputPaths `
                -CertificationFault "after-handoff-finalization" | Out-Null
        } -ExpectedMessage "cleanup"
    } finally {
        Remove-Item Env:TESSARA_VP_DOCKER_SHIM_RETAIN -ErrorAction SilentlyContinue
        Get-ChildItem -LiteralPath $shimRoot -Filter "*.json" -File -ErrorAction SilentlyContinue |
            Remove-Item -Force
    }

    $dockerTranscriptPath = Join-Path $shimRoot "docker-transcript.jsonl"
    if (-not (Test-Path -LiteralPath $dockerTranscriptPath -PathType Leaf)) {
        throw "Docker shim did not retain its exact command transcript."
    }
    $dockerTranscriptActions = @(
        Get-Content -LiteralPath $dockerTranscriptPath |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            ForEach-Object {
                $entry = $_ | ConvertFrom-Json -Depth 20
                @($entry.arguments | Where-Object {
                        [string]$_ -in @("config", "up", "ps", "down")
                    }) | Select-Object -Last 1
            }
    )
    foreach ($requiredComposeAction in @("config", "up", "ps", "down")) {
        if (@($dockerTranscriptActions | Where-Object {
                    [string]$_ -ceq $requiredComposeAction
                }).Count -eq 0) {
            throw "Docker shim transcript did not exercise '$requiredComposeAction'."
        }
    }

    $certificationHarnessCurrent = Get-CertificationHarnessSnapshot `
        -RepositoryRoot $repoRoot -RelativePaths $certificationHarnessRelativePaths
    Assert-Equal $certificationHarnessCurrent.fingerprint `
        $certificationHarnessSnapshot.fingerprint `
        "Certification harness end-of-run fingerprint"

    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.platform-certification"
        state = "passed"
        application_suites_executed = $false
        platform = $identity
        certification_harness = $certificationHarnessSnapshot
        certified = @(
            "aggregate-identity", "certification-harness-identity", "adapter-contract",
            "reserved-base-environment-collision-rejection", "no-topology",
            "isolated-git-environment",
            "finalization-only-recovery", "idempotent-finalization-recovery",
            "positive-post-checkpoint-integrity",
            "finalizer-exact-attempt-inventory", "finalizer-bounded-attempt-files",
            "finalizer-bounded-attempt-directories",
            "finalizer-publication-write-fault-recovery",
            "finalizer-reparse-inventory-rejection", "finalizer-attestation-commit-gate",
            "append-only-finalizer-reattestation",
            "finalizer-attestation-consumer-authentication",
            "finalizer-prerequisite-commit-recheck",
            "evidence-root-reparse-containment", "adapter-action-budget",
            "adapter-lane-time-budget",
            "topology-port-budget", "handoff-port-set-validation",
            "interpreter-bypass-rejection",
            "local-success", "hermetic-environment", "environment-restoration",
            "post-start-process-acquisition-containment",
            "truthful-assertion-process-start-accounting",
            "post-setup-topology-tool-recheck",
            "prerequisite-gating", "lane-scoped-invalidation",
            "planner-mutation-matrix",
            "git-nul-path-and-mode-binding",
            "required-source-setup-failure",
            "whitespace-source-pre-topology-containment",
            "secret-free-environment-evidence", "post-redaction-output-bound",
            "execution-input-drift-rejection",
            "child-failure", "timeout",
            "interruption", "failure-stage-classification",
            "external-finalization-actions-rejected", "no-op-lanes-rejected",
            "parallel-safe-leases",
            "per-invocation-tool-substitution-rejection",
            "compose-handoff", "compose-project-identity", "single-use-handoff",
            "preclaim-compose-drift-containment",
            "claimed-consumer-setup-failure-containment",
            "post-finalizer-handoff-transfer-containment",
            "complete-claim-cleanup-ownership",
            "partial-claim-publication-containment",
            "cross-root-handoff-replay-rejection",
            "compose-residue-rejection", "immutable-evidence",
            "component-lease-preservation"
        )
    } | ConvertTo-Json -Depth 20
} finally {
    foreach ($name in $shimEnvironmentNames) {
        if ([bool]$shimEnvironmentBefore[$name].present) {
            [Environment]::SetEnvironmentVariable(
                $name, [string]$shimEnvironmentBefore[$name].value, "Process"
            )
        } else {
            Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
        }
    }
    foreach ($name in $certificationEnvironmentNames) {
        if ([bool]$certificationEnvironmentBefore[$name].present) {
            [Environment]::SetEnvironmentVariable(
                $name, [string]$certificationEnvironmentBefore[$name].value, "Process"
            )
        } else {
            Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
        }
    }
    if ($laneASourceWasPresent) {
        [Environment]::SetEnvironmentVariable(
            "TESSARA_VP_LANE_A_SOURCE", $laneASourceBefore, "Process"
        )
    } else {
        Remove-Item Env:TESSARA_VP_LANE_A_SOURCE -ErrorAction SilentlyContinue
    }
    if ($sentinelWasPresent) {
        [Environment]::SetEnvironmentVariable(
            "TESSARA_VP_SENTINEL", $previousSentinel, "Process"
        )
    } else {
        Remove-Item Env:TESSARA_VP_SENTINEL -ErrorAction SilentlyContinue
    }
    Remove-CertificationRoot -Path $root -RepositoryRoot $repoRoot `
        -Capability $rootCapability
}

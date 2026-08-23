Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:FinalizerContract = "tessara.validation.finalizer"
$script:FinalizerRelease = "2.0.0"
$script:FinalizerMaximumAttemptFileCount = 256
$script:FinalizerMaximumAttemptDirectoryCount = 64
$script:FinalizerMaximumAttemptDepth = 8
$script:FinalizerMaximumRelativePathCharacters = 512
$script:FinalizerMaximumFileBytes = 16MB
$script:FinalizerMaximumAttemptBytes = 64MB
$script:FinalizerPublicationDataPaths = @(
    "execution-complete.json",
    "execution-integrity-verified.json",
    "lane-result.json",
    "attempt-index.json",
    "execution-revoked.json"
)
$script:FinalizerAttestationDataPattern = `
    '^finalization-attestation\.([0-9a-f]{64})\.json$'
$script:FinalizerAttestationSidecarPattern = `
    '^finalization-attestation\.([0-9a-f]{64})\.json\.sha256$'
$script:FinalizerModulePath = [IO.Path]::GetFullPath($PSCommandPath)
$script:FinalizerModuleSha256AtImport = (
    Get-FileHash -Algorithm SHA256 -LiteralPath $script:FinalizerModulePath
).Hash.ToLowerInvariant()

function Get-TessaraValidationFinalizerIdentity {
    [CmdletBinding()]
    param()
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = $script:FinalizerContract
        release_version = $script:FinalizerRelease
        module_sha256 = $script:FinalizerModuleSha256AtImport
    }
}

function Get-TessaraFinalizerSha256Bytes {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Split-TessaraFinalizerNativePath {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Path)
    $separators = [Collections.Generic.List[char]]::new()
    $separators.Add([IO.Path]::DirectorySeparatorChar)
    if ([IO.Path]::AltDirectorySeparatorChar -ne [IO.Path]::DirectorySeparatorChar) {
        $separators.Add([IO.Path]::AltDirectorySeparatorChar)
    }
    @($Path.Split($separators.ToArray(), [StringSplitOptions]::RemoveEmptyEntries))
}

function Test-TessaraFinalizerLinkOrReparsePoint {
    param([Parameter(Mandatory)]$Item)
    (($Item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) -or
        ($Item.PSObject.Properties.Name -contains "LinkType" -and
            -not [string]::IsNullOrWhiteSpace([string]$Item.LinkType))
}

function Assert-TessaraFinalizerOrdinaryDirectoryChain {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )
    $resolvedRoot = [IO.Path]::GetFullPath($Root)
    $resolvedPath = [IO.Path]::GetFullPath($Path)
    $relative = [IO.Path]::GetRelativePath($resolvedRoot, $resolvedPath)
    if ([IO.Path]::IsPathRooted($relative) -or $relative -eq ".." -or
        $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
        $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
        throw "$Label is outside its evidence root: $resolvedPath"
    }
    $paths = [Collections.Generic.List[string]]::new()
    $paths.Add($resolvedRoot)
    $current = $resolvedRoot
    if ($relative -ne ".") {
        foreach ($segment in @(Split-TessaraFinalizerNativePath -Path $relative)) {
            $current = Join-Path $current $segment
            $paths.Add($current)
        }
    }
    foreach ($candidate in $paths) {
        $item = Get-Item -Force -LiteralPath $candidate
        if ($item -isnot [IO.DirectoryInfo] -or -not $item.PSIsContainer -or
            (Test-TessaraFinalizerLinkOrReparsePoint -Item $item)) {
            throw "$Label traverses a non-ordinary or reparse directory: $candidate"
        }
    }
    $resolvedPath
}

function Assert-TessaraFinalizerOrdinaryFileItem {
    param(
        [Parameter(Mandatory)]$Item,
        [Parameter(Mandatory)][string]$Label
    )
    $unixModeProperty = $Item.PSObject.Properties["UnixMode"]
    $isNonRegularUnixFile = $null -ne $unixModeProperty -and
        -not [string]::IsNullOrWhiteSpace([string]$unixModeProperty.Value) -and
        -not ([string]$unixModeProperty.Value).StartsWith(
            "-", [StringComparison]::Ordinal
        )
    if ($Item -isnot [IO.FileInfo] -or $Item.PSIsContainer -or
        (Test-TessaraFinalizerLinkOrReparsePoint -Item $Item) -or
        (($Item.Attributes -band [IO.FileAttributes]::Directory) -ne 0) -or
        (($Item.Attributes -band [IO.FileAttributes]::Device) -ne 0) -or
        $isNonRegularUnixFile) {
        throw "$Label is not an ordinary file: $($Item.FullName)"
    }
}

function Get-TessaraFinalizerAttemptInventory {
    param([Parameter(Mandatory)][string]$AttemptRoot)
    $root = [IO.Path]::GetFullPath($AttemptRoot)
    $rootItem = Get-Item -Force -LiteralPath $root
    if ($rootItem -isnot [IO.DirectoryInfo] -or -not $rootItem.PSIsContainer -or
        (Test-TessaraFinalizerLinkOrReparsePoint -Item $rootItem)) {
        throw "Attempt evidence root is not an ordinary directory: $root"
    }

    $pending = [Collections.Generic.Stack[string]]::new()
    $pending.Push($root)
    $inventory = [Collections.Generic.List[object]]::new()
    $paths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $directoryCount = 0
    [int64]$totalBytes = 0
    while ($pending.Count -gt 0) {
        $directory = $pending.Pop()
        foreach ($entryPath in [IO.Directory]::EnumerateFileSystemEntries($directory)) {
            $entry = Get-Item -Force -LiteralPath $entryPath
            $relative = [IO.Path]::GetRelativePath($root, $entry.FullName).Replace('\', '/')
            $depth = @($relative -split '/').Count
            if ([IO.Path]::IsPathRooted($relative) -or $relative -eq "." -or
                $relative -eq ".." -or $relative.StartsWith("../", [StringComparison]::Ordinal)) {
                throw "Attempt evidence inventory escaped its root: $($entry.FullName)"
            }
            if ($relative.Length -gt $script:FinalizerMaximumRelativePathCharacters) {
                throw "Attempt evidence path exceeds the $($script:FinalizerMaximumRelativePathCharacters)-character relative-path limit."
            }
            if ($depth -gt $script:FinalizerMaximumAttemptDepth) {
                throw "Attempt evidence path exceeds the $($script:FinalizerMaximumAttemptDepth)-level depth limit: $relative"
            }
            if (Test-TessaraFinalizerLinkOrReparsePoint -Item $entry) {
                throw "Attempt evidence path is a reparse point: $relative"
            }
            if ($entry -is [IO.DirectoryInfo] -and $entry.PSIsContainer) {
                if ($directoryCount -ge $script:FinalizerMaximumAttemptDirectoryCount) {
                    throw "Attempt evidence inventory exceeds the $($script:FinalizerMaximumAttemptDirectoryCount)-directory limit."
                }
                $directoryCount++
                $pending.Push($entry.FullName)
                continue
            }
            Assert-TessaraFinalizerOrdinaryFileItem -Item $entry `
                -Label "Attempt evidence inventory entry"
            if (-not $paths.Add($relative)) {
                throw "Attempt evidence inventory contains a duplicate path: $relative"
            }
            if ($inventory.Count -ge $script:FinalizerMaximumAttemptFileCount) {
                throw "Attempt evidence inventory exceeds the $($script:FinalizerMaximumAttemptFileCount)-file limit."
            }
            [int64]$length = $entry.Length
            if ($length -gt $script:FinalizerMaximumFileBytes) {
                throw "Attempt evidence file exceeds the $($script:FinalizerMaximumFileBytes)-byte limit: $relative"
            }
            if ($length -gt ($script:FinalizerMaximumAttemptBytes - $totalBytes)) {
                throw "Attempt evidence inventory exceeds the $($script:FinalizerMaximumAttemptBytes)-byte aggregate limit."
            }
            $totalBytes += $length
            $inventory.Add([pscustomobject][ordered]@{
                path = $relative
                full_path = [IO.Path]::GetFullPath($entry.FullName)
                length = $length
            })
        }
    }
    @($inventory | Sort-Object path)
}

function Read-TessaraFinalizerBoundedBytes {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )
    $resolved = [IO.Path]::GetFullPath($Path)
    $item = Get-Item -Force -LiteralPath $resolved
    Assert-TessaraFinalizerOrdinaryFileItem -Item $item -Label $Label
    if ([int64]$item.Length -gt $script:FinalizerMaximumFileBytes) {
        throw "$Label exceeds the $($script:FinalizerMaximumFileBytes)-byte limit: $resolved"
    }
    $stream = [IO.File]::Open(
        $resolved,
        [IO.FileMode]::Open,
        [IO.FileAccess]::Read,
        [IO.FileShare]::Read
    )
    try {
        [int64]$length = $stream.Length
        if ($length -gt $script:FinalizerMaximumFileBytes) {
            throw "$Label exceeds the $($script:FinalizerMaximumFileBytes)-byte limit: $resolved"
        }
        $bytes = [byte[]]::new([int]$length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) {
                throw "$Label changed while it was being read: $resolved"
            }
            $offset += $read
        }
        if ($stream.ReadByte() -ne -1) {
            throw "$Label grew while it was being read: $resolved"
        }
        ,$bytes
    } finally {
        $stream.Dispose()
    }
}

function Read-TessaraFinalizerHashedJson {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Label)
    $resolved = [IO.Path]::GetFullPath($Path)
    $sidecarPath = "$resolved.sha256"
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
        throw "$Label pair is missing: $resolved"
    }
    $bytes = Read-TessaraFinalizerBoundedBytes -Path $resolved -Label $Label
    $hash = Get-TessaraFinalizerSha256Bytes -Bytes $bytes
    $sidecarBytes = Read-TessaraFinalizerBoundedBytes -Path $sidecarPath `
        -Label "$Label digest sidecar"
    try {
        $sidecar = [Text.UTF8Encoding]::new($false, $true).GetString($sidecarBytes)
    } catch {
        throw "$Label digest sidecar is not strict UTF-8."
    }
    if ($sidecar -cne "$hash  $([IO.Path]::GetFileName($resolved))`n") {
        throw "$Label digest sidecar is invalid."
    }
    try {
        $document = [Text.UTF8Encoding]::new($false, $true).GetString($bytes) |
            ConvertFrom-Json -Depth 100
    } catch {
        throw "$Label is not valid UTF-8 JSON."
    }
    [pscustomobject][ordered]@{
        path = $resolved
        sha256 = $hash
        bytes = $bytes
        document = $document
    }
}

function Assert-TessaraFinalizerAttemptInventory {
    param(
        [Parameter(Mandatory)][object[]]$Inventory,
        [Parameter(Mandatory)]$CheckpointDocument,
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)][string]$CurrentFinalizationPlatformFingerprint
    )
    $actual = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in $Inventory) {
        if (-not $actual.Add([string]$entry.path)) {
            throw "Attempt evidence inventory contains a duplicate path."
        }
    }
    $expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $artifactPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($artifact in @($CheckpointDocument.artifacts)) {
        $artifactPath = [string]$artifact.path
        $segments = @($artifactPath -split '/')
        if ([string]::IsNullOrWhiteSpace($artifactPath) -or
            $artifactPath -cne $artifactPath.Replace('\', '/') -or
            [IO.Path]::IsPathRooted($artifactPath) -or
            $artifactPath.EndsWith(".sha256", [StringComparison]::Ordinal) -or
            @($segments | Where-Object { $_ -in @("", ".", "..") }).Count -ne 0 -or
            $script:FinalizerPublicationDataPaths -contains $artifactPath -or
            @($script:FinalizerPublicationDataPaths | Where-Object {
                    "$_.sha256" -ceq $artifactPath
                }).Count -ne 0 -or
            $artifactPath -cmatch $script:FinalizerAttestationDataPattern -or
            $artifactPath -cmatch $script:FinalizerAttestationSidecarPattern -or
            [string]$artifact.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            -not $artifactPaths.Add($artifactPath)) {
            throw "Execution checkpoint contains an invalid artifact inventory."
        }
        $artifactFullPath = [IO.Path]::GetFullPath((Join-Path $AttemptRoot $artifactPath))
        $roundTrip = [IO.Path]::GetRelativePath($AttemptRoot, $artifactFullPath).Replace('\', '/')
        if ($roundTrip -cne $artifactPath) {
            throw "Execution checkpoint contains an invalid artifact inventory."
        }
        $null = $expected.Add($artifactPath)
        $null = $expected.Add("$artifactPath.sha256")
    }
    $null = $expected.Add("execution-complete.json")
    $null = $expected.Add("execution-complete.json.sha256")
    $null = $expected.Add("execution-integrity-verified.json")
    $null = $expected.Add("execution-integrity-verified.json.sha256")

    foreach ($publicationPath in @(
            "lane-result.json", "attempt-index.json", "execution-revoked.json"
        )) {
        $sidecarPath = "$publicationPath.sha256"
        if ($actual.Contains($sidecarPath) -and -not $actual.Contains($publicationPath)) {
            throw "Attempt evidence inventory contains orphan digest sidecar: $sidecarPath"
        }
        if ($actual.Contains($publicationPath)) {
            $null = $expected.Add($publicationPath)
            if ($actual.Contains($sidecarPath)) {
                $null = $expected.Add($sidecarPath)
            }
        }
    }

    $attestationFingerprints = `
        [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($path in $actual) {
        if ($path -cmatch $script:FinalizerAttestationDataPattern) {
            $fingerprint = [string]$Matches[1]
            $sidecarPath = "$path.sha256"
            $null = $attestationFingerprints.Add($fingerprint)
            $null = $expected.Add($path)
            if ($actual.Contains($sidecarPath)) {
                $null = $expected.Add($sidecarPath)
            } elseif ($fingerprint -cne $CurrentFinalizationPlatformFingerprint) {
                throw "Attempt evidence inventory contains incomplete prior finalization attestation: $path"
            }
        } elseif ($path -cmatch $script:FinalizerAttestationSidecarPattern) {
            $dataPath = $path.Substring(0, $path.Length - ".sha256".Length)
            if (-not $actual.Contains($dataPath)) {
                throw "Attempt evidence inventory contains orphan finalization attestation sidecar: $path"
            }
        }
    }

    foreach ($path in $actual) {
        if (-not $expected.Contains($path)) {
            if ($path.EndsWith(".sha256", [StringComparison]::Ordinal)) {
                throw "Attempt evidence inventory contains orphan or unlisted digest sidecar: $path"
            }
            throw "Attempt evidence inventory contains unlisted file: $path"
        }
    }
    foreach ($path in $expected) {
        if (-not $actual.Contains($path)) {
            throw "Attempt evidence inventory is missing checkpoint-declared file: $path"
        }
    }
    [pscustomobject][ordered]@{
        artifact_paths = $artifactPaths
        attestation_fingerprints = @($attestationFingerprints | Sort-Object)
    }
}

function Assert-TessaraFinalizerArtifactDigests {
    param(
        [Parameter(Mandatory)]$CheckpointDocument,
        [Parameter(Mandatory)][string]$AttemptRoot
    )
    foreach ($artifact in @($CheckpointDocument.artifacts)) {
        $artifactPath = [string]$artifact.path
        $snapshot = Read-TessaraFinalizerHashedJsonOrText `
            -Path (Join-Path $AttemptRoot $artifactPath) -Label "Checkpoint artifact"
        if ([string]$snapshot.sha256 -cne [string]$artifact.sha256) {
            throw "Execution checkpoint artifact digest changed: $artifactPath"
        }
    }
}

function Test-TessaraFinalizerContainedPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Path
    )
    $relative = [IO.Path]::GetRelativePath(
        [IO.Path]::GetFullPath($Root),
        [IO.Path]::GetFullPath($Path)
    )
    -not [IO.Path]::IsPathRooted($relative) -and
        $relative -ne ".." -and
        -not $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -and
        -not $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)
}

function Write-TessaraFinalizerCreateNewBytes {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes,
        [Parameter(Mandatory)][string]$Label,
        [AllowNull()][scriptblock]$AfterCreate
    )
    $stream = $null
    try {
        $stream = [IO.File]::Open(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::Read
        )
        if ($null -ne $AfterCreate) {
            & $AfterCreate $stream $Path | Out-Null
        }
        $stream.Write($Bytes, 0, $Bytes.Length)
        $stream.Flush($true)
        $stream.Dispose()
        $stream = $null
    } catch {
        $publicationFailure = $_
        # A non-null stream proves this invocation won CreateNew. Never delete
        # after an open collision, because that path belongs to another writer.
        $createdByInvocation = $null -ne $stream
        $cleanupFailures = [Collections.Generic.List[string]]::new()
        if ($createdByInvocation) {
            try { $stream.Dispose() } catch {
                $cleanupFailures.Add("dispose: $($_.Exception.Message)")
            }
            try { [IO.File]::Delete($Path) } catch {
                $cleanupFailures.Add("delete: $($_.Exception.Message)")
            }
        }
        if ($cleanupFailures.Count -ne 0) {
            $cleanupMessage = $cleanupFailures -join "; "
            $failureMessage = "$Label publication failed and its invocation-owned partial " +
                "file could not be completely cleaned up. " +
                "Failure: $($publicationFailure.Exception.Message) Cleanup: $cleanupMessage"
            throw $failureMessage
        }
        throw $publicationFailure
    }
}

function Write-TessaraFinalizerExactData {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes
    )
    if ($Bytes.Length -gt $script:FinalizerMaximumFileBytes) {
        throw "Finalization publication exceeds the $($script:FinalizerMaximumFileBytes)-byte limit: $Path"
    }
    $resolved = [IO.Path]::GetFullPath($Path)
    $parent = Split-Path -Parent $resolved
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    $hash = Get-TessaraFinalizerSha256Bytes -Bytes $Bytes
    if (Test-Path -LiteralPath $resolved -PathType Leaf) {
        $existing = Read-TessaraFinalizerBoundedBytes -Path $resolved `
            -Label "Existing finalization target"
        if ([Convert]::ToBase64String($existing) -cne [Convert]::ToBase64String($Bytes)) {
            throw "Finalization target already exists with different bytes: $resolved"
        }
    } else {
        Write-TessaraFinalizerCreateNewBytes -Path $resolved -Bytes $Bytes `
            -Label "Finalization data"
    }
    [pscustomobject][ordered]@{ path = $resolved; sha256 = $hash }
}

function Write-TessaraFinalizerExactDigest {
    param(
        [Parameter(Mandatory)][string]$DataPath,
        [Parameter(Mandatory)][string]$Sha256
    )
    if ($Sha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw "Finalization digest is not canonical SHA-256."
    }
    $resolved = [IO.Path]::GetFullPath($DataPath)
    $sidecarPath = "$resolved.sha256"
    $sidecarBytes = [Text.UTF8Encoding]::new($false).GetBytes(
        "$Sha256  $([IO.Path]::GetFileName($resolved))`n"
    )
    if (Test-Path -LiteralPath $sidecarPath -PathType Leaf) {
        $existingSidecar = Read-TessaraFinalizerBoundedBytes -Path $sidecarPath `
            -Label "Existing finalization digest"
        if ([Convert]::ToBase64String($existingSidecar) -cne
            [Convert]::ToBase64String($sidecarBytes)) {
            throw "Finalization digest already exists with different bytes: $sidecarPath"
        }
    } else {
        Write-TessaraFinalizerCreateNewBytes -Path $sidecarPath -Bytes $sidecarBytes `
            -Label "Finalization digest"
    }
}

function Write-TessaraFinalizerExactPair {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes
    )
    $reference = Write-TessaraFinalizerExactData -Path $Path -Bytes $Bytes
    Write-TessaraFinalizerExactDigest -DataPath ([string]$reference.path) `
        -Sha256 ([string]$reference.sha256)
    $reference
}

function Assert-TessaraFinalizerAttestationSnapshot {
    param(
        [Parameter(Mandatory)]$Snapshot,
        [Parameter(Mandatory)][string]$ExpectedFingerprint,
        [Parameter(Mandatory)][string]$ExpectedAttemptId,
        [Parameter(Mandatory)]$CheckpointReference,
        [Parameter(Mandatory)]$ResultReference,
        [Parameter(Mandatory)]$IndexReference
    )
    $document = $Snapshot.document
    $requiredProperties = @(
        "schema_version", "contract", "state", "attempt_id",
        "finalization_platform_fingerprint", "execution_checkpoint",
        "result", "attempt_index"
    )
    if ($null -eq $document -or @($requiredProperties | Where-Object {
                $document.PSObject.Properties.Name -notcontains $_
            }).Count -ne 0) {
        throw "Finalization attestation is missing required fields: $($Snapshot.path)"
    }
    foreach ($referenceName in @("execution_checkpoint", "result", "attempt_index")) {
        $reference = $document.$referenceName
        if ($null -eq $reference -or
            $reference.PSObject.Properties.Name -notcontains "path" -or
            $reference.PSObject.Properties.Name -notcontains "sha256") {
            throw "Finalization attestation contains an invalid $referenceName reference: $($Snapshot.path)"
        }
    }
    if ([string]$document.schema_version -cne "1" -or
        [string]$document.contract -cne "tessara.validation.finalization-attestation" -or
        [string]$document.state -cne "complete" -or
        [string]$document.attempt_id -cne $ExpectedAttemptId -or
        [string]$document.finalization_platform_fingerprint -cne $ExpectedFingerprint -or
        [string]$document.execution_checkpoint.path -cne [string]$CheckpointReference.path -or
        [string]$document.execution_checkpoint.sha256 -cne [string]$CheckpointReference.sha256 -or
        [string]$document.result.path -cne [string]$ResultReference.path -or
        [string]$document.result.sha256 -cne [string]$ResultReference.sha256 -or
        [string]$document.attempt_index.path -cne [string]$IndexReference.path -or
        [string]$document.attempt_index.sha256 -cne [string]$IndexReference.sha256) {
        throw "Finalization attestation authentication failed: $($Snapshot.path)"
    }
}

function Assert-TessaraFinalizerExistingAttestations {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Fingerprints,
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)][string]$CurrentFingerprint,
        [Parameter(Mandatory)][string]$ExpectedAttemptId,
        [Parameter(Mandatory)]$CheckpointReference,
        [Parameter(Mandatory)]$ResultReference,
        [Parameter(Mandatory)]$IndexReference
    )
    foreach ($fingerprint in $Fingerprints) {
        $path = Join-Path $AttemptRoot "finalization-attestation.$fingerprint.json"
        if (-not (Test-Path -LiteralPath "$path.sha256" -PathType Leaf)) {
            if ($fingerprint -ceq $CurrentFingerprint) { continue }
            # A data-only older attestation never committed authority. While the
            # attempt lock is held, a corrected finalizer may discard exactly
            # that unauthenticated member and publish its own attestation.
            Remove-Item -LiteralPath $path -Force -ErrorAction Stop
            if (Test-Path -LiteralPath $path) {
                throw "Unable to remove unauthenticated prior finalization attestation data: $path"
            }
            continue
        }
        $snapshot = Read-TessaraFinalizerHashedJson -Path $path `
            -Label "Finalization attestation"
        Assert-TessaraFinalizerAttestationSnapshot -Snapshot $snapshot `
            -ExpectedFingerprint $fingerprint -ExpectedAttemptId $ExpectedAttemptId `
            -CheckpointReference $CheckpointReference -ResultReference $ResultReference `
            -IndexReference $IndexReference
    }
}

function Get-TessaraFinalizerArtifactInventoryBinding {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Artifacts,
        [Parameter(Mandatory)][string]$Label
    )
    $lines = [Collections.Generic.List[string]]::new()
    foreach ($artifact in $Artifacts) {
        if ($null -eq $artifact -or
            (@($artifact.PSObject.Properties.Name | Sort-Object) -join "`n") -cne
                "path`nsha256" -or
            [string]::IsNullOrWhiteSpace([string]$artifact.path) -or
            [string]$artifact.sha256 -cnotmatch '^[0-9a-f]{64}$') {
            throw "$Label contains an invalid artifact binding."
        }
        $lines.Add("$([string]$artifact.path)`n$([string]$artifact.sha256)")
    }
    @($lines) -join "`n--`n"
}

function Assert-TessaraFinalizerCommittedAttempt {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$ResultReference,
        [Parameter(Mandatory)][string]$ExpectedLaneId,
        [Parameter(Mandatory)][string]$ExpectedAttemptId,
        [Parameter(Mandatory)][string]$CurrentFinalizationFingerprint,
        [Parameter(Mandatory)][string]$Label
    )
    if ($null -eq $ResultReference -or
        $ResultReference.PSObject.Properties.Name -notcontains "path" -or
        $ResultReference.PSObject.Properties.Name -notcontains "sha256") {
        throw "$Label result reference is invalid."
    }
    $resultPath = [IO.Path]::GetFullPath([string]$ResultReference.path)
    if (-not (Test-TessaraFinalizerContainedPath -Root $EvidenceRoot -Path $resultPath)) {
        throw "$Label result is outside its evidence root."
    }
    $relative = [IO.Path]::GetRelativePath($EvidenceRoot, $resultPath).Replace('\', '/')
    $segments = @($relative -split '/')
    if ($segments.Count -ne 5 -or $segments[0] -cne "lanes" -or
        $segments[1] -cne $ExpectedLaneId -or $segments[2] -cne "attempts" -or
        $segments[3] -cne $ExpectedAttemptId -or
        $segments[3] -cnotmatch '^\d{8}T\d{9}Z-[0-9a-f]{8}$' -or
        $segments[4] -cne "lane-result.json") {
        throw "$Label result is not a canonical lane attempt."
    }
    $attemptRoot = Assert-TessaraFinalizerOrdinaryDirectoryChain `
        -Root $EvidenceRoot -Path (Split-Path -Parent $resultPath) `
        -Label "$Label attempt path"
    $revocationPath = Join-Path $attemptRoot "execution-revoked.json"
    if ((Test-Path -LiteralPath $revocationPath -PathType Leaf) -or
        (Test-Path -LiteralPath "$revocationPath.sha256" -PathType Leaf)) {
        throw "$Label was revoked before dependent finalization."
    }

    $resultSnapshot = Read-TessaraFinalizerHashedJson -Path $resultPath `
        -Label "$Label lane result"
    $indexSnapshot = Read-TessaraFinalizerHashedJson `
        -Path (Join-Path $attemptRoot "attempt-index.json") `
        -Label "$Label attempt index"
    $checkpointSnapshot = Read-TessaraFinalizerHashedJson `
        -Path (Join-Path $attemptRoot "execution-complete.json") `
        -Label "$Label execution checkpoint"
    $attestationPath = Join-Path $attemptRoot `
        "finalization-attestation.$CurrentFinalizationFingerprint.json"
    $attestationSnapshot = Read-TessaraFinalizerHashedJson -Path $attestationPath `
        -Label "$Label finalization attestation"
    $result = $resultSnapshot.document
    $index = $indexSnapshot.document
    $checkpoint = $checkpointSnapshot.document
    $requiredIndexProperties = @(
        "schema_version", "contract", "state", "attempt_id",
        "execution_checkpoint", "result", "files"
    )
    $requiredCheckpointProperties = @(
        "schema_version", "contract", "state", "result", "artifacts"
    )
    if ($null -eq $result -or $null -eq $index -or $null -eq $checkpoint -or
        @($requiredIndexProperties | Where-Object {
                $index.PSObject.Properties.Name -notcontains $_
            }).Count -ne 0 -or
        @($requiredCheckpointProperties | Where-Object {
                $checkpoint.PSObject.Properties.Name -notcontains $_
            }).Count -ne 0 -or
        [string]$ResultReference.sha256 -cne [string]$resultSnapshot.sha256 -or
        [string]$result.contract -cne "tessara.validation.lane-result" -or
        [string]$result.state -cne "passed" -or -not [bool]$result.authoritative -or
        [string]$result.lane_id -cne $ExpectedLaneId -or
        [string]$result.attempt_id -cne $ExpectedAttemptId -or
        [string]$index.schema_version -cne "1" -or
        [string]$index.contract -cne "tessara.validation.attempt-index" -or
        [string]$index.state -cne "passed" -or
        [string]$index.attempt_id -cne $ExpectedAttemptId -or
        $null -eq $index.result -or
        [string]$index.result.path -cne [string]$resultSnapshot.path -or
        [string]$index.result.sha256 -cne [string]$resultSnapshot.sha256 -or
        $null -eq $index.execution_checkpoint -or
        [string]$index.execution_checkpoint.path -cne [string]$checkpointSnapshot.path -or
        [string]$index.execution_checkpoint.sha256 -cne
            [string]$checkpointSnapshot.sha256 -or
        [string]$checkpoint.schema_version -cne "1" -or
        [string]$checkpoint.contract -cne "tessara.validation.execution-checkpoint" -or
        [string]$checkpoint.state -cne "execution_complete" -or
        $null -eq $checkpoint.result -or
        [string]$checkpoint.result.contract -cne "tessara.validation.lane-result" -or
        [string]$checkpoint.result.state -cne "passed" -or
        [bool]$checkpoint.result.authoritative -or
        [string]$checkpoint.result.lane_id -cne $ExpectedLaneId -or
        [string]$checkpoint.result.attempt_id -cne $ExpectedAttemptId -or
        (Get-TessaraFinalizerArtifactInventoryBinding -Artifacts @($index.files) `
            -Label "$Label attempt index") -cne
            (Get-TessaraFinalizerArtifactInventoryBinding `
                -Artifacts @($checkpoint.artifacts) -Label "$Label execution checkpoint")) {
        throw "$Label committed-attempt authentication failed."
    }
    Assert-TessaraFinalizerAttestationSnapshot -Snapshot $attestationSnapshot `
        -ExpectedFingerprint $CurrentFinalizationFingerprint `
        -ExpectedAttemptId $ExpectedAttemptId -CheckpointReference $checkpointSnapshot `
        -ResultReference $resultSnapshot -IndexReference $indexSnapshot
    if ((Test-Path -LiteralPath $revocationPath -PathType Leaf) -or
        (Test-Path -LiteralPath "$revocationPath.sha256" -PathType Leaf)) {
        throw "$Label was revoked during dependent finalization."
    }
    $resultSnapshot
}

function Assert-TessaraFinalizerExecutionIntegritySnapshot {
    param(
        [Parameter(Mandatory)]$Snapshot,
        [Parameter(Mandatory)]$CheckpointReference,
        [Parameter(Mandatory)]$Result,
        [Parameter(Mandatory)][string]$ExpectedLaneId,
        [Parameter(Mandatory)][string]$ExpectedAttemptId,
        [Parameter(Mandatory)][string]$CandidateFingerprint,
        [Parameter(Mandatory)][string]$LaneCompatibilityFingerprint,
        [Parameter(Mandatory)][string]$ExecutionPlatformFingerprint,
        [Parameter(Mandatory)][string]$EvidenceRootFingerprint
    )
    $document = $Snapshot.document
    $requiredProperties = @(
        "schema_version", "contract", "state", "lane_id", "attempt_id",
        "candidate_fingerprint", "candidate_source_identity",
        "lane_compatibility_fingerprint", "compatibility_plan_fingerprint",
        "platform_execution_fingerprint", "evidence_root_fingerprint",
        "execution_checkpoint"
    ) | Sort-Object -CaseSensitive
    $actualProperties = if ($null -eq $document) { @() } else {
        @($document.PSObject.Properties.Name | Sort-Object -CaseSensitive)
    }
    if (($actualProperties -join "`n") -cne ($requiredProperties -join "`n") -or
        $Result.PSObject.Properties.Name -notcontains "source_integrity_failure" -or
        $null -ne $Result.source_integrity_failure -or
        [string]$document.schema_version -cne "1" -or
        [string]$document.contract -cne
            "tessara.validation.execution-integrity-commit" -or
        [string]$document.state -cne "verified" -or
        [string]$document.lane_id -cne $ExpectedLaneId -or
        [string]$document.attempt_id -cne $ExpectedAttemptId -or
        [string]$document.candidate_fingerprint -cne $CandidateFingerprint -or
        ($document.candidate_source_identity | ConvertTo-Json -Depth 50 -Compress) -cne
            ($Result.candidate_source_identity | ConvertTo-Json -Depth 50 -Compress) -or
        [string]$document.lane_compatibility_fingerprint -cne
            $LaneCompatibilityFingerprint -or
        [string]$document.compatibility_plan_fingerprint -cne
            [string]$Result.compatibility_plan_fingerprint -or
        [string]$document.platform_execution_fingerprint -cne
            $ExecutionPlatformFingerprint -or
        [string]$document.evidence_root_fingerprint -cne $EvidenceRootFingerprint -or
        $null -eq $document.execution_checkpoint -or
        [string]$document.execution_checkpoint.path -cne
            [string]$CheckpointReference.path -or
        [string]$document.execution_checkpoint.sha256 -cne
            [string]$CheckpointReference.sha256) {
        throw "Execution integrity commit does not authenticate the exact checkpoint and execution identity."
    }
}

function Complete-TessaraValidationEvidence {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$CheckpointPath,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$LaneId,
        [Parameter(Mandatory)][string]$CandidateFingerprint,
        [Parameter(Mandatory)]$LaneIdentity,
        [Parameter(Mandatory)][string]$ExecutionPlatformFingerprint,
        [Parameter(Mandatory)][string]$FinalizationPlatformFingerprint
    )
    if ($FinalizationPlatformFingerprint -cnotmatch '^[0-9a-f]{64}$') {
        throw "Finalization platform fingerprint is not canonical SHA-256."
    }
    $evidence = [IO.Path]::GetFullPath($EvidenceRoot)
    $evidenceRootFingerprint = Get-TessaraFinalizerSha256Bytes `
        -Bytes ([Text.UTF8Encoding]::new($false).GetBytes($evidence))
    $checkpointResolved = [IO.Path]::GetFullPath($CheckpointPath)
    $relative = [IO.Path]::GetRelativePath($evidence, $checkpointResolved).Replace('\', '/')
    $segments = @($relative -split '/')
    if ($segments.Count -ne 5 -or $segments[0] -cne "lanes" -or
        $segments[1] -cne $LaneId -or $segments[2] -cne "attempts" -or
        $segments[3] -cnotmatch '^\d{8}T\d{9}Z-[0-9a-f]{8}$' -or
        $segments[4] -cne "execution-complete.json") {
        throw "Execution checkpoint is not the canonical attempt checkpoint for lane '$LaneId'."
    }
    $attemptRoot = [IO.Path]::GetFullPath((Split-Path -Parent $checkpointResolved))
    $attemptRoot = Assert-TessaraFinalizerOrdinaryDirectoryChain -Root $evidence `
        -Path $attemptRoot -Label "Finalization attempt path"
    $lockPath = "$attemptRoot.finalize.lock"
    if (Test-Path -LiteralPath $lockPath) {
        $lockItem = Get-Item -Force -LiteralPath $lockPath
        Assert-TessaraFinalizerOrdinaryFileItem -Item $lockItem `
            -Label "Finalization lock"
    }
    $lock = [IO.File]::Open($lockPath, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    try {
        $attemptRoot = Assert-TessaraFinalizerOrdinaryDirectoryChain -Root $evidence `
            -Path $attemptRoot -Label "Finalization attempt path"
        $initialInventory = @(Get-TessaraFinalizerAttemptInventory -AttemptRoot $attemptRoot)
        $checkpoint = Read-TessaraFinalizerHashedJson -Path $checkpointResolved `
            -Label "Execution checkpoint"
        if ([string]$checkpoint.path -cne $checkpointResolved) {
            throw "Execution checkpoint path changed during finalization."
        }
        $checkpointDocument = $checkpoint.document
        if ($checkpointDocument.PSObject.Properties.Name -notcontains "result" -or
            $checkpointDocument.PSObject.Properties.Name -notcontains "artifacts") {
            throw "Execution checkpoint is missing its result or artifact inventory."
        }
        $result = $checkpointDocument.result
        if ([int]$checkpointDocument.schema_version -ne 1 -or
            [string]$checkpointDocument.contract -cne "tessara.validation.execution-checkpoint" -or
            [string]$checkpointDocument.state -cne "execution_complete" -or
            [string]$result.contract -cne "tessara.validation.lane-result" -or
            [string]$result.lane_id -cne $LaneId -or
            [string]$result.attempt_id -cne [string]$segments[3] -or
            [string]$result.candidate_fingerprint -cne $CandidateFingerprint -or
            [string]$result.adapter_fingerprint -cne [string]$LaneIdentity.adapter_fingerprint -or
            [string]$result.acceptance_fingerprint -cne [string]$LaneIdentity.acceptance_fingerprint -or
            [string]$result.lane_compatibility_fingerprint -cne
                [string]$LaneIdentity.compatibility_fingerprint -or
            [string]$result.platform_execution_fingerprint -cne $ExecutionPlatformFingerprint -or
            [string]$result.evidence_root_fingerprint -cne $evidenceRootFingerprint -or
            -not [bool]$result.immutable_raw_results_complete -or
            [bool]$result.authoritative) {
            throw "Execution checkpoint does not bind the current lane execution identity."
        }
        $integrityPath = Join-Path $attemptRoot "execution-integrity-verified.json"
        $integrityCommit = Read-TessaraFinalizerHashedJson -Path $integrityPath `
            -Label "Execution integrity commit"
        Assert-TessaraFinalizerExecutionIntegritySnapshot -Snapshot $integrityCommit `
            -CheckpointReference $checkpoint -Result $result `
            -ExpectedLaneId $LaneId -ExpectedAttemptId ([string]$segments[3]) `
            -CandidateFingerprint $CandidateFingerprint `
            -LaneCompatibilityFingerprint ([string]$LaneIdentity.compatibility_fingerprint) `
            -ExecutionPlatformFingerprint $ExecutionPlatformFingerprint `
            -EvidenceRootFingerprint $evidenceRootFingerprint
        $inventoryState = Assert-TessaraFinalizerAttemptInventory `
            -Inventory $initialInventory -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot `
            -CurrentFinalizationPlatformFingerprint $FinalizationPlatformFingerprint
        $artifactPaths = $inventoryState.artifact_paths

        $revocationPath = Join-Path $attemptRoot "execution-revoked.json"
        if ((Test-Path -LiteralPath $revocationPath -PathType Leaf) -or
            (Test-Path -LiteralPath "$revocationPath.sha256" -PathType Leaf)) {
            throw "Execution checkpoint was revoked and cannot be finalized: $revocationPath"
        }
        Assert-TessaraFinalizerArtifactDigests -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot
        $prepublicationIntegrity = Read-TessaraFinalizerHashedJson `
            -Path $integrityPath -Label "Execution integrity commit"
        Assert-TessaraFinalizerExecutionIntegritySnapshot `
            -Snapshot $prepublicationIntegrity -CheckpointReference $checkpoint `
            -Result $result -ExpectedLaneId $LaneId `
            -ExpectedAttemptId ([string]$segments[3]) `
            -CandidateFingerprint $CandidateFingerprint `
            -LaneCompatibilityFingerprint ([string]$LaneIdentity.compatibility_fingerprint) `
            -ExecutionPlatformFingerprint $ExecutionPlatformFingerprint `
            -EvidenceRootFingerprint $evidenceRootFingerprint
        if ([string]$prepublicationIntegrity.sha256 -cne [string]$integrityCommit.sha256) {
            throw "Execution integrity commit changed during finalization."
        }
        if (-not $artifactPaths.Contains("attempt-intent.json")) {
            throw "Execution checkpoint omits its attempt-intent evidence."
        }
        $intent = Read-TessaraFinalizerHashedJson `
            -Path (Join-Path $attemptRoot "attempt-intent.json") -Label "Attempt intent"
        if ([string]$intent.document.attempt_id -cne [string]$result.attempt_id -or
            [string]$intent.document.lane_compatibility_fingerprint -cne
                [string]$result.lane_compatibility_fingerprint -or
            [string]$intent.document.evidence_root_fingerprint -cne $evidenceRootFingerprint) {
            throw "Execution checkpoint does not match its attempt-intent identity."
        }
        $startPath = Join-Path $attemptRoot "attempt-start.json"
        if ((Test-Path -LiteralPath $startPath -PathType Leaf) -or
            (Test-Path -LiteralPath "$startPath.sha256" -PathType Leaf)) {
            if (-not $artifactPaths.Contains("attempt-start.json")) {
                throw "Execution checkpoint omits its attempt-start evidence."
            }
            $start = Read-TessaraFinalizerHashedJson -Path $startPath -Label "Attempt start"
            if ([string]$start.document.attempt_id -cne [string]$result.attempt_id -or
                [string]$start.document.lane_compatibility_fingerprint -cne
                    [string]$result.lane_compatibility_fingerprint -or
                [string]$start.document.evidence_root_fingerprint -cne $evidenceRootFingerprint) {
                throw "Execution checkpoint does not match its attempt-start identity."
            }
        }
        foreach ($action in @($result.actions)) {
            foreach ($streamName in @("stdout", "stderr")) {
                $reference = $action.$streamName
                $relativeLog = [IO.Path]::GetRelativePath(
                    $attemptRoot, [string]$reference.path
                ).Replace('\', '/')
                if (-not $artifactPaths.Contains($relativeLog) -or
                    @($checkpointDocument.artifacts | Where-Object {
                        [string]$_.path -ceq $relativeLog -and
                        [string]$_.sha256 -ceq [string]$reference.sha256
                    }).Count -ne 1) {
                    throw "Execution checkpoint does not bind action '$($action.id)' $streamName evidence."
                }
            }
        }
        foreach ($prerequisite in @($result.prerequisite_results)) {
            $reference = $prerequisite.result
            $snapshot = Assert-TessaraFinalizerCommittedAttempt `
                -EvidenceRoot $evidence -ResultReference $reference `
                -ExpectedLaneId ([string]$prerequisite.lane_id) `
                -ExpectedAttemptId ([string]$prerequisite.attempt_id) `
                -CurrentFinalizationFingerprint $FinalizationPlatformFingerprint `
                -Label "Execution checkpoint prerequisite"
            if ([string]$snapshot.document.evidence_root_fingerprint -cne
                    $evidenceRootFingerprint -or
                [string]$snapshot.document.lane_compatibility_fingerprint -cne
                    [string]$prerequisite.compatibility_fingerprint) {
                throw "Execution checkpoint prerequisite result authentication failed."
            }
        }
        if ($null -ne $result.topology_claim) {
            $claimPath = [string]$result.topology_claim.path
            if (-not (Test-TessaraFinalizerContainedPath -Root $evidence -Path $claimPath)) {
                throw "Execution checkpoint contains a topology claim outside its evidence root."
            }
            $null = Assert-TessaraFinalizerOrdinaryDirectoryChain -Root $evidence `
                -Path (Split-Path -Parent $claimPath) -Label "Topology claim path"
            $claim = Read-TessaraFinalizerHashedJson -Path $claimPath `
                -Label "Topology consumption claim"
            $claimRelative = [IO.Path]::GetRelativePath(
                $evidence, [string]$claim.path
            ).Replace('\', '/')
            if ($claimRelative -cnotmatch '^topology-claims/[0-9a-f]{64}\.json$' -or
                [string]$claim.sha256 -cne [string]$result.topology_claim.sha256 -or
                [string]$claim.document.contract -cne "tessara.validation.topology-consumption-claim" -or
                [string]$claim.document.to_lane -cne $LaneId -or
                [string]$claim.document.consumer_attempt_id -cne [string]$result.attempt_id -or
                [string]$claim.document.evidence_root_fingerprint -cne $evidenceRootFingerprint) {
                throw "Execution checkpoint topology claim authentication failed."
            }
        }
        if ([string]$result.cleanup_restoration.mode -ceq "retained-for-handoff") {
            $receipt = $result.cleanup_restoration.receipt
            $receiptPath = [IO.Path]::GetFullPath([string]$receipt.path)
            $expectedReceiptPath = [IO.Path]::GetFullPath(
                (Join-Path $attemptRoot "topology-receipt.json")
            )
            if ($receiptPath -cne $expectedReceiptPath) {
                throw "Execution checkpoint topology receipt is outside its canonical attempt path."
            }
            $receiptSnapshot = Read-TessaraFinalizerHashedJson -Path $receiptPath `
                -Label "Retained topology receipt"
            if ([string]$receiptSnapshot.sha256 -cne [string]$receipt.sha256 -or
                [string]$receiptSnapshot.document.evidence_root_fingerprint -cne $evidenceRootFingerprint -or
                [string]$receiptSnapshot.document.source_attempt_id -cne [string]$result.attempt_id -or
                -not $artifactPaths.Contains("topology-receipt.json") -or
                @($checkpointDocument.artifacts | Where-Object {
                    [string]$_.path -ceq "topology-receipt.json" -and
                    [string]$_.sha256 -ceq [string]$receipt.sha256
                }).Count -ne 1) {
                throw "Execution checkpoint topology receipt authentication failed."
            }
        }
        if ([string]$result.cleanup_restoration.environment_restoration.state -cne "passed") {
            throw "Execution checkpoint cannot finalize without passing environment containment."
        }

        $prepublicationInventory = @(
            Get-TessaraFinalizerAttemptInventory -AttemptRoot $attemptRoot
        )
        $null = Assert-TessaraFinalizerAttemptInventory `
            -Inventory $prepublicationInventory -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot `
            -CurrentFinalizationPlatformFingerprint $FinalizationPlatformFingerprint
        Assert-TessaraFinalizerArtifactDigests -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot

        $result.authoritative = $true
        $resultBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (($result | ConvertTo-Json -Depth 100 -Compress) + "`n")
        )
        $resultRef = Write-TessaraFinalizerExactPair `
            -Path (Join-Path $attemptRoot "lane-result.json") -Bytes $resultBytes
        $postResultInventory = @(Get-TessaraFinalizerAttemptInventory -AttemptRoot $attemptRoot)
        $null = Assert-TessaraFinalizerAttemptInventory `
            -Inventory $postResultInventory -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot `
            -CurrentFinalizationPlatformFingerprint $FinalizationPlatformFingerprint
        $index = [pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.attempt-index"
            state = [string]$result.state
            attempt_id = [string]$result.attempt_id
            execution_checkpoint = [pscustomobject][ordered]@{
                path = [string]$checkpoint.path
                sha256 = [string]$checkpoint.sha256
            }
            result = $resultRef
            files = @($checkpointDocument.artifacts)
        }
        $indexBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (($index | ConvertTo-Json -Depth 100 -Compress) + "`n")
        )
        $indexRef = Write-TessaraFinalizerExactPair `
            -Path (Join-Path $attemptRoot "attempt-index.json") -Bytes $indexBytes
        $postIndexInventory = @(
            Get-TessaraFinalizerAttemptInventory -AttemptRoot $attemptRoot
        )
        $postIndexInventoryState = Assert-TessaraFinalizerAttemptInventory `
            -Inventory $postIndexInventory -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot `
            -CurrentFinalizationPlatformFingerprint $FinalizationPlatformFingerprint
        Assert-TessaraFinalizerExistingAttestations `
            -Fingerprints @($postIndexInventoryState.attestation_fingerprints) `
            -AttemptRoot $attemptRoot -CurrentFingerprint $FinalizationPlatformFingerprint `
            -ExpectedAttemptId ([string]$result.attempt_id) `
            -CheckpointReference $checkpoint -ResultReference $resultRef `
            -IndexReference $indexRef

        $attestation = [pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.finalization-attestation"
            state = "complete"
            attempt_id = [string]$result.attempt_id
            finalization_platform_fingerprint = $FinalizationPlatformFingerprint
            execution_checkpoint = [pscustomobject][ordered]@{
                path = [string]$checkpoint.path
                sha256 = [string]$checkpoint.sha256
            }
            result = $resultRef
            attempt_index = $indexRef
        }
        $attestationBytes = [Text.UTF8Encoding]::new($false).GetBytes(
            (($attestation | ConvertTo-Json -Depth 100 -Compress) + "`n")
        )
        $attestationRef = Write-TessaraFinalizerExactData `
            -Path (Join-Path $attemptRoot `
                "finalization-attestation.$FinalizationPlatformFingerprint.json") `
            -Bytes $attestationBytes
        $finalInventory = @(Get-TessaraFinalizerAttemptInventory -AttemptRoot $attemptRoot)
        $null = Assert-TessaraFinalizerAttemptInventory `
            -Inventory $finalInventory -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot `
            -CurrentFinalizationPlatformFingerprint $FinalizationPlatformFingerprint
        $currentAttestationBytes = Read-TessaraFinalizerBoundedBytes `
            -Path ([string]$attestationRef.path) -Label "Finalization attestation"
        if ([Convert]::ToBase64String($currentAttestationBytes) -cne
            [Convert]::ToBase64String($attestationBytes) -or
            (Get-TessaraFinalizerSha256Bytes -Bytes $currentAttestationBytes) -cne
                [string]$attestationRef.sha256) {
            throw "Finalization attestation changed before its commit sidecar."
        }
        foreach ($prerequisite in @($result.prerequisite_results)) {
            $null = Assert-TessaraFinalizerCommittedAttempt `
                -EvidenceRoot $evidence -ResultReference $prerequisite.result `
                -ExpectedLaneId ([string]$prerequisite.lane_id) `
                -ExpectedAttemptId ([string]$prerequisite.attempt_id) `
                -CurrentFinalizationFingerprint $FinalizationPlatformFingerprint `
                -Label "Final prerequisite recheck"
        }
        Assert-TessaraFinalizerArtifactDigests -CheckpointDocument $checkpointDocument `
            -AttemptRoot $attemptRoot
        $finalIntegrity = Read-TessaraFinalizerHashedJson -Path $integrityPath `
            -Label "Execution integrity commit"
        Assert-TessaraFinalizerExecutionIntegritySnapshot -Snapshot $finalIntegrity `
            -CheckpointReference $checkpoint -Result $result `
            -ExpectedLaneId $LaneId -ExpectedAttemptId ([string]$segments[3]) `
            -CandidateFingerprint $CandidateFingerprint `
            -LaneCompatibilityFingerprint ([string]$LaneIdentity.compatibility_fingerprint) `
            -ExecutionPlatformFingerprint $ExecutionPlatformFingerprint `
            -EvidenceRootFingerprint $evidenceRootFingerprint
        if ([string]$finalIntegrity.sha256 -cne [string]$integrityCommit.sha256) {
            throw "Execution integrity commit changed before finalization commit."
        }
        if ((Test-Path -LiteralPath $revocationPath -PathType Leaf) -or
            (Test-Path -LiteralPath "$revocationPath.sha256" -PathType Leaf)) {
            throw "Execution checkpoint was revoked before finalization commit: $revocationPath"
        }
        $result | Add-Member -NotePropertyName evidence_path `
            -NotePropertyValue $resultRef.path -Force
        # This sidecar is the only final completion commit. Result and index
        # pairs remain inert until consumers authenticate this current-fingerprint pair.
        Write-TessaraFinalizerExactDigest -DataPath ([string]$attestationRef.path) `
            -Sha256 ([string]$attestationRef.sha256)
    } finally {
        try { $lock.Dispose() } catch { }
        Remove-Item -LiteralPath $lockPath -Force -ErrorAction SilentlyContinue
    }
    $result
}

function Read-TessaraFinalizerHashedJsonOrText {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Label)
    $resolved = [IO.Path]::GetFullPath($Path)
    $sidecarPath = "$resolved.sha256"
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
        throw "$Label pair is missing: $resolved"
    }
    $bytes = Read-TessaraFinalizerBoundedBytes -Path $resolved -Label $Label
    $hash = Get-TessaraFinalizerSha256Bytes -Bytes $bytes
    $sidecarBytes = Read-TessaraFinalizerBoundedBytes -Path $sidecarPath `
        -Label "$Label digest sidecar"
    try {
        $sidecar = [Text.UTF8Encoding]::new($false, $true).GetString($sidecarBytes)
    } catch {
        throw "$Label digest sidecar is not strict UTF-8."
    }
    if ($sidecar -cne "$hash  $([IO.Path]::GetFileName($resolved))`n") {
        throw "$Label digest sidecar is invalid."
    }
    [pscustomobject][ordered]@{ path = $resolved; sha256 = $hash }
}

Export-ModuleMember -Function @(
    "Get-TessaraValidationFinalizerIdentity",
    "Complete-TessaraValidationEvidence"
)

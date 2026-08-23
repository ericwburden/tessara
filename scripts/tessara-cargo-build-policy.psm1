Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:CargoBuildPolicyContract = "tessara.validation.cargo-build-policy"
$script:CargoBuildPolicyRelease = "1.0.0"
$script:CargoBuildPolicyMarkerName = ".tessara-cargo-target.json"
$script:CargoBuildPolicyModulePath = [IO.Path]::GetFullPath($PSCommandPath)
$script:CargoBuildPolicyModuleSha256AtImport = (
    Get-FileHash -Algorithm SHA256 -LiteralPath $script:CargoBuildPolicyModulePath
).Hash.ToLowerInvariant()
$script:ActiveCargoBuildTargets = @{}
$script:DefaultCargoFreeSpaceProbe = {
    param([Parameter(Mandatory)][string]$Path)

    $root = [IO.Path]::GetPathRoot([IO.Path]::GetFullPath($Path))
    return [IO.DriveInfo]::new($root).AvailableFreeSpace
}
$script:CargoFreeSpaceProbe = $script:DefaultCargoFreeSpaceProbe

function Get-TessaraCargoBuildPolicyIdentity {
    [CmdletBinding()]
    param()

    [pscustomobject][ordered]@{
        schema_version = 1
        contract = $script:CargoBuildPolicyContract
        release_version = $script:CargoBuildPolicyRelease
        module_sha256 = $script:CargoBuildPolicyModuleSha256AtImport
    }
}

function Get-TessaraCargoBuildPolicy {
    [CmdletBinding()]
    param(
        [ValidateSet("Development", "Validation", "Diagnostic")]
        [string]$Mode = "Development"
    )

    $identity = Get-TessaraCargoBuildPolicyIdentity
    switch ($Mode) {
        "Development" {
            return [pscustomobject][ordered]@{
                mode = $Mode
                isolated_target = $false
                incremental = $true
                test_debug = "1"
                automatic_cleanup = $false
                release_version = $identity.release_version
                policy_fingerprint = $identity.module_sha256
            }
        }
        "Validation" {
            return [pscustomobject][ordered]@{
                mode = $Mode
                isolated_target = $true
                incremental = $false
                test_debug = "0"
                automatic_cleanup = $true
                release_version = $identity.release_version
                policy_fingerprint = $identity.module_sha256
            }
        }
        "Diagnostic" {
            return [pscustomobject][ordered]@{
                mode = $Mode
                isolated_target = $true
                incremental = $false
                test_debug = "1"
                automatic_cleanup = $false
                release_version = $identity.release_version
                policy_fingerprint = $identity.module_sha256
            }
        }
    }
}

function Get-TessaraFreeSpaceBytes {
    param([Parameter(Mandatory)][string]$Path)

    return & $script:CargoFreeSpaceProbe $Path
}

function Get-TessaraPathComparison {
    if ([Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
            [Runtime.InteropServices.OSPlatform]::Windows
        )) {
        return [StringComparison]::OrdinalIgnoreCase
    }
    return [StringComparison]::Ordinal
}

function Assert-TessaraPathHasNoReparsePoints {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    $resolvedPath = [IO.Path]::GetFullPath($Path)
    $pathRoot = [IO.Path]::GetPathRoot($resolvedPath)
    if ([string]::IsNullOrWhiteSpace($pathRoot)) {
        throw "$Label does not have a filesystem root: $resolvedPath"
    }

    $pathsToInspect = [Collections.Generic.List[string]]::new()
    $pathsToInspect.Add($pathRoot)
    $relativePath = [IO.Path]::GetRelativePath($pathRoot, $resolvedPath)
    [char[]]$pathSeparators = @(
        [IO.Path]::DirectorySeparatorChar,
        [IO.Path]::AltDirectorySeparatorChar
    )
    $currentPath = $pathRoot
    foreach ($segment in @(
            $relativePath.Split(
                $pathSeparators,
                [StringSplitOptions]::RemoveEmptyEntries
            )
        )) {
        if ($segment -eq ".") {
            continue
        }
        $currentPath = Join-Path $currentPath $segment
        $pathsToInspect.Add($currentPath)
    }

    foreach ($candidatePath in $pathsToInspect) {
        if (-not (Test-Path -LiteralPath $candidatePath)) {
            continue
        }
        $candidateItem = Get-Item -LiteralPath $candidatePath -Force
        if ($candidateItem.Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw "$Label traverses a reparse point: $candidatePath"
        }
    }
}

function Assert-TessaraOrdinaryDirectory {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-TessaraPathHasNoReparsePoints -Path $Path -Label $Label
    $item = Get-Item -LiteralPath $Path -Force
    if (-not $item.PSIsContainer -or
        ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "$Label is not an ordinary directory: $Path"
    }
}

function Assert-TessaraOrdinaryFile {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    Assert-TessaraPathHasNoReparsePoints -Path $Path -Label $Label
    $item = Get-Item -LiteralPath $Path -Force
    if ($item.PSIsContainer -or
        ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "$Label is not an ordinary file: $Path"
    }
}

function Assert-TessaraPathWithinRoot {
    param(
        [Parameter(Mandatory)][string]$RootPath,
        [Parameter(Mandatory)][string]$ChildPath,
        [Parameter(Mandatory)][string]$Label
    )

    $resolvedRoot = [IO.Path]::GetFullPath($RootPath)
    $resolvedChild = [IO.Path]::GetFullPath($ChildPath)
    $relativePath = [IO.Path]::GetRelativePath($resolvedRoot, $resolvedChild)
    $parentPrefix = "..$([IO.Path]::DirectorySeparatorChar)"
    $alternateParentPrefix = "..$([IO.Path]::AltDirectorySeparatorChar)"
    if ($relativePath -eq "." -or
        $relativePath -eq ".." -or
        [IO.Path]::IsPathRooted($relativePath) -or
        $relativePath.StartsWith($parentPrefix, [StringComparison]::Ordinal) -or
        $relativePath.StartsWith($alternateParentPrefix, [StringComparison]::Ordinal)) {
        throw "$Label is outside its declared root: $resolvedChild"
    }
}

function Get-TessaraRequiredStateProperty {
    param(
        [Parameter(Mandatory)][psobject]$State,
        [Parameter(Mandatory)][string]$Name
    )

    $property = $State.PSObject.Properties[$Name]
    if ($null -eq $property) {
        throw "Cargo target state is missing required property '$Name'."
    }
    return $property.Value
}

function Assert-TessaraStateMatchesLease {
    param(
        [Parameter(Mandatory)][psobject]$State,
        [Parameter(Mandatory)][psobject]$Lease
    )

    foreach ($propertyName in @(
            "lease_id",
            "target_id",
            "mode",
            "lane",
            "target_directory",
            "target_root",
            "repository_root",
            "cargo_executable_path",
            "cargo_executable_sha256",
            "marker_sha256",
            "policy_release",
            "policy_fingerprint"
        )) {
        $actual = [string](Get-TessaraRequiredStateProperty -State $State -Name $propertyName)
        $expected = [string]$Lease.$propertyName
        $comparison = if ($propertyName -in @(
                "target_directory",
                "target_root",
                "repository_root"
            )) {
            Get-TessaraPathComparison
        } else {
            [StringComparison]::Ordinal
        }
        if (-not $actual.Equals($expected, $comparison)) {
            throw "Cargo target state property '$propertyName' does not match its active lease."
        }
    }

    $actualRetain = [bool](Get-TessaraRequiredStateProperty -State $State -Name "retain_target")
    if ($actualRetain -ne [bool]$Lease.retain_target) {
        throw "Cargo target state property 'retain_target' does not match its active lease."
    }
}

function Assert-TessaraCargoTargetMarker {
    param([Parameter(Mandatory)][psobject]$Lease)

    $markerPath = Join-Path $Lease.target_directory $script:CargoBuildPolicyMarkerName
    if (-not (Test-Path -LiteralPath $markerPath -PathType Leaf)) {
        throw "Cargo target marker is missing: $markerPath"
    }
    Assert-TessaraOrdinaryFile -Path $markerPath -Label "Cargo target marker"
    $markerHash = (
        Get-FileHash -Algorithm SHA256 -LiteralPath $markerPath
    ).Hash.ToLowerInvariant()
    if ($markerHash -cne $Lease.marker_sha256) {
        throw "Cargo target marker digest does not match its active lease: $markerPath"
    }

    try {
        $marker = Get-Content -Raw -LiteralPath $markerPath | ConvertFrom-Json
    } catch {
        throw "Cargo target marker is not valid JSON: $markerPath"
    }
    $expectedMarker = [ordered]@{
        schema_version = 1
        contract = $script:CargoBuildPolicyContract
        policy_release = $Lease.policy_release
        policy_fingerprint = $Lease.policy_fingerprint
        lease_id = $Lease.lease_id
        target_id = $Lease.target_id
        lane = $Lease.lane
        mode = $Lease.mode
        target_root = $Lease.target_root
        target_directory = $Lease.target_directory
        repository_root = $Lease.repository_root
        retain_target = [bool]$Lease.retain_target
    }
    foreach ($propertyName in $expectedMarker.Keys) {
        $property = $marker.PSObject.Properties[$propertyName]
        if ($null -eq $property -or $property.Value -cne $expectedMarker[$propertyName]) {
            throw "Cargo target marker property '$propertyName' does not match its active lease."
        }
    }
    $unexpectedProperties = @(
        $marker.PSObject.Properties.Name |
            Where-Object { $_ -notin $expectedMarker.Keys }
    )
    if ($unexpectedProperties.Count -ne 0) {
        throw "Cargo target marker contains unexpected properties: $($unexpectedProperties -join ', ')"
    }
}

function Enter-TessaraCargoBuildPolicy {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [ValidatePattern("^[a-z0-9][a-z0-9-]{0,63}$")]
        [string]$Lane,

        [Parameter(Mandatory)]
        [string]$RepositoryRoot,

        [ValidateSet("Validation", "Diagnostic")]
        [string]$Mode = "Validation",

        [string]$TargetRoot = (Join-Path ([IO.Path]::GetTempPath()) "tessara-cargo-targets"),

        [ValidateRange(0, 1024)]
        [int]$MinimumFreeSpaceGB = 20,

        [string]$CargoExecutablePath,

        [switch]$RetainTarget
    )

    $policy = Get-TessaraCargoBuildPolicy -Mode $Mode
    if ($RetainTarget -and $Mode -ne "Diagnostic") {
        throw "Retaining a Cargo target is allowed only in explicit Diagnostic mode."
    }
    if ($Mode -eq "Diagnostic" -and -not $RetainTarget) {
        throw "Diagnostic mode requires explicit target retention."
    }
    if ($script:ActiveCargoBuildTargets.Count -ne 0) {
        throw "A Cargo build policy lease is already active in this PowerShell session."
    }

    $resolvedRepositoryRoot = [IO.Path]::GetFullPath($RepositoryRoot)
    Assert-TessaraOrdinaryDirectory `
        -Path $resolvedRepositoryRoot `
        -Label "Cargo policy repository root"
    $manifestPath = Join-Path $resolvedRepositoryRoot "Cargo.toml"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Cargo policy repository root does not contain Cargo.toml: $resolvedRepositoryRoot"
    }
    Assert-TessaraOrdinaryFile -Path $manifestPath -Label "Cargo policy manifest"
    $resolvedCargoExecutable = if ([string]::IsNullOrWhiteSpace($CargoExecutablePath)) {
        $cargoCommand = @(Get-Command cargo -CommandType Application -All -ErrorAction Stop) |
            Select-Object -First 1
        if ($null -eq $cargoCommand) {
            throw "Cargo build policy requires one directly executable Cargo program."
        }
        [IO.Path]::GetFullPath([string]$cargoCommand.Source)
    } else {
        [IO.Path]::GetFullPath($CargoExecutablePath)
    }
    Assert-TessaraOrdinaryFile `
        -Path $resolvedCargoExecutable -Label "Cargo policy executable"
    $cargoExecutableSha256 = (
        Get-FileHash -Algorithm SHA256 -LiteralPath $resolvedCargoExecutable
    ).Hash.ToLowerInvariant()

    $resolvedTargetRoot = [IO.Path]::GetFullPath($TargetRoot)
    New-Item -ItemType Directory -Path $resolvedTargetRoot -Force | Out-Null
    Assert-TessaraOrdinaryDirectory `
        -Path $resolvedTargetRoot `
        -Label "Cargo target root"
    $freeSpaceBytes = Get-TessaraFreeSpaceBytes -Path $resolvedTargetRoot
    $minimumFreeSpaceBytes = $MinimumFreeSpaceGB * 1GB
    if ($freeSpaceBytes -lt $minimumFreeSpaceBytes) {
        throw "Cargo $Mode lane '$Lane' requires at least $MinimumFreeSpaceGB GB free on '$([IO.Path]::GetPathRoot($resolvedTargetRoot))'; $([math]::Round($freeSpaceBytes / 1GB, 2)) GB is available."
    }

    $targetId = [guid]::NewGuid().ToString("N")
    $targetDirectory = Join-Path $resolvedTargetRoot "$Lane-$targetId"
    New-Item -ItemType Directory -Path $targetDirectory | Out-Null
    Assert-TessaraPathWithinRoot `
        -RootPath $resolvedTargetRoot `
        -ChildPath $targetDirectory `
        -Label "Cargo target directory"
    Assert-TessaraOrdinaryDirectory `
        -Path $targetDirectory `
        -Label "Cargo target directory"

    $identity = Get-TessaraCargoBuildPolicyIdentity
    $leaseId = [guid]::NewGuid().ToString("N")
    $marker = [pscustomobject][ordered]@{
        schema_version = 1
        contract = $identity.contract
        policy_release = $identity.release_version
        policy_fingerprint = $identity.module_sha256
        lease_id = $leaseId
        target_id = $targetId
        lane = $Lane
        mode = $Mode
        target_root = $resolvedTargetRoot
        target_directory = $targetDirectory
        repository_root = $resolvedRepositoryRoot
        retain_target = [bool]$RetainTarget
    }
    $markerPath = Join-Path $targetDirectory $script:CargoBuildPolicyMarkerName
    $markerJson = $marker | ConvertTo-Json -Compress
    [IO.File]::WriteAllText(
        $markerPath,
        $markerJson,
        [Text.UTF8Encoding]::new($false)
    )
    $markerHash = (
        Get-FileHash -Algorithm SHA256 -LiteralPath $markerPath
    ).Hash.ToLowerInvariant()

    $lease = [pscustomobject][ordered]@{
        lease_id = $leaseId
        target_id = $targetId
        mode = $Mode
        lane = $Lane
        target_directory = $targetDirectory
        target_root = $resolvedTargetRoot
        repository_root = $resolvedRepositoryRoot
        cargo_executable_path = $resolvedCargoExecutable
        cargo_executable_sha256 = $cargoExecutableSha256
        marker_sha256 = $markerHash
        retain_target = [bool]$RetainTarget
        policy_release = $identity.release_version
        policy_fingerprint = $identity.module_sha256
        previous_cargo_target_dir = $env:CARGO_TARGET_DIR
        previous_cargo_incremental = $env:CARGO_INCREMENTAL
        previous_test_debug = $env:CARGO_PROFILE_TEST_DEBUG
    }
    $script:ActiveCargoBuildTargets[$leaseId] = $lease

    $state = [pscustomobject][ordered]@{}
    foreach ($property in $lease.PSObject.Properties) {
        $state | Add-Member -NotePropertyName $property.Name -NotePropertyValue $property.Value
    }

    $env:CARGO_TARGET_DIR = $targetDirectory
    $env:CARGO_INCREMENTAL = if ($policy.incremental) { "1" } else { "0" }
    $env:CARGO_PROFILE_TEST_DEBUG = $policy.test_debug

    Write-Host (
        "Cargo {0} policy {1}: lane={2}; target={3}; incremental={4}; test-debug={5}; cleanup={6}; fingerprint={7}" -f
            $Mode,
            $identity.release_version,
            $Lane,
            $targetDirectory,
            $env:CARGO_INCREMENTAL,
            $env:CARGO_PROFILE_TEST_DEBUG,
            $policy.automatic_cleanup,
            $identity.module_sha256
    ) -ForegroundColor Yellow

    return $state
}

function Exit-TessaraCargoBuildPolicy {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][psobject]$State,
        [scriptblock]$CargoInvoker
    )

    $leaseId = [string](
        Get-TessaraRequiredStateProperty -State $State -Name "lease_id"
    )
    if ($leaseId -notmatch "^[a-f0-9]{32}$" -or
        -not $script:ActiveCargoBuildTargets.ContainsKey($leaseId)) {
        throw "Cargo target state does not identify an active policy lease."
    }

    $lease = $script:ActiveCargoBuildTargets[$leaseId]
    $completed = $false
    try {
        Assert-TessaraStateMatchesLease -State $State -Lease $lease

        $currentIdentity = Get-TessaraCargoBuildPolicyIdentity
        if ($currentIdentity.release_version -cne $lease.policy_release -or
            $currentIdentity.module_sha256 -cne $lease.policy_fingerprint) {
            throw "Cargo build policy identity changed while the target lease was active."
        }

        Assert-TessaraPathWithinRoot `
            -RootPath $lease.target_root `
            -ChildPath $lease.target_directory `
            -Label "Cargo target directory"
        $expectedDirectoryName = "$($lease.lane)-$($lease.target_id)"
        $actualDirectoryName = [IO.Path]::GetFileName($lease.target_directory)
        if ($actualDirectoryName -cne $expectedDirectoryName) {
            throw "Cargo target directory name does not match its active lease."
        }
        Assert-TessaraOrdinaryDirectory `
            -Path $lease.repository_root `
            -Label "Cargo policy repository root"
        Assert-TessaraOrdinaryFile `
            -Path (Join-Path $lease.repository_root "Cargo.toml") `
            -Label "Cargo policy manifest"
        Assert-TessaraOrdinaryDirectory `
            -Path $lease.target_root `
            -Label "Cargo target root"
        Assert-TessaraOrdinaryDirectory `
            -Path $lease.target_directory `
            -Label "Cargo target directory"
        Assert-TessaraCargoTargetMarker -Lease $lease

        if ($lease.retain_target) {
            Write-Host (
                "Retained diagnostic Cargo target: {0}; policy={1}; fingerprint={2}" -f
                    $lease.target_directory,
                    $lease.policy_release,
                    $lease.policy_fingerprint
            ) -ForegroundColor Yellow
            $completed = $true
            return
        }

        Assert-TessaraOrdinaryFile `
            -Path ([string]$lease.cargo_executable_path) `
            -Label "Cargo policy executable"
        if ((Get-FileHash -Algorithm SHA256 `
                -LiteralPath ([string]$lease.cargo_executable_path)).Hash.ToLowerInvariant() -cne
            [string]$lease.cargo_executable_sha256) {
            throw "Cargo policy executable changed while the target lease was active."
        }
        $cargoArguments = @(
            "clean",
            "--manifest-path", (Join-Path $lease.repository_root "Cargo.toml"),
            "--target-dir", [string]$lease.target_directory
        )
        if ($null -eq $CargoInvoker) {
            & ([string]$lease.cargo_executable_path) @cargoArguments
        } else {
            & $CargoInvoker -Arguments $cargoArguments
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo target cleanup failed with exit code ${LASTEXITCODE}: $($lease.target_directory)"
        }
        if (Test-Path -LiteralPath $lease.target_directory) {
            throw "Cargo target cleanup did not remove the leased target: $($lease.target_directory)"
        }

        Write-Host "Cleaned validation Cargo target: $($lease.target_directory)" -ForegroundColor Green
        $completed = $true
    } finally {
        $env:CARGO_TARGET_DIR = $lease.previous_cargo_target_dir
        $env:CARGO_INCREMENTAL = $lease.previous_cargo_incremental
        $env:CARGO_PROFILE_TEST_DEBUG = $lease.previous_test_debug
        if ($completed) {
            [void]$script:ActiveCargoBuildTargets.Remove($leaseId)
        }
    }
}

Export-ModuleMember -Function @(
    "Get-TessaraCargoBuildPolicyIdentity",
    "Get-TessaraCargoBuildPolicy",
    "Enter-TessaraCargoBuildPolicy",
    "Exit-TessaraCargoBuildPolicy"
)

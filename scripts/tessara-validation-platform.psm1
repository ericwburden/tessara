Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
if ($PSVersionTable.PSEdition -cne "Core" -or $PSVersionTable.PSVersion -lt [version]"7.3") {
    throw "The validation platform requires pwsh (PowerShell Core) 7.3 or newer."
}

$script:PlatformModulePath = [IO.Path]::GetFullPath($PSCommandPath)
$script:PlatformModuleSha256AtImport = (
    Get-FileHash -Algorithm SHA256 -LiteralPath $script:PlatformModulePath
).Hash.ToLowerInvariant()
$script:RepositoryRoot = Split-Path -Parent $PSScriptRoot
$script:PlatformRoot = Join-Path $PSScriptRoot "validation-platform"
$script:ManifestPath = Join-Path $script:PlatformRoot "platform-manifest.json"
$script:AdapterSchemaPath = Join-Path $script:PlatformRoot "validation-adapter.schema.json"
$script:CargoModulePath = Join-Path $PSScriptRoot "tessara-cargo-build-policy.psm1"
$script:ValidationPolicyModulePath = Join-Path $PSScriptRoot "tessara-validation-policy.psm1"
$script:LifecycleModulePath = Join-Path $script:PlatformRoot "tessara-validation-lifecycle.psm1"
$script:FinalizerModulePath = Join-Path $script:PlatformRoot "tessara-validation-finalizer.psm1"
$gitCommand = @(Get-Command git -CommandType Application -All -ErrorAction Stop) |
    Select-Object -First 1
if ($null -eq $gitCommand -or [string]::IsNullOrWhiteSpace([string]$gitCommand.Source)) {
    throw "The validation platform requires one directly executable Git program."
}
$script:GitExecutablePath = [IO.Path]::GetFullPath([string]$gitCommand.Source)
$gitEntry = Get-Item -Force -LiteralPath $script:GitExecutablePath
if (-not (Test-Path -LiteralPath $script:GitExecutablePath -PathType Leaf) -or
    ($gitEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
    throw "The validation platform Git program must be one ordinary executable file."
}
$script:GitExecutableSha256AtImport = (
    Get-FileHash -Algorithm SHA256 -LiteralPath $script:GitExecutablePath
).Hash.ToLowerInvariant()
$gitVersionOutput = @(& $script:GitExecutablePath --version)
if ($LASTEXITCODE -ne 0 -or $gitVersionOutput.Count -ne 1) {
    throw "The validation platform could not identify its Git runtime."
}
$script:GitVersionAtImport = [string]$gitVersionOutput[0]

foreach ($componentModulePath in @(
        $script:CargoModulePath,
        $script:ValidationPolicyModulePath,
        $script:LifecycleModulePath,
        $script:FinalizerModulePath
    )) {
    $resolvedComponentModulePath = [IO.Path]::GetFullPath($componentModulePath)
    $loadedComponent = @(Get-Module | Where-Object {
        -not [string]::IsNullOrWhiteSpace($_.Path) -and
        [IO.Path]::GetFullPath($_.Path) -eq $resolvedComponentModulePath
    }) | Select-Object -First 1
    if ($null -eq $loadedComponent) {
        Import-Module $resolvedComponentModulePath
    }
}

function Get-TessaraPlatformSha256Text {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
}

function Get-TessaraPlatformSha256Bytes {
    param([Parameter(Mandatory)][byte[]]$Bytes)
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Split-TessaraPlatformNativePath {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Path)
    $separators = [Collections.Generic.List[char]]::new()
    $separators.Add([IO.Path]::DirectorySeparatorChar)
    if ([IO.Path]::AltDirectorySeparatorChar -ne [IO.Path]::DirectorySeparatorChar) {
        $separators.Add([IO.Path]::AltDirectorySeparatorChar)
    }
    @($Path.Split($separators.ToArray(), [StringSplitOptions]::RemoveEmptyEntries))
}

function Assert-TessaraPlatformNoReparsePath {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )
    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $resolved = [IO.Path]::GetFullPath($Path)
    $relative = [IO.Path]::GetRelativePath($root, $resolved)
    if ($relative -eq "." -or $relative -eq ".." -or
        [IO.Path]::IsPathRooted($relative) -or
        $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
        $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
        throw "$Label is outside the repository root: $Path"
    }
    $current = $root
    foreach ($segment in @(Split-TessaraPlatformNativePath -Path $relative)) {
        $current = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $current)) { continue }
        $entry = Get-Item -Force -LiteralPath $current
        if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "$Label crosses a reparse point: $current"
        }
    }
    $resolved
}

function Read-TessaraPlatformUtf8Snapshot {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label,
        [int64]$MaximumBytes = 4194304
    )
    $entry = Get-Item -Force -LiteralPath $Path
    if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Label must be one ordinary file: $Path"
    }
    if ($entry.Length -gt $MaximumBytes) {
        throw "$Label exceeds its $MaximumBytes-byte validation limit."
    }
    $bytes = [IO.File]::ReadAllBytes($Path)
    try {
        $text = [Text.UTF8Encoding]::new($false, $true).GetString($bytes)
    } catch {
        throw "$Label is not strict UTF-8. $($_.Exception.Message)"
    }
    [pscustomobject][ordered]@{
        path = [IO.Path]::GetFullPath($Path)
        text = $text
        sha256 = Get-TessaraPlatformSha256Bytes -Bytes $bytes
    }
}

function Assert-TessaraPlatformSnapshotCurrent {
    param(
        [Parameter(Mandatory)]$Snapshot,
        [Parameter(Mandatory)][string]$Label
    )
    if (-not (Test-Path -LiteralPath ([string]$Snapshot.path) -PathType Leaf) -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath ([string]$Snapshot.path)).Hash.ToLowerInvariant() -cne
            [string]$Snapshot.sha256) {
        throw "$Label changed while it was being validated."
    }
}

function Assert-TessaraPlatformGitExecutableCurrent {
    $entry = Get-Item -Force -LiteralPath $script:GitExecutablePath
    if (-not (Test-Path -LiteralPath $script:GitExecutablePath -PathType Leaf) -or
        ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $script:GitExecutablePath).Hash.ToLowerInvariant() -cne
            $script:GitExecutableSha256AtImport) {
        throw "The validation-platform Git executable changed after module import."
    }
    $script:GitExecutablePath
}

function Invoke-TessaraPlatformIsolatedGit {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments
    )
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $GitPath
    $start.WorkingDirectory = [IO.Path]::GetFullPath($RepositoryRoot)
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $strictUtf8 = [Text.UTF8Encoding]::new($false, $true)
    $start.StandardOutputEncoding = $strictUtf8
    $start.StandardErrorEncoding = $strictUtf8
    $start.Environment.Clear()
    if ($IsWindows) {
        $windowsRoot = [Environment]::GetEnvironmentVariable("SystemRoot", "Machine")
        if ([string]::IsNullOrWhiteSpace($windowsRoot)) {
            $windowsRoot = [Environment]::GetEnvironmentVariable("SystemRoot", "Process")
        }
        $start.Environment["SystemRoot"] = $windowsRoot
        $start.Environment["WINDIR"] = $windowsRoot
        $nullDevice = "NUL"
    } else {
        $nullDevice = "/dev/null"
    }
    $start.Environment["GIT_CONFIG_NOSYSTEM"] = "1"
    $start.Environment["GIT_CONFIG_GLOBAL"] = $nullDevice
    $start.Environment["GIT_ATTR_NOSYSTEM"] = "1"
    $start.Environment["GIT_OPTIONAL_LOCKS"] = "0"
    $start.Environment["GIT_TERMINAL_PROMPT"] = "0"
    $start.Environment["LC_ALL"] = "C"
    $start.Environment["LANG"] = "C"
    $start.ArgumentList.Add("-c")
    $start.ArgumentList.Add("safe.directory=$([IO.Path]::GetFullPath($RepositoryRoot))")
    foreach ($argument in @($Arguments)) {
        $start.ArgumentList.Add([string]$argument)
    }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        if (-not $process.Start()) { throw "The validation-platform Git process did not start." }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        [pscustomobject][ordered]@{
            exit_code = $process.ExitCode
            stdout = $stdoutTask.GetAwaiter().GetResult()
            stderr = $stderrTask.GetAwaiter().GetResult()
        }
    } finally {
        $process.Dispose()
    }
}

function Assert-TessaraPlatformEvidenceRootContract {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$DeclaredPath
    )
    $normalized = $DeclaredPath.Replace('\', '/')
    if (-not $normalized.StartsWith("artifacts/", [StringComparison]::Ordinal) -or
        $normalized.EndsWith("/", [StringComparison]::Ordinal)) {
        throw "Validation-contract evidence root must be one owned artifacts/... directory."
    }
    $resolved = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $RepositoryRoot `
        -Path $normalized -Label "Validation-contract evidence root"
    $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $RepositoryRoot `
        -Path $resolved -Label "Validation-contract evidence root"
    $gitPath = Assert-TessaraPlatformGitExecutableCurrent
    $ignore = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $RepositoryRoot `
        -Arguments @("-C", $RepositoryRoot, "check-ignore", "--no-index", "--verbose", "--", $normalized)
    if ([int]$ignore.exit_code -eq 1) {
        throw "Validation-contract evidence root must be ignored by the repository: $normalized"
    }
    if ([int]$ignore.exit_code -ne 0) {
        throw "Unable to authenticate the validation-contract evidence-root ignore rule."
    }
    $ignoreMetadata = @([string]$ignore.stdout -split '\r?\n' | Where-Object {
            -not [string]::IsNullOrWhiteSpace($_)
        }) | Select-Object -First 1
    if ($ignoreMetadata -cnotmatch '^(?<path>.*\.gitignore):\d+:') {
        throw "Validation-contract evidence root must be owned by a repository .gitignore rule."
    }
    $ignoreRulePath = [string]$Matches.path
    $trackedRule = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $RepositoryRoot `
        -Arguments @("-C", $RepositoryRoot, "ls-files", "--error-unmatch", "--", $ignoreRulePath)
    if ([int]$trackedRule.exit_code -ne 0) {
        throw "Validation-contract evidence root ignore rule must be tracked by the repository."
    }
    $resolved
}

function Enter-TessaraPlatformAdmission {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][int]$MaximumParallelLanes
    )
    [IO.Directory]::CreateDirectory($EvidenceRoot) | Out-Null
    $admissionRoot = Join-Path $EvidenceRoot ".admission"
    [IO.Directory]::CreateDirectory($admissionRoot) | Out-Null
    for ($slot = 0; $slot -lt $MaximumParallelLanes; $slot++) {
        $path = Join-Path $admissionRoot "slot-$slot.lock"
        try {
            $stream = [IO.FileStream]::new(
                $path, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite,
                [IO.FileShare]::None, 4096, [IO.FileOptions]::DeleteOnClose
            )
            $bytes = [Text.UTF8Encoding]::new($false).GetBytes(
                "$PID`n$([DateTimeOffset]::UtcNow.ToString('O'))`n"
            )
            $stream.SetLength(0)
            $stream.Write($bytes, 0, $bytes.Length)
            $stream.Flush($true)
            return $stream
        } catch [IO.IOException] {
            continue
        }
    }
    throw "Validation-platform admission is full; the configured parallel-lane limit is $MaximumParallelLanes."
}

function Assert-TessaraPlatformEvidenceBudget {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$LaneId,
        [Parameter(Mandatory)][int]$MaximumAttempts,
        [Parameter(Mandatory)][long]$MaximumBytes
    )
    $laneRoot = Join-Path $EvidenceRoot "lanes/$LaneId"
    if (Test-Path -LiteralPath $laneRoot -PathType Container) {
        $attemptCount = @(Get-ChildItem -Force -LiteralPath $laneRoot -Directory).Count
        if ($attemptCount -ge $MaximumAttempts) {
            throw "Validation evidence reached the configured $MaximumAttempts-attempt limit for lane '$LaneId'."
        }
    }
    [long]$bytes = 0
    [int]$entries = 0
    $pending = [Collections.Generic.Stack[string]]::new()
    $pending.Push([IO.Path]::GetFullPath($EvidenceRoot))
    while ($pending.Count -gt 0) {
        $directory = $pending.Pop()
        foreach ($entry in [IO.Directory]::EnumerateFileSystemEntries($directory)) {
            $entries++
            if ($entries -gt 100000) {
                throw "Validation evidence exceeds the platform's 100,000-entry inspection bound."
            }
            $item = Get-Item -Force -LiteralPath $entry
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Validation evidence contains a reparse entry outside platform ownership: $entry"
            }
            if ($item.PSIsContainer) {
                $pending.Push([string]$item.FullName)
            } else {
                $bytes += [long]$item.Length
                if ($bytes -gt $MaximumBytes) {
                    throw "Validation evidence exceeds the configured $MaximumBytes-byte retention budget."
                }
            }
        }
    }
}

function Get-TessaraPlatformRuntimeIdentity {
    $hostPath = [Environment]::ProcessPath
    if ([string]::IsNullOrWhiteSpace($hostPath)) {
        $hostPath = (Get-Process -Id $PID).Path
    }
    $resolvedHost = [IO.Path]::GetFullPath($hostPath)
    if (-not (Test-Path -LiteralPath $resolvedHost -PathType Leaf)) {
        throw "Validation-platform host executable could not be identified."
    }
    [pscustomobject][ordered]@{
        contract = "tessara.validation.runtime"
        powershell_version = $PSVersionTable.PSVersion.ToString()
        powershell_edition = [string]$PSVersionTable.PSEdition
        dotnet = [Runtime.InteropServices.RuntimeInformation]::FrameworkDescription
        os = [Runtime.InteropServices.RuntimeInformation]::OSDescription
        os_architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
        process_architecture = [Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString()
        host_path_sha256 = Get-TessaraPlatformSha256Text -Text $resolvedHost
        host_content_sha256 = (
            Get-FileHash -Algorithm SHA256 -LiteralPath $resolvedHost
        ).Hash.ToLowerInvariant()
        git_executable_path = Assert-TessaraPlatformGitExecutableCurrent
        git_path_sha256 = Get-TessaraPlatformSha256Text -Text $script:GitExecutablePath
        git_content_sha256 = $script:GitExecutableSha256AtImport
        git_version = $script:GitVersionAtImport
    }
}

function ConvertTo-TessaraPlatformCanonicalJsonValue {
    param([AllowNull()]$Value)
    if ($null -eq $Value -or $Value -is [string] -or
        $Value -is [bool] -or $Value -is [ValueType]) {
        return $Value
    }
    if ($Value -is [Collections.IDictionary]) {
        $result = [ordered]@{}
        foreach ($key in @($Value.Keys | ForEach-Object { [string]$_ } | Sort-Object)) {
            $result[$key] = ConvertTo-TessaraPlatformCanonicalJsonValue -Value $Value[$key]
        }
        return $result
    }
    if ($Value -is [Collections.IEnumerable]) {
        return @($Value | ForEach-Object {
            ConvertTo-TessaraPlatformCanonicalJsonValue -Value $_
        })
    }
    $objectResult = [ordered]@{}
    foreach ($property in @($Value.PSObject.Properties | Sort-Object Name)) {
        $objectResult[$property.Name] = ConvertTo-TessaraPlatformCanonicalJsonValue `
            -Value $property.Value
    }
    $objectResult
}

function Get-TessaraPlatformCanonicalJsonSha256 {
    param([Parameter(Mandatory)]$Value)
    $canonical = ConvertTo-TessaraPlatformCanonicalJsonValue -Value $Value
    $json = $canonical | ConvertTo-Json -Depth 100 -Compress
    Get-TessaraPlatformSha256Text -Text ($json + "`n")
}

function ConvertFrom-TessaraPlatformGitNulRecords {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [Parameter(Mandatory)][string]$Label
    )
    if ($Text.Length -eq 0) { return @() }
    if (-not $Text.EndsWith([string][char]0, [StringComparison]::Ordinal)) {
        throw "$Label did not use complete NUL-delimited framing."
    }
    $frames = @($Text.Split([char]0))
    $records = @($frames[0..($frames.Count - 2)])
    if (@($records | Where-Object { [string]::IsNullOrEmpty([string]$_) }).Count -ne 0) {
        throw "$Label contains an invalid empty path frame."
    }
    @($records)
}

function Resolve-TessaraPlatformRepositoryPath {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )
    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $resolved = if ([IO.Path]::IsPathRooted($Path)) {
        [IO.Path]::GetFullPath($Path)
    } else {
        [IO.Path]::GetFullPath((Join-Path $root $Path))
    }
    $relative = [IO.Path]::GetRelativePath($root, $resolved)
    $prefix = "..$([IO.Path]::DirectorySeparatorChar)"
    if ($relative -eq "." -or $relative -eq ".." -or
        [IO.Path]::IsPathRooted($relative) -or
        $relative.StartsWith($prefix, [StringComparison]::Ordinal)) {
        throw "$Label is outside the repository root: $Path"
    }
    $resolved
}

function Get-TessaraPlatformLiveDependencyFingerprints {
    param(
        [Parameter(Mandatory)]$ValidationContract,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $gitPath = Assert-TessaraPlatformGitExecutableCurrent
    $cachedInventory = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $repository `
        -Arguments @("-C", $repository, "ls-files", "--stage", "-z", "--cached")
    $untrackedInventory = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $repository `
        -Arguments @("-C", $repository, "ls-files", "-z", "--others", "--exclude-per-directory=.gitignore")
    if ([int]$cachedInventory.exit_code -ne 0 -or
        [int]$untrackedInventory.exit_code -ne 0) {
        throw "Unable to enumerate the candidate working tree for lane dependency fingerprints."
    }
    $cachedModes = [Collections.Generic.Dictionary[string, string]]::new(
        [StringComparer]::Ordinal
    )
    foreach ($record in @(ConvertFrom-TessaraPlatformGitNulRecords `
            -Text ([string]$cachedInventory.stdout) `
            -Label "Candidate cached working-tree inventory")) {
        if ([string]$record -cnotmatch
            '(?s)^(?<mode>[0-9]{6}) [0-9a-f]{40,64} (?<stage>[0-3])\t(?<path>.+)$' -or
            [string]$Matches.stage -cne "0") {
            throw "Candidate cached working-tree inventory contains an unsupported index entry."
        }
        $mode = [string]$Matches.mode
        $path = [string]$Matches.path
        if (-not $cachedModes.TryAdd($path, $mode)) {
            throw "Candidate cached working-tree inventory contains a duplicate path: $path"
        }
    }
    $allPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($path in $cachedModes.Keys) { $null = $allPaths.Add($path) }
    foreach ($path in @(ConvertFrom-TessaraPlatformGitNulRecords `
            -Text ([string]$untrackedInventory.stdout) `
            -Label "Candidate untracked working-tree inventory")) {
        if (-not $allPaths.Add([string]$path)) {
            throw "Candidate working-tree inventory contains a duplicate cached/untracked path: $path"
        }
    }
    $entries = [Collections.Generic.List[object]]::new()
    $domainPatterns = @($ValidationContract.dependency_domains | ForEach-Object {
        @($_.tracked_inputs | ForEach-Object {
            ([string]$_).Replace('\', '/').Trim()
        })
    })
    foreach ($rawPath in @($allPaths | Sort-Object -CaseSensitive)) {
        $path = [string]$rawPath
        if (@($domainPatterns | Where-Object {
                    $path -clike ([string]$_)
                }).Count -eq 0) {
            continue
        }
        if ($cachedModes.ContainsKey($path) -and
            [string]$cachedModes[$path] -notin @("100644", "100755")) {
            throw "Candidate dependency input uses unsupported tracked entry mode '$([string]$cachedModes[$path])': $path"
        }
        $resolved = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $repository `
            -Path $path -Label "Candidate dependency input"
        if (Test-Path -LiteralPath $resolved -PathType Leaf) {
            $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $repository `
                -Path $resolved -Label "Candidate dependency input"
            $entries.Add([pscustomobject][ordered]@{
                path = $path
                sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $resolved).Hash.ToLowerInvariant()
            })
        } elseif (Test-Path -LiteralPath $resolved) {
            throw "Candidate dependency input is not one ordinary file: $path"
        } elseif ($cachedModes.ContainsKey($path)) {
            # Keep a cached-but-deleted path in the identity rather than
            # silently dropping it from its declared dependency domain.
            $entries.Add([pscustomobject][ordered]@{
                path = $path
                sha256 = "absent"
            })
        } else {
            throw "Untracked candidate dependency input disappeared during inventory: $path"
        }
    }
    @($ValidationContract.dependency_domains | ForEach-Object {
        $domain = $_
        $matches = @($entries | Where-Object {
            $candidatePath = [string]$_.path
            @($domain.tracked_inputs | Where-Object {
                $candidatePath -clike ([string]$_).Replace('\', '/').Trim()
            }).Count -gt 0
        } | Sort-Object path)
        $fingerprintValue = [pscustomobject][ordered]@{
            entries = @($matches | ForEach-Object {
                [pscustomobject][ordered]@{
                    path = [string]$_.path
                    sha256 = [string]$_.sha256
                }
            })
        }
        [pscustomobject][ordered]@{
            domain = [string]$domain.name
            sha256 = Get-TessaraPlatformCanonicalJsonSha256 -Value $fingerprintValue
            file_count = $matches.Count
        }
    } | Sort-Object domain)
}

function Get-TessaraPlatformLaneEnvironmentObservationFingerprint {
    param([Parameter(Mandatory)]$Lane)
    $allowedNames = @(
        "PATHEXT", "SystemRoot", "WINDIR", "ComSpec",
        "PROGRAMDATA", "ProgramFiles", "ProgramFiles(x86)", "ProgramW6432",
        "NUMBER_OF_PROCESSORS", "PROCESSOR_ARCHITECTURE", "OS",
        "LANG", "LC_ALL"
    )
    $source = [Environment]::GetEnvironmentVariables("Process")
    $observations = [Collections.Generic.List[object]]::new()
    foreach ($name in $allowedNames) {
        if ($source.Contains($name)) {
            $value = [string][Environment]::GetEnvironmentVariable($name, "Process")
            $observations.Add([pscustomobject][ordered]@{
                name = $name
                kind = "platform-base"
                present = $true
                value_sha256 = Get-TessaraPlatformSha256Text -Text $value
            })
        }
    }
    foreach ($entry in @($Lane.environment | Where-Object {
            $_.PSObject.Properties.Name -contains "source"
        })) {
        $sourceName = [string]$entry.source
        $present = $source.Contains($sourceName)
        $value = if ($present) {
            [string][Environment]::GetEnvironmentVariable($sourceName, "Process")
        } else { $null }
        $observations.Add([pscustomobject][ordered]@{
            name = [string]$entry.name
            kind = "declared-source"
            source = $sourceName
            present = $present
            value_sha256 = if ($present) {
                Get-TessaraPlatformSha256Text -Text $value
            } else { $null }
        })
    }
    Get-TessaraPlatformCanonicalJsonSha256 -Value (
        [pscustomobject][ordered]@{ observations = @($observations | Sort-Object kind, name) }
    )
}

function Get-TessaraPlatformProgramObservation {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [AllowEmptyCollection()][string[]]$InputPaths = @()
    )
    $expandedProgram = $Program.Replace('${repository_root}', [IO.Path]::GetFullPath($RepositoryRoot))
    if ($expandedProgram -match '\$\{[^}]+\}') {
        throw "Validation command '$Id' program may use only the repository_root placeholder."
    }
    $candidate = if ([IO.Path]::IsPathRooted($expandedProgram)) {
        [IO.Path]::GetFullPath($expandedProgram)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $expandedProgram))
    }
    $resolved = if (Test-Path -LiteralPath $candidate -PathType Leaf) {
        $candidate
    } else {
        $command = Get-Command -Name $expandedProgram -CommandType Application,ExternalScript `
            -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -eq $command -or
            [string]::IsNullOrWhiteSpace([string]$command.Source)) {
            $null
        } else { [IO.Path]::GetFullPath([string]$command.Source) }
    }
    [pscustomobject][ordered]@{
        id = $Id
        present = $null -ne $resolved
        program_path_sha256 = if ($null -eq $resolved) { $null } else {
            Get-TessaraPlatformSha256Text -Text $resolved
        }
        program_sha256 = if ($null -eq $resolved) { $null } else {
            (Get-FileHash -Algorithm SHA256 -LiteralPath $resolved).Hash.ToLowerInvariant()
        }
        input_paths = @(Get-TessaraPlatformCommandInputObservations `
            -InputPaths $InputPaths -RepositoryRoot $RepositoryRoot)
    }
}

function Get-TessaraPlatformCommandInputObservations {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$InputPaths,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    @($InputPaths | ForEach-Object {
        $declaredPath = [string]$_
        $resolved = Resolve-TessaraPlatformRepositoryPath `
            -RepositoryRoot $RepositoryRoot -Path $declaredPath `
            -Label "Declared command input"
        if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
            throw "Declared command input is missing: $declaredPath"
        }
        $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $RepositoryRoot `
            -Path $resolved -Label "Declared command input"
        [pscustomobject][ordered]@{
            path = $declaredPath
            expanded_path_sha256 = Get-TessaraPlatformSha256Text -Text $resolved
            content_sha256 = (
                Get-FileHash -Algorithm SHA256 -LiteralPath $resolved
            ).Hash.ToLowerInvariant()
        }
    } | Sort-Object path)
}

function Assert-TessaraPlatformDirectInterpreterInvocation {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$InputPaths,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    $programName = [IO.Path]::GetFileName($Program).ToLowerInvariant()
    $declaredResolvedPaths = @($InputPaths | ForEach-Object {
        Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $RepositoryRoot `
            -Path ([string]$_) -Label "$Label declared input"
    })
    $expandedArguments = @($Arguments | ForEach-Object {
        ([string]$_).Replace('${repository_root}', [IO.Path]::GetFullPath($RepositoryRoot))
    })
    if ($programName -in @(
            "powershell", "powershell.exe", "cmd", "cmd.exe", "wsl", "wsl.exe"
        )) {
        throw "$Label uses an unsupported command-string interpreter '$programName'."
    }
    if ($programName -in @(
            "sh", "sh.exe", "bash", "bash.exe", "dash", "dash.exe",
            "zsh", "zsh.exe", "fish", "fish.exe", "ksh", "ksh.exe",
            "ash", "ash.exe"
        )) {
        if ($expandedArguments.Count -eq 0) {
            throw "$Label shell invocation has no directly declared script input."
        }
        $scriptPath = if ([IO.Path]::IsPathRooted($expandedArguments[0])) {
            [IO.Path]::GetFullPath($expandedArguments[0])
        } else {
            [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $expandedArguments[0]))
        }
        if ($scriptPath -cnotin $declaredResolvedPaths) {
            throw "$Label shell invocation must name one declared input directly as its first argument."
        }
        return
    }
    if ($programName -notin @("pwsh", "pwsh.exe")) { return }
    $fileIndexes = @(for ($index = 0; $index -lt $Arguments.Count; $index++) {
        if ([string]$Arguments[$index] -ceq "-File") { $index }
    })
    if ($fileIndexes.Count -ne 1) {
        throw "$Label pwsh invocation must use exactly one exact '-File' switch."
    }
    $fileIndex = [int]$fileIndexes[0]
    if ($fileIndex + 1 -ge $expandedArguments.Count) {
        throw "$Label pwsh invocation has no direct script path after '-File'."
    }
    $approvedPrefix = @("-NoProfile", "-NonInteractive", "-NoLogo")
    $prefix = @(if ($fileIndex -ne 0) { $Arguments[0..($fileIndex - 1)] })
    if ((@($prefix | Where-Object { [string]$_ -cnotin $approvedPrefix })).Count -ne 0 -or
        (@($prefix | Sort-Object -Unique)).Count -ne $prefix.Count -or
        "-NoProfile" -cnotin $prefix -or "-NonInteractive" -cnotin $prefix) {
        throw "$Label pwsh invocation may use only exact approved pre-File switches and must include '-NoProfile' and '-NonInteractive'."
    }
    $scriptPath = if ([IO.Path]::IsPathRooted($expandedArguments[$fileIndex + 1])) {
        [IO.Path]::GetFullPath($expandedArguments[$fileIndex + 1])
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $expandedArguments[$fileIndex + 1]))
    }
    if ($scriptPath -cnotin $declaredResolvedPaths) {
        throw "$Label pwsh '-File' target is not one declared input path."
    }
}

function Get-TessaraPlatformLaneToolObservation {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string[]]$DockerCommand,
        [AllowEmptyCollection()][string[]]$DockerCommandInputPaths = @()
    )
    $observations = [Collections.Generic.List[object]]::new()
    if ([string]$Lane.topology.provider -ceq "local-process") {
        $observations.Add((Get-TessaraPlatformProgramObservation -Id "topology" `
            -Program ([string]$Lane.topology.command.program) -RepositoryRoot $RepositoryRoot `
            -InputPaths @($Lane.topology.command.input_paths)))
        foreach ($tool in @($Lane.topology.command.tools)) {
            $observations.Add((Get-TessaraPlatformProgramObservation `
                -Id "topology-tool:$([string]$tool.id)" `
                -Program ([string]$tool.program) -RepositoryRoot $RepositoryRoot `
                -InputPaths @($tool.input_paths)))
        }
    } elseif ([string]$Lane.topology.provider -ceq "docker-compose") {
        $observations.Add((Get-TessaraPlatformProgramObservation -Id "docker" `
            -Program ([string]$DockerCommand[0]) -RepositoryRoot $RepositoryRoot `
            -InputPaths $DockerCommandInputPaths))
    }
    foreach ($action in @($Lane.actions)) {
        $observations.Add((Get-TessaraPlatformProgramObservation `
            -Id "action:$([string]$action.id)" -Program ([string]$action.program) `
            -RepositoryRoot $RepositoryRoot -InputPaths @($action.input_paths)))
        foreach ($tool in @($action.tools)) {
            $observations.Add((Get-TessaraPlatformProgramObservation `
                -Id "action:$([string]$action.id):tool:$([string]$tool.id)" `
                -Program ([string]$tool.program) -RepositoryRoot $RepositoryRoot `
                -InputPaths @($tool.input_paths)))
        }
    }
    $dockerPrefix = if ([string]$Lane.topology.provider -ceq "docker-compose" -and
        $DockerCommand.Count -gt 1) { @($DockerCommand[1..($DockerCommand.Count - 1)]) } else { @() }
    $document = [pscustomobject][ordered]@{
        programs = @($observations | Sort-Object id)
        docker_prefix = @($dockerPrefix)
    }
    [pscustomobject][ordered]@{
        programs = @($document.programs)
        docker_prefix_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 `
            -Value ([pscustomobject][ordered]@{
                arguments = @($dockerPrefix)
                input_paths = @(if ([string]$Lane.topology.provider -ceq "docker-compose") {
                    Get-TessaraPlatformCommandInputObservations `
                        -InputPaths $DockerCommandInputPaths -RepositoryRoot $RepositoryRoot
                } else { @() })
            })
        fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value $document
    }
}

function Get-TessaraValidationPlatformIdentity {
    [CmdletBinding()]
    param()

    if (-not (Test-Path -LiteralPath $script:ManifestPath -PathType Leaf)) {
        throw "Validation-platform manifest is missing: $script:ManifestPath"
    }
    $manifestSnapshot = Read-TessaraPlatformUtf8Snapshot -Path $script:ManifestPath `
        -Label "Validation-platform manifest"
    $manifest = [string]$manifestSnapshot.text | ConvertFrom-Json -Depth 20
    if ([int]$manifest.schema_version -ne 1 -or
        [string]$manifest.contract -cne "tessara.validation.platform-manifest" -or
        [string]$manifest.release_version -cne "2.0.0") {
        throw "Validation-platform manifest has an unsupported identity."
    }
    $boundaryDeclarations = @($manifest.boundary_inputs)
    $expectedBoundaryIds = @(
        "public-entrypoint",
        "adapter-schema",
        "validation-contract-schema",
        "phase-certificate-v2-schema"
    )
    if ((@($boundaryDeclarations.id) -join "`n") -cne ($expectedBoundaryIds -join "`n")) {
        throw "Validation-platform manifest does not declare the exact ordered public-boundary inventory."
    }
    $expectedBoundaryPaths = @(
        "scripts/tessara-validation-platform.psm1",
        "scripts/validation-platform/validation-adapter.schema.json",
        ".codex/skills/tessara-sprint-validation/references/validation-contract.schema.json",
        ".codex/skills/tessara-sprint-validation/references/phase-certificate-v2.schema.json"
    )
    if ((@($boundaryDeclarations.path) -join "`n") -cne ($expectedBoundaryPaths -join "`n")) {
        throw "Validation-platform manifest redirects a canonical public-boundary path."
    }
    $boundaryInputs = [Collections.Generic.List[object]]::new()
    foreach ($declaration in $boundaryDeclarations) {
        $boundaryPath = Resolve-TessaraPlatformRepositoryPath `
            -RepositoryRoot $script:RepositoryRoot -Path ([string]$declaration.path) `
            -Label "Validation-platform boundary input"
        if (-not (Test-Path -LiteralPath $boundaryPath -PathType Leaf)) {
            throw "Validation-platform boundary input is missing: $boundaryPath"
        }
        $boundaryInputs.Add([pscustomobject][ordered]@{
            id = [string]$declaration.id
            path = [string]$declaration.path
            sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $boundaryPath).Hash.ToLowerInvariant()
        })
    }
    $componentDeclarations = @($manifest.components)
    $expectedIds = @("cargo-build-policy", "validation-policy", "lifecycle", "finalizer")
    if ((@($componentDeclarations.id) -join "`n") -cne ($expectedIds -join "`n")) {
        throw "Validation-platform manifest does not declare the exact ordered component inventory."
    }
    $expectedComponentPaths = @(
        "scripts/tessara-cargo-build-policy.psm1",
        "scripts/tessara-validation-policy.psm1",
        "scripts/validation-platform/tessara-validation-lifecycle.psm1",
        "scripts/validation-platform/tessara-validation-finalizer.psm1"
    )
    if ((@($componentDeclarations.path) -join "`n") -cne ($expectedComponentPaths -join "`n")) {
        throw "Validation-platform manifest redirects a canonical component path."
    }
    $componentIdentities = @(
        Get-TessaraCargoBuildPolicyIdentity
        Get-TessaraValidationPolicyIdentity
        Get-TessaraValidationLifecycleIdentity
        Get-TessaraValidationFinalizerIdentity
    )
    $components = [Collections.Generic.List[object]]::new()
    for ($index = 0; $index -lt $componentDeclarations.Count; $index++) {
        $declaration = $componentDeclarations[$index]
        $identity = $componentIdentities[$index]
        $componentPath = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $script:RepositoryRoot `
            -Path ([string]$declaration.path) -Label "Validation-platform component"
        $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $componentPath).Hash.ToLowerInvariant()
        if ([string]$declaration.contract -cne [string]$identity.contract -or
            [string]$declaration.release_version -cne [string]$identity.release_version -or
            $actualHash -cne [string]$identity.module_sha256) {
            throw "Validation-platform component '$($declaration.id)' does not match its declared identity."
        }
        $components.Add([pscustomobject][ordered]@{
            id = [string]$declaration.id
            contract = [string]$identity.contract
            release_version = [string]$identity.release_version
            path = [string]$declaration.path
            sha256 = $actualHash
            capability_fingerprints = if ($identity.PSObject.Properties.Name -contains
                "capability_fingerprints") { $identity.capability_fingerprints } else { $null }
        })
    }
    Assert-TessaraPlatformSnapshotCurrent -Snapshot $manifestSnapshot `
        -Label "Validation-platform manifest"
    $manifestHash = [string]$manifestSnapshot.sha256
    $publicBoundary = @($boundaryInputs | Where-Object {
        [string]$_.id -ceq "public-entrypoint"
    })[0]
    if ([string]$publicBoundary.sha256 -cne $script:PlatformModuleSha256AtImport) {
        throw "The loaded validation-platform entry point does not match its current source bytes."
    }
    $runtimeIdentity = Get-TessaraPlatformRuntimeIdentity
    $runtimeIdentityText = (
        ConvertTo-TessaraPlatformCanonicalJsonValue -Value $runtimeIdentity
    ) | ConvertTo-Json -Depth 20 -Compress
    $fingerprintInput = @(
        "tessara.validation.platform"
        [string]$manifest.release_version
        $manifestHash
        @($boundaryInputs | ForEach-Object {
            "$($_.id)`n$($_.path)`n$($_.sha256)"
        }) -join "`n"
        @($components | ForEach-Object {
            "$($_.id)`n$($_.contract)`n$($_.release_version)`n$($_.path)`n$($_.sha256)"
        }) -join "`n"
        $runtimeIdentityText
    ) -join "`n"
    $executionComponents = @($components | Where-Object {
        [string]$_.id -ceq "validation-policy"
    })
    $finalizationComponents = @($components | Where-Object {
        [string]$_.id -ceq "finalizer"
    })
    $executionFingerprintInput = @(
        "tessara.validation.platform-execution"
        [string]$manifest.release_version
        @($boundaryInputs | ForEach-Object {
            "$($_.id)`n$($_.path)`n$($_.sha256)"
        }) -join "`n"
        @($executionComponents | ForEach-Object {
            "$($_.id)`n$($_.contract)`n$($_.release_version)`n$($_.path)`n$($_.sha256)"
        }) -join "`n"
        $runtimeIdentityText
    ) -join "`n"
    $finalizationFingerprintInput = @(
        "tessara.validation.platform-finalization"
        [string]$manifest.release_version
        @($finalizationComponents | ForEach-Object {
            "$($_.id)`n$($_.contract)`n$($_.release_version)`n$($_.path)`n$($_.sha256)"
        }) -join "`n"
        $runtimeIdentityText
    ) -join "`n"
    $lifecycleComponent = @($components | Where-Object {
        [string]$_.id -ceq "lifecycle"
    })[0]
    $providerExecutionFingerprints = [ordered]@{}
    foreach ($provider in @("none", "local-process", "docker-compose")) {
        $providerCapability = switch ($provider) {
            "none" { "" }
            "local-process" { [string]$lifecycleComponent.capability_fingerprints.local_process }
            "docker-compose" { [string]$lifecycleComponent.capability_fingerprints.docker_compose }
        }
        $providerExecutionFingerprints[$provider] = Get-TessaraPlatformSha256Text -Text ((@(
            "tessara.validation.platform-provider-execution"
            $executionFingerprintInput
            [string]$lifecycleComponent.capability_fingerprints.common
            [string]$lifecycleComponent.capability_fingerprints.orchestration
            $provider
            $providerCapability
        ) -join "`n") + "`n")
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.platform"
        release_version = [string]$manifest.release_version
        platform_fingerprint = Get-TessaraPlatformSha256Text -Text ($fingerprintInput + "`n")
        execution_fingerprint = Get-TessaraPlatformSha256Text `
            -Text ($executionFingerprintInput + "`n")
        provider_execution_fingerprints = [pscustomobject]$providerExecutionFingerprints
        finalization_fingerprint = Get-TessaraPlatformSha256Text `
            -Text ($finalizationFingerprintInput + "`n")
        manifest_sha256 = $manifestHash
        runtime_identity = $runtimeIdentity
        boundary_inputs = @($boundaryInputs)
        components = @($components)
    }
}

function Assert-TessaraValidationAdapter {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$AdapterPath,
        [string]$RepositoryRoot = $script:RepositoryRoot
    )
    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $resolvedAdapter = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $repository `
        -Path $AdapterPath -Label "Validation adapter"
    if (-not (Test-Path -LiteralPath $resolvedAdapter -PathType Leaf)) {
        throw "Validation adapter is missing: $resolvedAdapter"
    }
    $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $repository `
        -Path $resolvedAdapter -Label "Validation adapter"
    $adapterSnapshot = Read-TessaraPlatformUtf8Snapshot -Path $resolvedAdapter `
        -Label "Validation adapter"
    $raw = [string]$adapterSnapshot.text
    $adapterRawHash = [string]$adapterSnapshot.sha256
    $adapterSchemaSnapshot = Read-TessaraPlatformUtf8Snapshot `
        -Path $script:AdapterSchemaPath -Label "Validation-adapter schema"
    $schemaErrors = @()
    if (-not (Test-Json -Json $raw -Schema ([string]$adapterSchemaSnapshot.text) `
            -ErrorVariable schemaErrors -ErrorAction SilentlyContinue)) {
        $message = @($schemaErrors | ForEach-Object { $_.Exception.Message }) -join " "
        throw "Validation adapter does not satisfy contract v2. $message"
    }
    $adapter = $raw | ConvertFrom-Json -Depth 100
    $contractPath = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $repository `
        -Path ([string]$adapter.validation_contract_path) -Label "Validation contract"
    if (-not (Test-Path -LiteralPath $contractPath -PathType Leaf)) {
        throw "Validation contract is missing: $contractPath"
    }
    $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $repository `
        -Path $contractPath -Label "Validation contract"
    $contractSnapshot = Read-TessaraPlatformUtf8Snapshot -Path $contractPath `
        -Label "Validation contract"
    $validationContractRaw = [string]$contractSnapshot.text
    $validationContract = $validationContractRaw | ConvertFrom-Json -Depth 100
    try {
        $null = Assert-TessaraValidationContract -Contract $validationContract
    } catch {
        throw "Governing validation contract is invalid. $($_.Exception.Message)"
    }
    $null = Assert-TessaraPlatformEvidenceRootContract -RepositoryRoot $repository `
        -DeclaredPath ([string]$validationContract.evidence_policy.root)
    $contractLaneIds = @($validationContract.lanes | ForEach-Object { [string]$_.id })
    $adapterLaneIds = @($adapter.lanes | ForEach-Object { [string]$_.id })
    if ($contractLaneIds.Count -eq 0 -or
        ($contractLaneIds | Sort-Object) -join "`n" -cne
            (($adapterLaneIds | Sort-Object) -join "`n")) {
        throw "Validation adapter lanes do not exactly cover the governing validation contract."
    }
    if (@($adapterLaneIds | Sort-Object -Unique).Count -ne $adapterLaneIds.Count) {
        throw "Validation adapter lane IDs are not unique."
    }
    if (@($contractLaneIds | Sort-Object -Unique).Count -ne $contractLaneIds.Count) {
        throw "Governing validation contract lane IDs are not unique."
    }
    $contractTargetMap = @{}
    foreach ($target in @($validationContract.implementation_targets)) {
        $contractTargetMap[[string]$target.id] = $target
    }
    $allowedTokens = @(
        "repository_root", "attempt_root", "candidate_fingerprint",
        "acceptance_fingerprint", "adapter_fingerprint", "platform_fingerprint",
        "compose_project"
    )
    foreach ($lane in @($adapter.lanes)) {
        $actionIds = @($lane.actions | ForEach-Object { [string]$_.id })
        if (@($actionIds | Sort-Object -Unique).Count -ne $actionIds.Count) {
            throw "Validation adapter lane '$($lane.id)' has duplicate action IDs."
        }
        $declaredActionSeconds = [long](@($lane.actions | Measure-Object `
                -Property timeout_seconds -Sum).Sum)
        if ($declaredActionSeconds -gt 14400) {
            throw "Validation adapter lane '$($lane.id)' exceeds the 14,400-second aggregate action budget."
        }
        $readinessSeconds = if ([string]$lane.topology.provider -ceq "none") {
            0
        } else { [int]$lane.topology.readiness.timeout_seconds }
        if (($declaredActionSeconds + $readinessSeconds) -gt
            [int]$lane.deadline_seconds) {
            throw "Validation adapter lane '$($lane.id)' declared work exceeds its lane deadline."
        }
        $isHandoffProducer = [string]$lane.topology.provider -ceq "docker-compose" -and
            [string]$lane.topology.mode -ceq "create" -and
            [string]$lane.topology.on_success -ceq "handoff"
        if (-not $isHandoffProducer -and
            @($lane.actions | Where-Object { [string]$_.stage -ceq "assertion" }).Count -eq 0) {
            throw "Validation adapter terminal lane '$($lane.id)' must declare at least one product assertion action."
        }
        $environmentNames = @($lane.environment | ForEach-Object { [string]$_.name })
        $blockedEnvironmentNames = @($lane.blocked_environment | ForEach-Object { [string]$_ })
        $reservedEnvironmentNames = @(
            "PATH", "PATHEXT", "SystemRoot", "WINDIR", "ComSpec", "TEMP", "TMP",
            "TMPDIR", "HOME", "USERPROFILE", "HOMEDRIVE", "HOMEPATH", "LOCALAPPDATA",
            "APPDATA", "PROGRAMDATA", "ProgramFiles", "ProgramFiles(x86)", "ProgramW6432",
            "PSModulePath", "NUMBER_OF_PROCESSORS", "PROCESSOR_ARCHITECTURE", "OS",
            "LANG", "LC_ALL", "COMPOSE_PROJECT_NAME",
             "DOCKER_CONFIG", "TESSARA_VALIDATION_TOPOLOGY_LEASE",
             "TESSARA_VALIDATION_TOPOLOGY_ATTEMPT",
             "TESSARA_VALIDATION_READINESS_CAPABILITY"
        )
        $topologyPorts = @(if ($lane.topology.PSObject.Properties.Name -contains "ports") {
            @($lane.topology.ports)
        } else { @() })
        $portNames = @($topologyPorts | ForEach-Object { [string]$_.name })
        $portEnvironmentNames = @($topologyPorts | ForEach-Object { [string]$_.environment })
        $laneEnvironmentNames = @($environmentNames) + @($blockedEnvironmentNames) +
            @($portEnvironmentNames)
        if (@($laneEnvironmentNames | Sort-Object -Unique).Count -ne $laneEnvironmentNames.Count) {
            throw "Validation adapter lane '$($lane.id)' reuses an environment variable."
        }
        if (@($laneEnvironmentNames | Where-Object {
                    [string]$_ -in $reservedEnvironmentNames
                }).Count -ne 0) {
            throw "Validation adapter lane '$($lane.id)' overrides or blocks a platform-reserved environment variable."
        }
        if (@($portNames | Sort-Object -Unique).Count -ne $portNames.Count) {
            throw "Validation adapter lane '$($lane.id)' has duplicate port names."
        }
        $contractLane = @($validationContract.lanes | Where-Object {
            [string]$_.id -ceq [string]$lane.id
        })[0]
        foreach ($candidateValue in @($lane.actions.program) + @($lane.actions.arguments)) {
            foreach ($tokenMatch in [regex]::Matches([string]$candidateValue, '\$\{([^}]+)\}')) {
                $candidateToken = [string]$tokenMatch.Groups[1].Value
                if (-not $candidateToken.StartsWith("port:") -and
                    $candidateToken -notin $allowedTokens) {
                    throw "Validation adapter lane '$($lane.id)' references unknown token '$candidateToken'."
                }
            }
        }
        $applicableTargetIds = @($validationContract.requirements | Where-Object {
            [string]$lane.id -in @($_.validation_lanes)
        } | ForEach-Object { @($_.implementation_targets) } | ForEach-Object {
            [string]$_
        } | Sort-Object -Unique)
        $assertionActions = @($lane.actions | Where-Object {
            [string]$_.stage -ceq "assertion"
        })
        $assertionTargetIds = @($assertionActions | ForEach-Object {
            [string]$_.implementation_target
        })
        if (@($assertionTargetIds | Sort-Object -Unique).Count -ne
            $assertionTargetIds.Count) {
            throw "Validation adapter lane '$($lane.id)' executes an implementation target more than once."
        }
        foreach ($action in $assertionActions) {
            $targetId = [string]$action.implementation_target
            Assert-TessaraPlatformDirectInterpreterInvocation `
                -Label "Validation adapter lane '$($lane.id)' action '$($action.id)'" `
                -Program ([string]$action.program) `
                -Arguments @($action.arguments | ForEach-Object { [string]$_ }) `
                -InputPaths @($action.input_paths | ForEach-Object { [string]$_ }) `
                -RepositoryRoot $repository
            if ($targetId -notin $applicableTargetIds -or
                -not $contractTargetMap.ContainsKey($targetId)) {
                throw "Validation adapter lane '$($lane.id)' assertion '$($action.id)' maps an undeclared implementation target '$targetId'."
            }
            $target = $contractTargetMap[$targetId]
            if ($target.command -is [string]) {
                throw "Validation adapter target '$targetId' uses a legacy command string; platform execution requires one structured exact command."
            }
            $actionCommand = [pscustomobject][ordered]@{
                program = [string]$action.program
                arguments = @($action.arguments | ForEach-Object { [string]$_ })
                input_paths = @($action.input_paths | ForEach-Object { [string]$_ })
                tools = @($action.tools)
            }
            if ((Get-TessaraPlatformCanonicalJsonSha256 -Value $actionCommand) -cne
                (Get-TessaraPlatformCanonicalJsonSha256 -Value $target.command)) {
                throw "Validation adapter lane '$($lane.id)' assertion '$($action.id)' does not execute the exact command declared by target '$targetId'."
            }
            $actionProofs = @($action.proof_classes | ForEach-Object { [string]$_ } | Sort-Object)
            $targetProofs = @($target.proof_classes | ForEach-Object { [string]$_ } | Sort-Object)
            if (($actionProofs -join "`n") -cne ($targetProofs -join "`n")) {
                throw "Validation adapter lane '$($lane.id)' assertion '$($action.id)' does not cover the exact proof classes of target '$targetId'."
            }
            $targetDomains = @($target.dependency_domains | ForEach-Object { [string]$_ })
            if (@($targetDomains | Where-Object {
                        [string]$_ -notin @($contractLane.dependency_domains)
                    }).Count -ne 0) {
                throw "Validation adapter lane '$($lane.id)' cannot execute target '$targetId' outside the target dependency cone."
            }
        }
        if (-not $isHandoffProducer) {
            $requiredTargetIds = @($applicableTargetIds | Where-Object {
                [bool]$contractTargetMap[[string]$_].required
            } | Sort-Object)
            if (($requiredTargetIds -join "`n") -cne
                (@($assertionTargetIds | Sort-Object) -join "`n")) {
                throw "Validation adapter terminal lane '$($lane.id)' does not execute its exact required implementation-target set."
            }
        }
        $contractPrerequisites = @($contractLane.prerequisites | ForEach-Object { [string]$_ } | Sort-Object)
        $adapterPrerequisites = @($lane.prerequisites | ForEach-Object { [string]$_ } | Sort-Object)
        if (($contractPrerequisites -join "`n") -cne ($adapterPrerequisites -join "`n")) {
            throw "Validation adapter lane '$($lane.id)' prerequisites do not exactly match the governing validation contract."
        }
        if ([string]$lane.topology.provider -ne "none" -and
            [string]$lane.topology.readiness.port -notin $portNames) {
            throw "Validation adapter lane '$($lane.id)' readiness references an unknown port."
        }
        if ([string]$lane.topology.on_success -ceq "handoff") {
            $successor = @($adapter.lanes | Where-Object {
                [string]$_.id -ceq [string]$lane.topology.handoff_to
            })
            $successorPortNames = if ($successor.Count -eq 1 -and
                $successor[0].topology.PSObject.Properties.Name -contains "ports") {
                @($successor[0].topology.ports | ForEach-Object { [string]$_.name } |
                    Sort-Object)
            } else { @() }
            if ([string]$lane.topology.provider -cne "docker-compose" -or
                [string]$lane.topology.mode -cne "create" -or
                $successor.Count -ne 1 -or
                [string]$successor[0].topology.mode -cne "consume" -or
                [string]$successor[0].topology.from_lane -cne [string]$lane.id -or
                [string]$lane.id -notin @($successor[0].prerequisites) -or
                ((@($portNames | Sort-Object) -join "`n") -cne
                    ($successorPortNames -join "`n"))) {
                throw "Validation adapter lane '$($lane.id)' does not target one matching consume successor."
            }
        }
        if ([string]$lane.topology.mode -ceq "consume") {
            $source = @($adapter.lanes | Where-Object {
                [string]$_.id -ceq [string]$lane.topology.from_lane
            })
            if ($source.Count -ne 1 -or
                [string]$source[0].topology.provider -cne "docker-compose" -or
                [string]$source[0].topology.mode -cne "create" -or
                [string]$source[0].topology.on_success -cne "handoff" -or
                -not ($source[0].topology.PSObject.Properties.Name -contains "handoff_to") -or
                [string]$source[0].topology.handoff_to -cne [string]$lane.id -or
                [string]$lane.topology.from_lane -notin @($lane.prerequisites)) {
                throw "Validation adapter lane '$($lane.id)' does not consume one declared prerequisite handoff."
            }
        }
        $topologyCommandValues = if (
            $lane.topology.PSObject.Properties.Name -contains "command"
        ) {
            @($lane.topology.command.program) + @($lane.topology.command.arguments)
        } else { @() }
        $environmentValues = @($lane.environment | Where-Object {
            $_.PSObject.Properties.Name -contains "value"
        } | ForEach-Object { [string]$_.value })
        $values = @($environmentValues) + @($lane.actions.program) +
            @($lane.actions.arguments) + @($topologyCommandValues)
        foreach ($value in @($values | Where-Object { $null -ne $_ })) {
            foreach ($match in [regex]::Matches([string]$value, '\$\{([^}]+)\}')) {
                $token = $match.Groups[1].Value
                if ($token -ceq "candidate_fingerprint" -and
                    [string]$contractLane.phase -in @(
                        "validation-readiness", "candidate-rehearsal"
                    )) {
                    throw "Validation adapter candidate-neutral lane '$($lane.id)' uses candidate_fingerprint."
                }
                if ($token.StartsWith("port:")) {
                    if ($token.Substring(5) -notin $portNames) {
                        throw "Validation adapter lane '$($lane.id)' references unknown port token '$token'."
                    }
                } elseif ($token -notin $allowedTokens) {
                    throw "Validation adapter lane '$($lane.id)' references unknown token '$token'."
                }
            }
        }
        $stageOrder = @{ setup = 0; assertion = 1 }
        $priorStage = -1
        foreach ($action in @($lane.actions)) {
            $toolIds = @($action.tools | ForEach-Object { [string]$_.id })
            if (@($toolIds | Sort-Object -Unique).Count -ne $toolIds.Count) {
                throw "Validation adapter lane '$($lane.id)' action '$($action.id)' has duplicate transitive tool IDs."
            }
            $stage = [string]$action.stage
            if ($stageOrder[$stage] -lt $priorStage) {
                throw "Validation adapter lane '$($lane.id)' action stages are not monotonic."
            }
            $priorStage = $stageOrder[$stage]
            if ([string]$action.program -match '\$\{(?!repository_root\})[^}]+\}') {
                throw "Validation adapter lane '$($lane.id)' action '$($action.id)' uses a dynamic program placeholder."
            }
            Assert-TessaraPlatformDirectInterpreterInvocation `
                -Label "Validation adapter lane '$($lane.id)' action '$($action.id)'" `
                -Program ([string]$action.program) `
                -Arguments @($action.arguments | ForEach-Object { [string]$_ }) `
                -InputPaths @($action.input_paths | ForEach-Object { [string]$_ }) `
                -RepositoryRoot $repository
        }
        if ($lane.topology.PSObject.Properties.Name -contains "command") {
            if ([string]$lane.topology.command.program -match '\$\{(?!repository_root\})[^}]+\}') {
                throw "Validation adapter lane '$($lane.id)' topology uses a dynamic program placeholder."
            }
            Assert-TessaraPlatformDirectInterpreterInvocation `
                -Label "Validation adapter lane '$($lane.id)' topology" `
                -Program ([string]$lane.topology.command.program) `
                -Arguments @($lane.topology.command.arguments | ForEach-Object { [string]$_ }) `
                -InputPaths @($lane.topology.command.input_paths | ForEach-Object { [string]$_ }) `
                -RepositoryRoot $repository
        }
    }
    foreach ($lane in @($adapter.lanes)) {
        foreach ($prerequisite in @($lane.prerequisites)) {
            if ([string]$prerequisite -notin $adapterLaneIds) {
                throw "Validation adapter lane '$($lane.id)' references unknown prerequisite '$prerequisite'."
            }
        }
    }
    function Visit-AdapterLane {
        param([string]$Id, [hashtable]$Visiting, [hashtable]$Visited)
        if ($Visiting.ContainsKey($Id)) { throw "Validation adapter prerequisite graph contains a cycle at '$Id'." }
        if ($Visited.ContainsKey($Id)) { return }
        $Visiting[$Id] = $true
        $current = @($adapter.lanes | Where-Object { [string]$_.id -ceq $Id })[0]
        foreach ($prerequisite in @($current.prerequisites)) {
            Visit-AdapterLane -Id ([string]$prerequisite) -Visiting $Visiting -Visited $Visited
        }
        $Visiting.Remove($Id)
        $Visited[$Id] = $true
    }
    $visited = @{}
    foreach ($laneId in $adapterLaneIds) { Visit-AdapterLane -Id $laneId -Visiting @{} -Visited $visited }

    $inputInventories = [ordered]@{}
    foreach ($inputKind in @("acceptance_inputs", "fixture_inputs", "execution_inputs")) {
        $entries = [Collections.Generic.List[object]]::new()
        $seenPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($declaration in @($adapter.$inputKind)) {
            $path = [string]$declaration.path
            if (-not $seenPaths.Add($path)) {
                throw "Validation adapter $inputKind contains duplicate path '$path'."
            }
            $declaredLanes = @($declaration.lanes | ForEach-Object { [string]$_ })
            foreach ($declaredLane in $declaredLanes) {
                if ($declaredLane -notin $adapterLaneIds) {
                    throw "Validation adapter $inputKind path '$path' references unknown lane '$declaredLane'."
                }
            }
            $resolved = Resolve-TessaraPlatformRepositoryPath -RepositoryRoot $repository `
                -Path $path -Label ($inputKind.Replace('_', ' '))
            if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
                throw "Validation adapter $inputKind input is missing: $resolved"
            }
            $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $repository `
                -Path $resolved -Label "Validation adapter $inputKind input"
            $entries.Add([pscustomobject][ordered]@{
                path = $path
                lanes = @($declaredLanes)
                sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $resolved).Hash.ToLowerInvariant()
            })
        }
        $inputInventories[$inputKind] = @($entries)
    }
    foreach ($laneId in $adapterLaneIds) {
        if (@($inputInventories.acceptance_inputs | Where-Object {
                $laneId -in @($_.lanes)
            }).Count -eq 0) {
            throw "Validation adapter lane '$laneId' has no owned acceptance input."
        }
        if (@($inputInventories.execution_inputs | Where-Object {
                $laneId -in @($_.lanes)
            }).Count -eq 0) {
            throw "Validation adapter lane '$laneId' has no owned execution input."
        }
        $laneDocument = @($adapter.lanes | Where-Object { [string]$_.id -ceq $laneId })[0]
        $ownedPaths = @($inputInventories.execution_inputs | Where-Object {
            $laneId -in @($_.lanes)
        } | ForEach-Object { [string]$_.path })
        $referencedPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        if ($laneDocument.topology.PSObject.Properties.Name -contains "compose_file") {
            [void]$referencedPaths.Add([string]$laneDocument.topology.compose_file)
        }
        if ($laneDocument.topology.PSObject.Properties.Name -contains "command") {
            foreach ($inputPath in @($laneDocument.topology.command.input_paths)) {
                [void]$referencedPaths.Add([string]$inputPath)
            }
            foreach ($tool in @($laneDocument.topology.command.tools)) {
                foreach ($inputPath in @($tool.input_paths)) {
                    [void]$referencedPaths.Add([string]$inputPath)
                }
            }
        }
        foreach ($action in @($laneDocument.actions)) {
            foreach ($inputPath in @($action.input_paths)) {
                [void]$referencedPaths.Add([string]$inputPath)
            }
            foreach ($tool in @($action.tools)) {
                foreach ($inputPath in @($tool.input_paths)) {
                    [void]$referencedPaths.Add([string]$inputPath)
                }
            }
        }
        foreach ($referencedPath in $referencedPaths) {
            if ([string]$referencedPath -notin $ownedPaths) {
                throw "Validation adapter lane '$laneId' declares command input '$referencedPath' outside its execution-input inventory."
            }
        }
    }
    Assert-TessaraPlatformSnapshotCurrent -Snapshot $adapterSnapshot -Label "Validation adapter"
    Assert-TessaraPlatformSnapshotCurrent -Snapshot $contractSnapshot -Label "Validation contract"
    Assert-TessaraPlatformSnapshotCurrent -Snapshot $adapterSchemaSnapshot `
        -Label "Validation-adapter schema"
    $validationContractHash = [string]$contractSnapshot.sha256
    $acceptanceText = @(
        "validation-contract`n$([string]$adapter.validation_contract_path)`n$validationContractHash"
        @($inputInventories.acceptance_inputs | Sort-Object path | ForEach-Object {
        "$($_.path)`n$($_.sha256)"
        })
    ) -join "`n"
    $executionText = @(
        @($inputInventories.fixture_inputs | Sort-Object path | ForEach-Object {
            "fixture`n$($_.path)`n$($_.sha256)"
        })
        @($inputInventories.execution_inputs | Sort-Object path | ForEach-Object {
            "execution`n$($_.path)`n$($_.sha256)"
        })
    ) -join "`n"
    $laneIdentities = [Collections.Generic.List[object]]::new()
    foreach ($lane in @($adapter.lanes)) {
        $laneId = [string]$lane.id
        $contractLane = @($validationContract.lanes | Where-Object {
            [string]$_.id -ceq $laneId
        })[0]
        $domainNames = @($contractLane.dependency_domains | ForEach-Object { [string]$_ })
        $contractDomains = @($validationContract.dependency_domains | Where-Object {
            [string]$_.name -in $domainNames
        } | Sort-Object name)
        $laneRequirements = @($validationContract.requirements | Where-Object {
            $laneId -in @($_.validation_lanes)
        } | Sort-Object id | ForEach-Object {
            [pscustomobject][ordered]@{
                id = [string]$_.id
                implementation_targets = @($_.implementation_targets | ForEach-Object { [string]$_ })
            }
        })
        $targetIds = @($laneRequirements.implementation_targets | ForEach-Object {
            @($_) | ForEach-Object { [string]$_ }
        } | Sort-Object -Unique)
        $contractTargets = @($validationContract.implementation_targets | Where-Object {
            [string]$_.id -in $targetIds
        } | Sort-Object id)
        $laneAcceptanceInputs = @($inputInventories.acceptance_inputs | Where-Object {
            $laneId -in @($_.lanes)
        } | Sort-Object path | ForEach-Object {
            [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
        })
        $laneFixtureInputs = @($inputInventories.fixture_inputs | Where-Object {
            $laneId -in @($_.lanes)
        } | Sort-Object path | ForEach-Object {
            [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
        })
        $laneExecutionInputs = @($inputInventories.execution_inputs | Where-Object {
            $laneId -in @($_.lanes)
        } | Sort-Object path | ForEach-Object {
            [pscustomobject][ordered]@{ path = [string]$_.path; sha256 = [string]$_.sha256 }
        })
        $contractSlice = [pscustomobject][ordered]@{
            schema_version = [int]$validationContract.schema_version
            contract = [string]$validationContract.contract
            policy_version = [string]$validationContract.policy_version
            sprint = [string]$validationContract.sprint
            implementation_profile = $validationContract.implementation_profile
            lane = $contractLane
            dependency_domains = @($contractDomains)
            requirements = @($laneRequirements)
            implementation_targets = @($contractTargets)
        }
        $adapterSlice = [pscustomobject][ordered]@{
            schema_version = [int]$adapter.schema_version
            contract = [string]$adapter.contract
            adapter_id = [string]$adapter.adapter_id
            lane = $lane
            fixture_inputs = @($laneFixtureInputs)
            execution_inputs = @($laneExecutionInputs)
        }
        $laneIdentities.Add([pscustomobject][ordered]@{
            lane_id = $laneId
            phase = [string]$contractLane.phase
            touches_live_state = [bool]$contractLane.touches_live_state
            dependency_domains = @($domainNames)
            dependency_domain_definitions = @($contractDomains)
            adapter_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value $adapterSlice
            acceptance_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value (
                [pscustomobject][ordered]@{
                    contract_slice = $contractSlice
                    acceptance_inputs = @($laneAcceptanceInputs)
                }
            )
            fixture_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 `
                -Value ([pscustomobject][ordered]@{ inputs = @($laneFixtureInputs) })
            harness_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 `
                -Value ([pscustomobject][ordered]@{ inputs = @($laneExecutionInputs) })
            environment_contract_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 `
                -Value ([pscustomobject][ordered]@{
                    environment = @($lane.environment)
                    ports = @(if ($lane.topology.PSObject.Properties.Name -contains "ports") {
                        @($lane.topology.ports)
                    } else { @() })
                })
            contract_lane_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value $contractSlice
            acceptance_inputs = @($laneAcceptanceInputs)
            fixture_inputs = @($laneFixtureInputs)
            execution_inputs = @($laneExecutionInputs)
            adapter_source = [pscustomobject][ordered]@{
                path = $resolvedAdapter
                sha256 = $adapterRawHash
            }
            validation_contract_source = [pscustomobject][ordered]@{
                path = $contractPath
                sha256 = $validationContractHash
            }
        })
    }
    [pscustomobject][ordered]@{
        adapter = $adapter
        adapter_path = $resolvedAdapter
        adapter_fingerprint = $adapterRawHash
        adapter_execution_fingerprint = Get-TessaraPlatformSha256Text -Text ($executionText + "`n")
        acceptance_fingerprint = Get-TessaraPlatformSha256Text -Text ($acceptanceText + "`n")
        acceptance_inputs = @($inputInventories.acceptance_inputs)
        fixture_inputs = @($inputInventories.fixture_inputs)
        execution_inputs = @($inputInventories.execution_inputs)
        lane_identities = @($laneIdentities)
        validation_contract = $validationContract
        validation_contract_path = $contractPath
        validation_contract_sha256 = $validationContractHash
    }
}

function Get-TessaraPlatformCandidateIdentityFromValidation {
    param(
        [Parameter(Mandatory)]$ValidatedAdapter,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $gitPath = Assert-TessaraPlatformGitExecutableCurrent
    $commitResult = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $repository -Arguments @("-C", $repository, "rev-parse", "HEAD")
    $treeResult = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $repository -Arguments @("-C", $repository, "rev-parse", "HEAD^{tree}")
    $statusResult = Invoke-TessaraPlatformIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $repository `
        -Arguments @("-C", $repository, "status", "--porcelain=v1", "-z", "--untracked-files=all")
    if ([int]$commitResult.exit_code -ne 0 -or [int]$treeResult.exit_code -ne 0 -or
        [int]$statusResult.exit_code -ne 0) {
        throw "Unable to authenticate the repository source identity for the validation candidate."
    }
    $commit = ([string]$commitResult.stdout).Trim()
    $tree = ([string]$treeResult.stdout).Trim()
    if ($commit -cnotmatch '^[0-9a-f]{40,64}$' -or
        $tree -cnotmatch '^[0-9a-f]{40,64}$') {
        throw "The repository returned an invalid commit or tree identity."
    }
    $dependencyFingerprints = Get-TessaraPlatformLiveDependencyFingerprints `
        -ValidationContract $ValidatedAdapter.validation_contract `
        -RepositoryRoot $repository
    $source = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.candidate-source"
        commit = $commit
        tree = $tree
        dirty = -not [string]::IsNullOrEmpty([string]$statusResult.stdout)
        status_fingerprint = Get-TessaraPlatformSha256Text -Text ([string]$statusResult.stdout)
        validation_contract_sha256 = [string]$ValidatedAdapter.validation_contract_sha256
        dependency_fingerprints = @($dependencyFingerprints | Sort-Object domain)
    }
    [pscustomobject][ordered]@{
        source_identity = $source
        candidate_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value $source
    }
}

function Get-TessaraValidationCandidateIdentity {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$AdapterPath,
        [string]$RepositoryRoot = $script:RepositoryRoot
    )
    $validated = Assert-TessaraValidationAdapter -AdapterPath $AdapterPath `
        -RepositoryRoot $RepositoryRoot
    Get-TessaraPlatformCandidateIdentityFromValidation -ValidatedAdapter $validated `
        -RepositoryRoot $RepositoryRoot
}

function Invoke-TessaraValidationLane {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$AdapterPath,
        [Parameter(Mandatory)][string]$LaneId,
        [Parameter(Mandatory)][ValidatePattern("^[0-9a-f]{64}$")][string]$CandidateFingerprint,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [string]$RepositoryRoot = $script:RepositoryRoot,
        [string]$TopologyReceiptPath,
        [AllowEmptyCollection()][string[]]$PrerequisiteResultPaths = @(),
        [string]$FinalizeCheckpointPath,
        [string]$CancellationPath,
        [string[]]$DockerCommand = @("docker"),
        [AllowEmptyCollection()][string[]]$DockerCommandInputPaths = @(),
        [Parameter(DontShow)][switch]$PlanOnly,
        [Parameter(DontShow)]
        [ValidateSet(
            "local-topology-after-start",
            "assertion-after-process-acquisition",
            "assertion-cancellation-after-process-acquisition",
            "after-post-setup-verification",
            "after-consumer-claim",
            "after-handoff-finalization",
            "execution-revocation-publication",
            "partial-claim-publication",
            "consumer-compose-config-substitution"
        )]
        [string]$CertificationFault
    )
    if ($DockerCommand.Count -eq 0) {
        throw "DockerCommand must contain one direct program."
    }
    if ($DockerCommand.Count -gt 1 -and $DockerCommandInputPaths.Count -eq 0) {
        throw "A Docker command prefix with arguments requires explicit repository input paths."
    }
    [string[]]$dockerPrefixArguments = @()
    if ($DockerCommand.Count -gt 1) {
        $dockerPrefixArguments = @($DockerCommand[1..($DockerCommand.Count - 1)])
    }
    Assert-TessaraPlatformDirectInterpreterInvocation -Label "Docker command prefix" `
        -Program ([string]$DockerCommand[0]) `
        -Arguments $dockerPrefixArguments -InputPaths $DockerCommandInputPaths `
        -RepositoryRoot $RepositoryRoot
    $platform = Get-TessaraValidationPlatformIdentity
    $validated = Assert-TessaraValidationAdapter -AdapterPath $AdapterPath `
        -RepositoryRoot $RepositoryRoot
    $lane = @($validated.adapter.lanes | Where-Object { [string]$_.id -ceq $LaneId })
    if ($lane.Count -ne 1) { throw "Validation adapter does not declare exact lane '$LaneId'." }
    $candidateIdentity = Get-TessaraPlatformCandidateIdentityFromValidation `
        -ValidatedAdapter $validated -RepositoryRoot $RepositoryRoot
    $candidateBound = [string](@($validated.lane_identities | Where-Object {
        [string]$_.lane_id -ceq $LaneId
    })[0].phase) -in @("validation-preflight", "sit", "uat")
    if ($candidateBound -and $CandidateFingerprint -cne
        [string]$candidateIdentity.candidate_fingerprint) {
        throw "Candidate fingerprint is not authenticated by the current source and dependency snapshot."
    }
    $CandidateFingerprint = [string]$candidateIdentity.candidate_fingerprint
    $declaredEvidenceRoot = Assert-TessaraPlatformEvidenceRootContract `
        -RepositoryRoot $RepositoryRoot `
        -DeclaredPath ([string]$validated.validation_contract.evidence_policy.root)
    $resolvedEvidenceRoot = if ([IO.Path]::IsPathRooted($EvidenceRoot)) {
        [IO.Path]::GetFullPath($EvidenceRoot)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $EvidenceRoot))
    }
    $evidenceRelative = [IO.Path]::GetRelativePath($declaredEvidenceRoot, $resolvedEvidenceRoot)
    if ([IO.Path]::IsPathRooted($evidenceRelative) -or
        $evidenceRelative -eq ".." -or
        $evidenceRelative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
        $evidenceRelative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
        throw "EvidenceRoot must equal or be contained by the governing validation-contract evidence root."
    }
    $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $RepositoryRoot `
        -Path $resolvedEvidenceRoot -Label "EvidenceRoot"
    $dependencyFingerprints = Get-TessaraPlatformLiveDependencyFingerprints `
        -ValidationContract $validated.validation_contract -RepositoryRoot $RepositoryRoot
    $baseLaneIdentityMap = @{}
    foreach ($baseIdentity in @($validated.lane_identities)) {
        $baseLaneIdentityMap[[string]$baseIdentity.lane_id] = $baseIdentity
    }
    $resolvedLaneIdentityMap = @{}
    function Resolve-ValidationLaneIdentity {
        param([Parameter(Mandatory)][string]$IdentityLaneId, [hashtable]$Visiting)
        if ($resolvedLaneIdentityMap.ContainsKey($IdentityLaneId)) {
            return $resolvedLaneIdentityMap[$IdentityLaneId]
        }
        if ($Visiting.ContainsKey($IdentityLaneId)) {
            throw "Lane compatibility identity recursion encountered a cycle at '$IdentityLaneId'."
        }
        $Visiting[$IdentityLaneId] = $true
        $laneIdentity = $baseLaneIdentityMap[$IdentityLaneId]
        $laneDomains = @($dependencyFingerprints | Where-Object {
            [string]$_.domain -in @($laneIdentity.dependency_domains)
        } | Sort-Object domain)
        $adapterLane = @($validated.adapter.lanes | Where-Object {
            [string]$_.id -ceq $IdentityLaneId
        })[0]
        $environmentObservationFingerprint = `
            Get-TessaraPlatformLaneEnvironmentObservationFingerprint -Lane $adapterLane
        $toolObservation = Get-TessaraPlatformLaneToolObservation -Lane $adapterLane `
            -RepositoryRoot $RepositoryRoot -DockerCommand $DockerCommand `
            -DockerCommandInputPaths $DockerCommandInputPaths
        $prerequisiteIdentities = @($adapterLane.prerequisites | ForEach-Object {
            Resolve-ValidationLaneIdentity -IdentityLaneId ([string]$_) -Visiting $Visiting
        } | Sort-Object lane_id | ForEach-Object {
            [pscustomobject][ordered]@{
                lane_id = [string]$_.lane_id
                compatibility_fingerprint = [string]$_.compatibility_fingerprint
            }
        })
        $candidateBinding = if ([string]$laneIdentity.phase -in @(
                "validation-preflight", "sit", "uat"
            )) { $CandidateFingerprint } else { $null }
        $compatibility = [pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.lane-compatibility"
            lane_id = [string]$laneIdentity.lane_id
            phase = [string]$laneIdentity.phase
            candidate_fingerprint = $candidateBinding
            contract_lane_fingerprint = [string]$laneIdentity.contract_lane_fingerprint
            dependency_fingerprints = @($laneDomains)
            acceptance_fingerprint = [string]$laneIdentity.acceptance_fingerprint
            fixture_fingerprint = [string]$laneIdentity.fixture_fingerprint
            harness_fingerprint = [string]$laneIdentity.harness_fingerprint
            adapter_fingerprint = [string]$laneIdentity.adapter_fingerprint
            environment_contract_fingerprint = [string]$laneIdentity.environment_contract_fingerprint
            environment_observation_fingerprint = $environmentObservationFingerprint
            tool_observation_fingerprint = [string]$toolObservation.fingerprint
            platform_execution_fingerprint = [string]$platform.provider_execution_fingerprints.([string]$adapterLane.topology.provider)
            prerequisite_compatibility_fingerprints = @($prerequisiteIdentities)
        }
        $copy = [ordered]@{}
        foreach ($property in $laneIdentity.PSObject.Properties) {
            $copy[$property.Name] = $property.Value
        }
        $copy.dependency_fingerprints = @($laneDomains)
        $copy.candidate_binding = $candidateBinding
        $copy.candidate_source_identity = $candidateIdentity.source_identity
        $copy.environment_observation_fingerprint = $environmentObservationFingerprint
        $copy.tool_observation_fingerprint = [string]$toolObservation.fingerprint
        $copy.tool_observations = @($toolObservation.programs)
        $copy.platform_execution_fingerprint = [string]$platform.provider_execution_fingerprints.([string]$adapterLane.topology.provider)
        $copy.docker_command_prefix_fingerprint = [string]$toolObservation.docker_prefix_fingerprint
        $copy.prerequisite_compatibility_fingerprints = @($prerequisiteIdentities)
        $copy.compatibility_fingerprint = Get-TessaraPlatformCanonicalJsonSha256 `
            -Value $compatibility
        $resolved = [pscustomobject]$copy
        $resolvedLaneIdentityMap[$IdentityLaneId] = $resolved
        $Visiting.Remove($IdentityLaneId)
        return $resolved
    }
    $resolvedLaneIdentities = @($validated.lane_identities | ForEach-Object {
        Resolve-ValidationLaneIdentity -IdentityLaneId ([string]$_.lane_id) -Visiting @{}
    })
    $declaredLanes = @($resolvedLaneIdentities | Sort-Object phase, lane_id | ForEach-Object {
        [pscustomobject][ordered]@{
            lane_id = [string]$_.lane_id
            phase = [string]$_.phase
            compatibility_fingerprint = [string]$_.compatibility_fingerprint
            platform_execution_fingerprint = [string]$_.platform_execution_fingerprint
            environment_fingerprint = [string]$_.environment_observation_fingerprint
            dependency_fingerprints = @($_.dependency_fingerprints)
            prerequisite_compatibility_fingerprints = @($_.prerequisite_compatibility_fingerprints)
            implementation_targets = @()
        }
    })
    foreach ($planLane in $declaredLanes) {
        $laneRequirements = @($validated.validation_contract.requirements | Where-Object {
            [string]$planLane.lane_id -in @($_.validation_lanes)
        })
        $planLane.implementation_targets = @($laneRequirements | ForEach-Object {
            @($_.implementation_targets)
        } | ForEach-Object { [string]$_ } | Sort-Object -Unique)
    }
    $planBody = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.compatibility-plan"
        sprint = [string]$validated.validation_contract.sprint
        source_identity = $candidateIdentity.source_identity
        candidate_fingerprint = [string]$candidateIdentity.candidate_fingerprint
        validation_contract = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath(
                [IO.Path]::GetFullPath($RepositoryRoot),
                [string]$validated.validation_contract_path
            ).Replace('\', '/')
            sha256 = [string]$validated.validation_contract_sha256
        }
        adapter = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath(
                [IO.Path]::GetFullPath($RepositoryRoot),
                [string]$validated.adapter_path
            ).Replace('\', '/')
            sha256 = [string]$validated.adapter_fingerprint
        }
        platform_execution_fingerprint = [string]$platform.execution_fingerprint
        lanes = @($declaredLanes)
    }
    $compatibilityPlan = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.compatibility-plan"
        fingerprint = Get-TessaraPlatformCanonicalJsonSha256 -Value $planBody
        body = $planBody
    }
    if ($PlanOnly) { return $compatibilityPlan }
    $selectedLaneIdentity = @($resolvedLaneIdentities | Where-Object {
        [string]$_.lane_id -ceq $LaneId
    })[0]
    $selectedLaneIdentity | Add-Member -NotePropertyName compatibility_plan_fingerprint `
        -NotePropertyValue ([string]$compatibilityPlan.fingerprint)
    foreach ($source in @(
            $selectedLaneIdentity.adapter_source,
            $selectedLaneIdentity.validation_contract_source
        )) {
        if (-not (Test-Path -LiteralPath ([string]$source.path) -PathType Leaf) -or
            (Get-FileHash -Algorithm SHA256 -LiteralPath ([string]$source.path)).Hash.ToLowerInvariant() -cne
                [string]$source.sha256) {
            throw "Validation adapter or governing contract changed before lifecycle entry."
        }
    }
    $platformRecheck = Get-TessaraValidationPlatformIdentity
    if ([string]$platformRecheck.platform_fingerprint -cne
        [string]$platform.platform_fingerprint) {
        throw "Validation-platform boundary changed during lane planning."
    }
    $admission = Enter-TessaraPlatformAdmission -EvidenceRoot $resolvedEvidenceRoot `
        -MaximumParallelLanes ([int]$validated.adapter.execution_policy.max_parallel_lanes)
    try {
        Assert-TessaraPlatformEvidenceBudget -EvidenceRoot $resolvedEvidenceRoot `
            -LaneId $LaneId `
            -MaximumAttempts ([int]$validated.adapter.execution_policy.max_attempts_per_lane) `
            -MaximumBytes ([long]$validated.adapter.execution_policy.max_evidence_bytes)
        Invoke-TessaraValidationLifecycleLane -Adapter $validated.adapter -Lane $lane[0] `
            -RepositoryRoot $RepositoryRoot -EvidenceRoot $resolvedEvidenceRoot `
            -CandidateFingerprint $CandidateFingerprint `
            -AggregateAcceptanceFingerprint $validated.acceptance_fingerprint `
            -AggregateAdapterFingerprint $validated.adapter_fingerprint `
            -LaneIdentity $selectedLaneIdentity -AllLaneIdentities $resolvedLaneIdentities `
            -PlatformIdentity $platform -TopologyReceiptPath $TopologyReceiptPath `
            -PrerequisiteResultPaths $PrerequisiteResultPaths `
            -FinalizeCheckpointPath $FinalizeCheckpointPath -CancellationPath $CancellationPath `
            -DockerCommand $DockerCommand -DockerCommandInputPaths $DockerCommandInputPaths `
            -CertificationFault $CertificationFault
    } finally {
        $admission.Dispose()
    }
}

function Get-TessaraValidationCompatibilityPlan {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$AdapterPath,
        [string]$RepositoryRoot = $script:RepositoryRoot,
        [string[]]$DockerCommand = @("docker"),
        [AllowEmptyCollection()][string[]]$DockerCommandInputPaths = @(),
        [string]$OutputPath
    )
    $validated = Assert-TessaraValidationAdapter -AdapterPath $AdapterPath `
        -RepositoryRoot $RepositoryRoot
    $candidate = Get-TessaraPlatformCandidateIdentityFromValidation `
        -ValidatedAdapter $validated -RepositoryRoot $RepositoryRoot
    $laneId = [string]@($validated.adapter.lanes)[0].id
    $evidenceRoot = [string]$validated.validation_contract.evidence_policy.root
    $plan = Invoke-TessaraValidationLane -AdapterPath $AdapterPath -LaneId $laneId `
        -CandidateFingerprint ([string]$candidate.candidate_fingerprint) `
        -EvidenceRoot $evidenceRoot -RepositoryRoot $RepositoryRoot `
        -DockerCommand $DockerCommand -DockerCommandInputPaths $DockerCommandInputPaths `
        -PlanOnly
    if ([string]::IsNullOrWhiteSpace($OutputPath)) { return $plan }
    $declaredRoot = Assert-TessaraPlatformEvidenceRootContract `
        -RepositoryRoot $RepositoryRoot -DeclaredPath $evidenceRoot
    $resolvedOutput = if ([IO.Path]::IsPathRooted($OutputPath)) {
        [IO.Path]::GetFullPath($OutputPath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $OutputPath))
    }
    $relative = [IO.Path]::GetRelativePath($declaredRoot, $resolvedOutput)
    if ([IO.Path]::IsPathRooted($relative) -or $relative -eq ".." -or
        $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
        $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
        throw "Compatibility-plan output must remain inside the governing evidence root."
    }
    $parent = Split-Path -Parent $resolvedOutput
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    $null = Assert-TessaraPlatformNoReparsePath -RepositoryRoot $RepositoryRoot `
        -Path $parent -Label "Compatibility-plan output directory"
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(
        (($plan | ConvertTo-Json -Depth 100 -Compress) + "`n")
    )
    $sha256 = Get-TessaraPlatformSha256Bytes -Bytes $bytes
    foreach ($publication in @(
            [pscustomobject]@{ path = $resolvedOutput; bytes = $bytes },
            [pscustomobject]@{
                path = "$resolvedOutput.sha256"
                bytes = [Text.UTF8Encoding]::new($false).GetBytes("$sha256`n")
            }
        )) {
        if (Test-Path -LiteralPath ([string]$publication.path)) {
            $existing = [IO.File]::ReadAllBytes([string]$publication.path)
            if ([Convert]::ToBase64String($existing) -cne
                [Convert]::ToBase64String([byte[]]$publication.bytes)) {
                throw "Compatibility-plan publication is immutable: $($publication.path)"
            }
            continue
        }
        $stream = [IO.FileStream]::new([string]$publication.path, [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write, [IO.FileShare]::None)
        try {
            $stream.Write([byte[]]$publication.bytes, 0, ([byte[]]$publication.bytes).Length)
            $stream.Flush($true)
        } catch {
            $stream.Dispose()
            Remove-Item -LiteralPath ([string]$publication.path) -Force -ErrorAction SilentlyContinue
            throw
        } finally {
            $stream.Dispose()
        }
    }
    [pscustomobject][ordered]@{
        plan = $plan
        reference = [pscustomobject][ordered]@{
            path = [IO.Path]::GetRelativePath([IO.Path]::GetFullPath($RepositoryRoot), $resolvedOutput).Replace('\', '/')
            sha256 = $sha256
        }
    }
}

Export-ModuleMember -Function @(
    "Get-TessaraValidationPlatformIdentity",
    "Get-TessaraValidationCandidateIdentity",
    "Get-TessaraValidationCompatibilityPlan",
    "Assert-TessaraValidationAdapter",
    "Invoke-TessaraValidationLane"
)

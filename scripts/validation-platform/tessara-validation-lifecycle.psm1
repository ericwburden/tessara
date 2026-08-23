Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:LifecycleContract = "tessara.validation.lifecycle"
$script:LifecycleRelease = "2.0.0"
$script:LifecycleModulePath = [IO.Path]::GetFullPath($PSCommandPath)
$script:LifecycleModuleSha256AtImport = (
    Get-FileHash -Algorithm SHA256 -LiteralPath $script:LifecycleModulePath
).Hash.ToLowerInvariant()
$script:FinalizerModulePath = Join-Path $PSScriptRoot "tessara-validation-finalizer.psm1"
$resolvedFinalizerModulePath = [IO.Path]::GetFullPath($script:FinalizerModulePath)
$loadedFinalizer = @(Get-Module | Where-Object {
    -not [string]::IsNullOrWhiteSpace($_.Path) -and
    [IO.Path]::GetFullPath($_.Path) -eq $resolvedFinalizerModulePath
}) | Select-Object -First 1
if ($null -eq $loadedFinalizer) {
    Import-Module $resolvedFinalizerModulePath
}

$script:BoundedCaptureImplementationId = "tessara.validation.bounded-capture.v1.1048576"
if ($null -eq ("Tessara.Validation.BoundedCaptureStream" -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace Tessara.Validation
{
    public sealed class BoundedCaptureStream : Stream
    {
        public const string ImplementationId = "tessara.validation.bounded-capture.v1.1048576";
        private readonly MemoryStream inner = new MemoryStream();
        private readonly long maximumBytes;

        public BoundedCaptureStream(long maximumBytes)
        {
            if (maximumBytes < 1) throw new ArgumentOutOfRangeException(nameof(maximumBytes));
            this.maximumBytes = maximumBytes;
        }

        public bool Overflowed { get; private set; }
        public string GetUtf8Text() => new UTF8Encoding(false, false).GetString(inner.ToArray());
        public override bool CanRead => false;
        public override bool CanSeek => false;
        public override bool CanWrite => true;
        public override long Length => inner.Length;
        public override long Position { get => inner.Position; set => throw new NotSupportedException(); }
        public override void Flush() { }
        public override Task FlushAsync(CancellationToken cancellationToken) => Task.CompletedTask;
        public override int Read(byte[] buffer, int offset, int count) => throw new NotSupportedException();
        public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
        public override void SetLength(long value) => throw new NotSupportedException();

        public override void Write(byte[] buffer, int offset, int count)
        {
            if (count <= 0) return;
            long remaining = maximumBytes - inner.Length;
            int retained = remaining <= 0 ? 0 : (int)Math.Min(remaining, count);
            if (retained > 0) inner.Write(buffer, offset, retained);
            if (retained != count) Overflowed = true;
        }

        public override Task WriteAsync(
            byte[] buffer,
            int offset,
            int count,
            CancellationToken cancellationToken)
        {
            cancellationToken.ThrowIfCancellationRequested();
            Write(buffer, offset, count);
            return Task.CompletedTask;
        }
    }
}
'@
} elseif ([Tessara.Validation.BoundedCaptureStream]::ImplementationId -cne
    $script:BoundedCaptureImplementationId) {
    throw "The loaded bounded-output runtime does not match this lifecycle module. Start a fresh PowerShell process."
}

$script:MaximumCapturedStreamBytes = 1048576

function Get-TessaraValidationLifecycleIdentity {
    [CmdletBinding()]
    param()

    $tokens = $null
    $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile(
        $script:LifecycleModulePath, [ref]$tokens, [ref]$errors
    )
    if (@($errors).Count -ne 0) {
        throw "The lifecycle module cannot derive capability identities from invalid source."
    }
    $functions = @($ast.FindAll({
        param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst]
    }, $true))
    $groups = [ordered]@{
        common = @($functions | Where-Object {
            $_.Name -cnotmatch 'Docker|Compose' -and
            $_.Name -cnotmatch 'PortLease|TcpReadiness' -and
            $_.Name -cne 'Invoke-TessaraValidationLifecycleLane'
        })
        local_process = @($functions | Where-Object {
            $_.Name -match 'PortLease|TcpReadiness|LifecycleProcess'
        })
        docker_compose = @($functions | Where-Object { $_.Name -match 'Docker|Compose' })
        orchestration = @($functions | Where-Object {
            $_.Name -ceq 'Invoke-TessaraValidationLifecycleLane'
        })
    }
    $capabilities = [ordered]@{}
    foreach ($group in $groups.GetEnumerator()) {
        $text = @($group.Value | Sort-Object Name | ForEach-Object {
            "$($_.Name)`n$($_.Extent.Text)"
        }) -join "`n"
        $capabilities[[string]$group.Key] = Get-TessaraLifecycleSha256Text -Text ($text + "`n")
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = $script:LifecycleContract
        release_version = $script:LifecycleRelease
        module_sha256 = $script:LifecycleModuleSha256AtImport
        capability_fingerprints = [pscustomobject]$capabilities
    }
}

function Get-TessaraLifecycleSha256Text {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
}

function Split-TessaraLifecycleNativePath {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Path)
    $separators = [Collections.Generic.List[char]]::new()
    $separators.Add([IO.Path]::DirectorySeparatorChar)
    if ([IO.Path]::AltDirectorySeparatorChar -ne [IO.Path]::DirectorySeparatorChar) {
        $separators.Add([IO.Path]::AltDirectorySeparatorChar)
    }
    @($Path.Split($separators.ToArray(), [StringSplitOptions]::RemoveEmptyEntries))
}

function Protect-TessaraLifecycleText {
    param(
        [AllowNull()][string]$Text,
        [AllowEmptyCollection()][string[]]$SensitiveValues = @()
    )
    $protected = [string]$Text
    $distinct = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($value in @($SensitiveValues)) {
        $candidate = [string]$value
        if (-not [string]::IsNullOrEmpty($candidate)) {
            $null = $distinct.Add($candidate)
        }
    }
    if ($distinct.Count -ne 0) {
        # One alternation pass prevents a replacement marker from being
        # interpreted as input by a later secret and expanding recursively.
        $pattern = @($distinct | Sort-Object Length -Descending | ForEach-Object {
                [Text.RegularExpressions.Regex]::Escape([string]$_)
            }) -join '|'
        $protected = [Text.RegularExpressions.Regex]::Replace(
            $protected,
            $pattern,
            "[REDACTED:TESSARA_SOURCE_ENV]",
            [Text.RegularExpressions.RegexOptions]::CultureInvariant
        )
    }
    $encoding = [Text.UTF8Encoding]::new($false)
    $bytes = $encoding.GetBytes($protected)
    if ($bytes.Length -gt $script:MaximumCapturedStreamBytes) {
        $suffix = $encoding.GetBytes("`n[TESSARA_REDACTED_OUTPUT_TRUNCATED]`n")
        $end = $script:MaximumCapturedStreamBytes - $suffix.Length
        while ($end -gt 0 -and ($bytes[$end] -band 0xC0) -eq 0x80) {
            $end--
        }
        $protected = $encoding.GetString($bytes, 0, $end) +
            $encoding.GetString($suffix)
    }
    $protected
}

function Resolve-TessaraLifecycleRepositoryPath {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )
    $root = [IO.Path]::GetFullPath($RepositoryRoot)
    $resolved = [IO.Path]::GetFullPath((Join-Path $root $Path))
    $relative = [IO.Path]::GetRelativePath($root, $resolved)
    $prefix = "..$([IO.Path]::DirectorySeparatorChar)"
    if ($relative -eq "." -or $relative -eq ".." -or
        [IO.Path]::IsPathRooted($relative) -or
        $relative.StartsWith($prefix, [StringComparison]::Ordinal)) {
        throw "$Label is outside the repository root: $Path"
    }
    $resolved
}

function Assert-TessaraLifecycleNoReparsePath {
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
        throw "$Label is outside the repository root."
    }
    $current = $root
    foreach ($segment in @(Split-TessaraLifecycleNativePath -Path $relative)) {
        $current = Join-Path $current $segment
        if (-not (Test-Path -LiteralPath $current)) { continue }
        $entry = Get-Item -Force -LiteralPath $current
        if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "$Label crosses a reparse point: $current"
        }
    }
    $resolved
}

function Assert-TessaraLifecycleGitBinding {
    param([Parameter(Mandatory)]$PlatformIdentity)
    if ($null -eq $PlatformIdentity.runtime_identity -or
        [string]::IsNullOrWhiteSpace(
            [string]$PlatformIdentity.runtime_identity.git_executable_path
        ) -or
        [string]::IsNullOrWhiteSpace(
            [string]$PlatformIdentity.runtime_identity.git_content_sha256
        )) {
        throw "Validation lifecycle did not receive an authenticated Git runtime binding."
    }
    $path = [IO.Path]::GetFullPath(
        [string]$PlatformIdentity.runtime_identity.git_executable_path
    )
    $entry = Get-Item -Force -LiteralPath $path
    if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
        ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
        (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant() -cne
            [string]$PlatformIdentity.runtime_identity.git_content_sha256) {
        throw "The validation-platform Git executable no longer matches its planned identity."
    }
    $path
}

function Invoke-TessaraLifecycleIsolatedGit {
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

function ConvertFrom-TessaraLifecycleGitNulRecords {
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

function Get-TessaraLifecycleLiveDependencyFingerprints {
    param(
        [Parameter(Mandatory)][object[]]$DomainDefinitions,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)]$PlatformIdentity
    )
    $gitPath = Assert-TessaraLifecycleGitBinding -PlatformIdentity $PlatformIdentity
    $cachedInventory = Invoke-TessaraLifecycleIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $RepositoryRoot `
        -Arguments @("-C", $RepositoryRoot, "ls-files", "--stage", "-z", "--cached")
    $untrackedInventory = Invoke-TessaraLifecycleIsolatedGit -GitPath $gitPath `
        -RepositoryRoot $RepositoryRoot `
        -Arguments @("-C", $RepositoryRoot, "ls-files", "-z", "--others", "--exclude-per-directory=.gitignore")
    if ([int]$cachedInventory.exit_code -ne 0 -or
        [int]$untrackedInventory.exit_code -ne 0) {
        throw "Unable to re-enumerate the candidate working tree after lane execution."
    }
    $cachedModes = [Collections.Generic.Dictionary[string, string]]::new(
        [StringComparer]::Ordinal
    )
    foreach ($record in @(ConvertFrom-TessaraLifecycleGitNulRecords `
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
    foreach ($path in @(ConvertFrom-TessaraLifecycleGitNulRecords `
            -Text ([string]$untrackedInventory.stdout) `
            -Label "Candidate untracked working-tree inventory")) {
        if (-not $allPaths.Add([string]$path)) {
            throw "Candidate working-tree inventory contains a duplicate cached/untracked path: $path"
        }
    }
    $entries = [Collections.Generic.List[object]]::new()
    $domainPatterns = @($DomainDefinitions | ForEach-Object {
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
        $resolved = Resolve-TessaraLifecycleRepositoryPath -RepositoryRoot $RepositoryRoot `
            -Path $path -Label "Candidate dependency input"
        if (Test-Path -LiteralPath $resolved -PathType Leaf) {
            $null = Assert-TessaraLifecycleNoReparsePath -RepositoryRoot $RepositoryRoot `
                -Path $resolved -Label "Candidate dependency input"
            $entries.Add([pscustomobject][ordered]@{
                path = $path
                sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $resolved).Hash.ToLowerInvariant()
            })
        } elseif (Test-Path -LiteralPath $resolved) {
            throw "Candidate dependency input is not one ordinary file: $path"
        } elseif ($cachedModes.ContainsKey($path)) {
            $entries.Add([pscustomobject][ordered]@{
                path = $path
                sha256 = "absent"
            })
        } else {
            throw "Untracked candidate dependency input disappeared during inventory: $path"
        }
    }
    @($DomainDefinitions | ForEach-Object {
        $domain = $_
        $matches = @($entries | Where-Object {
            $candidatePath = [string]$_.path
            @($domain.tracked_inputs | Where-Object {
                $candidatePath -clike ([string]$_).Replace('\', '/').Trim()
            }).Count -gt 0
        } | Sort-Object path)
        $fingerprintValue = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value (
            [pscustomobject][ordered]@{
                entries = @($matches | ForEach-Object {
                    [pscustomobject][ordered]@{
                        path = [string]$_.path
                        sha256 = [string]$_.sha256
                    }
                })
            }
        )
        $fingerprintJson = $fingerprintValue | ConvertTo-Json -Depth 20 -Compress
        [pscustomobject][ordered]@{
            domain = [string]$domain.name
            sha256 = Get-TessaraLifecycleSha256Text -Text ($fingerprintJson + "`n")
            file_count = $matches.Count
        }
    } | Sort-Object domain)
}

function Get-TessaraLifecycleCommandInputObservations {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$InputPaths,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    @($InputPaths | ForEach-Object {
        $declaredPath = [string]$_
        $resolved = Resolve-TessaraLifecycleRepositoryPath `
            -RepositoryRoot $RepositoryRoot -Path $declaredPath `
            -Label "Declared command input"
        if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
            throw "Declared command input is missing: $declaredPath"
        }
        $null = Assert-TessaraLifecycleNoReparsePath -RepositoryRoot $RepositoryRoot `
            -Path $resolved -Label "Declared command input"
        [pscustomobject][ordered]@{
            path = $declaredPath
            expanded_path_sha256 = Get-TessaraLifecycleSha256Text -Text $resolved
            content_sha256 = (
                Get-FileHash -Algorithm SHA256 -LiteralPath $resolved
            ).Hash.ToLowerInvariant()
        }
    } | Sort-Object path)
}

function Assert-TessaraLifecycleCommandInputObservationsUnchanged {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Expected,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    $actual = @(Get-TessaraLifecycleCommandInputObservations `
        -InputPaths @($Expected | ForEach-Object { [string]$_.path }) `
        -RepositoryRoot $RepositoryRoot)
    $expectedCanonical = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value @($Expected)
    $actualCanonical = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value @($actual)
    $expectedFingerprint = Get-TessaraLifecycleSha256Text -Text (
        ($expectedCanonical | ConvertTo-Json -Depth 50 -Compress) + "`n"
    )
    $actualFingerprint = Get-TessaraLifecycleSha256Text -Text (
        ($actualCanonical | ConvertTo-Json -Depth 50 -Compress) + "`n"
    )
    if ($actualFingerprint -cne $expectedFingerprint) {
        throw "A declared command input changed immediately before invocation."
    }
}

function Assert-TessaraLifecycleLaneInputsUnchanged {
    param(
        [Parameter(Mandatory)]$LaneIdentity,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)]$PlatformIdentity
    )
    # The validated adapter and governing contract are immutable snapshots for
    # this attempt. Their lane-scoped canonical fingerprints remain on the
    # result; a later plan with changed bytes cannot reuse it. Requiring raw
    # whole-file equality here would unnecessarily revoke an independent lane
    # for JSON formatting or another lane's declaration change.
    foreach ($input in @($LaneIdentity.acceptance_inputs) +
        @($LaneIdentity.fixture_inputs) + @($LaneIdentity.execution_inputs)) {
        $resolved = Resolve-TessaraLifecycleRepositoryPath -RepositoryRoot $RepositoryRoot `
            -Path ([string]$input.path) -Label "Lane-owned execution input"
        if (-not (Test-Path -LiteralPath $resolved -PathType Leaf) -or
            (Get-FileHash -Algorithm SHA256 -LiteralPath $resolved).Hash.ToLowerInvariant() -cne
                [string]$input.sha256) {
            throw "Lane-owned input '$($input.path)' changed during execution."
        }
        $null = Assert-TessaraLifecycleNoReparsePath -RepositoryRoot $RepositoryRoot `
            -Path $resolved -Label "Lane-owned execution input"
    }
    $commandInputObservations = @($LaneIdentity.tool_observations | ForEach-Object {
        @($_.input_paths)
    } | Sort-Object -Property path -CaseSensitive -Unique)
    Assert-TessaraLifecycleCommandInputObservationsUnchanged `
        -Expected $commandInputObservations -RepositoryRoot $RepositoryRoot
    $currentDomains = Get-TessaraLifecycleLiveDependencyFingerprints `
        -DomainDefinitions @($LaneIdentity.dependency_domain_definitions) `
        -RepositoryRoot $RepositoryRoot -PlatformIdentity $PlatformIdentity
    $plannedDomains = @($LaneIdentity.dependency_fingerprints | Sort-Object domain)
    if ($currentDomains.Count -ne $plannedDomains.Count) {
        throw "Lane dependency-domain inventory changed during execution."
    }
    for ($index = 0; $index -lt $currentDomains.Count; $index++) {
        if ([string]$currentDomains[$index].domain -cne [string]$plannedDomains[$index].domain -or
            [string]$currentDomains[$index].sha256 -cne [string]$plannedDomains[$index].sha256 -or
            [int]$currentDomains[$index].file_count -ne [int]$plannedDomains[$index].file_count) {
            throw "Lane dependency domain '$($plannedDomains[$index].domain)' changed during execution."
        }
    }
}

function Get-TessaraLifecycleBaseProcessEnvironment {
    $allowedNames = @(
        "PATHEXT", "SystemRoot", "WINDIR", "ComSpec",
        "PROGRAMDATA", "ProgramFiles", "ProgramFiles(x86)", "ProgramW6432",
        "NUMBER_OF_PROCESSORS", "PROCESSOR_ARCHITECTURE", "OS",
        "LANG", "LC_ALL"
    )
    $source = [Environment]::GetEnvironmentVariables("Process")
    $environment = [ordered]@{}
    foreach ($name in $allowedNames) {
        if ($source.Contains($name)) {
            $environment[$name] = [string][Environment]::GetEnvironmentVariable($name, "Process")
        }
    }
    $environment
}

function New-TessaraLifecycleProcessEnvironmentPlan {
    param(
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)]$Topology,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$TopologyPorts
    )
    $environment = Get-TessaraLifecycleBaseProcessEnvironment
    $sourceEnvironment = [Environment]::GetEnvironmentVariables("Process")
    $bindings = [Collections.Generic.List[object]]::new()
    $errors = [Collections.Generic.List[string]]::new()
    $redactions = [Collections.Generic.List[string]]::new()
    foreach ($entry in $environment.GetEnumerator()) {
        $baseValue = [string]$entry.Value
        $bindings.Add([pscustomobject][ordered]@{
            name = [string]$entry.Key
            origin = "platform-base"
            source = [string]$entry.Key
            present = $true
            value_sha256 = Get-TessaraLifecycleSha256Text -Text $baseValue
        })
    }
    foreach ($entry in @($Lane.environment)) {
        $name = [string]$entry.name
        if ($entry.PSObject.Properties.Name -contains "value") {
            $value = Expand-TessaraLifecycleValue -Value ([string]$entry.value) -Context $Context
            $environment[$name] = $value
            $bindings.Add([pscustomobject][ordered]@{
                name = $name
                origin = "literal"
                source = $null
                present = $true
                value_sha256 = Get-TessaraLifecycleSha256Text -Text $value
            })
        } else {
            $sourceName = [string]$entry.source
            $present = $sourceEnvironment.Contains($sourceName)
            $optional = $entry.PSObject.Properties.Name -contains "optional" -and [bool]$entry.optional
            $value = if ($present) {
                [string][Environment]::GetEnvironmentVariable($sourceName, "Process")
            } else { $null }
            if (-not $present -and -not $optional) {
                $errors.Add("Required source environment variable '$sourceName' is absent for '$name'.")
            }
            if ($present -and [string]::IsNullOrWhiteSpace($value)) {
                $errors.Add("Source environment variable '$sourceName' is empty for '$name'.")
            }
            if ($present) { $environment[$name] = $value }
            if ($present -and -not [string]::IsNullOrEmpty($value)) {
                $redactions.Add($value)
            }
            $bindings.Add([pscustomobject][ordered]@{
                name = $name
                origin = "process-source"
                source = $sourceName
                present = $present
                value_sha256 = if ($present) { Get-TessaraLifecycleSha256Text -Text $value } else { $null }
            })
        }
    }
    foreach ($tempName in @("TEMP", "TMP", "TMPDIR")) {
        $tempValue = [string]$Context.temp_root
        $environment[$tempName] = $tempValue
        $bindings.Add([pscustomobject][ordered]@{
            name = $tempName
            origin = "platform"
            source = $null
            present = $true
            value_sha256 = Get-TessaraLifecycleSha256Text -Text $tempValue
        })
    }
    foreach ($name in @($Lane.blocked_environment | ForEach-Object { [string]$_ })) {
        $value = "__TESSARA_VALIDATION_UNDECLARED_ENVIRONMENT__"
        $environment[$name] = $value
        $bindings.Add([pscustomobject][ordered]@{
            name = $name
            origin = "platform-deny"
            source = $null
            present = $true
            value_sha256 = Get-TessaraLifecycleSha256Text -Text $value
        })
    }
    foreach ($port in $TopologyPorts) {
        $name = [string]$port.environment
        $value = [string]$Context.ports[[string]$port.name]
        $environment[$name] = $value
        $bindings.Add([pscustomobject][ordered]@{
            name = $name
            origin = "port-lease"
            source = [string]$port.name
            present = $true
            value_sha256 = Get-TessaraLifecycleSha256Text -Text $value
        })
    }
    if ([string]$Topology.provider -ceq "docker-compose") {
        $environment["COMPOSE_PROJECT_NAME"] = [string]$Context.compose_project
        $environment["TESSARA_VALIDATION_TOPOLOGY_LEASE"] = [string]$Context.topology_lease
        $environment["TESSARA_VALIDATION_TOPOLOGY_ATTEMPT"] = [string]$Context.topology_attempt_id
        $environment["DOCKER_CONFIG"] = [string]$Context.docker_config
        foreach ($platformBinding in @(
                [pscustomobject]@{ name = "COMPOSE_PROJECT_NAME"; value = [string]$Context.compose_project },
                [pscustomobject]@{ name = "TESSARA_VALIDATION_TOPOLOGY_LEASE"; value = [string]$Context.topology_lease },
                [pscustomobject]@{ name = "TESSARA_VALIDATION_TOPOLOGY_ATTEMPT"; value = [string]$Context.topology_attempt_id },
                [pscustomobject]@{ name = "DOCKER_CONFIG"; value = [string]$Context.docker_config }
            )) {
            $bindings.Add([pscustomobject][ordered]@{
                name = [string]$platformBinding.name
                origin = "platform"
                source = $null
                present = $true
                value_sha256 = Get-TessaraLifecycleSha256Text -Text ([string]$platformBinding.value)
            })
        }
    }
    $fingerprintText = @($bindings | Sort-Object name | ForEach-Object {
        "$($_.name)`n$($_.origin)`n$($_.source)`n$($_.present)`n$($_.value_sha256)"
    }) -join "`n"
    [pscustomobject][ordered]@{
        environment = $environment
        bindings = @($bindings)
        fingerprint = Get-TessaraLifecycleSha256Text -Text ($fingerprintText + "`n")
        redactions = @($redactions | Sort-Object -CaseSensitive -Unique)
        errors = @($errors)
    }
}

function Get-TessaraLifecycleEnvironmentObservationFingerprint {
    param([Parameter(Mandatory)]$EnvironmentPlan)
    $observations = @($EnvironmentPlan.bindings | Where-Object {
        [string]$_.origin -in @("platform-base", "process-source")
    } | ForEach-Object {
        if ([string]$_.origin -ceq "platform-base") {
            [pscustomobject][ordered]@{
                name = [string]$_.name
                kind = "platform-base"
                present = $true
                value_sha256 = [string]$_.value_sha256
            }
        } else {
            [pscustomobject][ordered]@{
                name = [string]$_.name
                kind = "declared-source"
                source = [string]$_.source
                present = [bool]$_.present
                value_sha256 = if ([bool]$_.present) {
                    [string]$_.value_sha256
                } else { $null }
            }
        }
    } | Sort-Object kind, name)
    $canonical = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value (
        [pscustomobject][ordered]@{ observations = $observations }
    )
    Get-TessaraLifecycleSha256Text -Text (
        ($canonical | ConvertTo-Json -Depth 50 -Compress) + "`n"
    )
}

function New-TessaraLifecyclePortLease {
    param([Parameter(Mandatory)][object[]]$Ports)
    $listeners = [Collections.Generic.List[object]]::new()
    $values = [ordered]@{}
    try {
        foreach ($port in $Ports) {
            $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
            $listener.Start()
            $listeners.Add($listener)
            $values[[string]$port.name] = ([Net.IPEndPoint]$listener.LocalEndpoint).Port
        }
        [pscustomobject][ordered]@{ listeners = $listeners; values = $values }
    } catch {
        foreach ($listener in $listeners) { $listener.Stop() }
        throw
    }
}

function Close-TessaraLifecyclePortLease {
    param([AllowNull()]$Lease)
    if ($null -ne $Lease) {
        foreach ($listener in @($Lease.listeners)) { $listener.Stop() }
    }
}

function Expand-TessaraLifecycleValue {
    param(
        [Parameter(Mandatory)][AllowEmptyString()][string]$Value,
        [Parameter(Mandatory)]$Context
    )
    $expanded = $Value
    $tokens = [ordered]@{
        '${repository_root}' = [string]$Context.repository_root
        '${attempt_root}' = [string]$Context.attempt_root
        '${candidate_fingerprint}' = [string]$Context.candidate_fingerprint
        '${acceptance_fingerprint}' = [string]$Context.acceptance_fingerprint
        '${adapter_fingerprint}' = [string]$Context.adapter_fingerprint
        '${platform_fingerprint}' = [string]$Context.platform_fingerprint
        '${compose_project}' = [string]$Context.compose_project
    }
    foreach ($entry in $tokens.GetEnumerator()) {
        $expanded = $expanded.Replace([string]$entry.Key, [string]$entry.Value)
    }
    foreach ($name in @($Context.ports.Keys)) {
        $expanded = $expanded.Replace('${port:' + $name + '}', [string]$Context.ports[$name])
    }
    if ($expanded -match '\$\{[^}]+\}') {
        throw "Value contains an unknown validation-platform placeholder: $Value"
    }
    $expanded
}

function Start-TessaraLifecycleProcess {
    param(
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [AllowNull()][ref]$ProcessStarted,
        [string]$ProcessKind = "generic",
        [string]$CertificationFault
    )
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Program
    $start.WorkingDirectory = $WorkingDirectory
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.RedirectStandardInput = $true
    $start.Environment.Clear()
    foreach ($entry in $Environment.GetEnumerator()) {
        $start.Environment[[string]$entry.Key] = [string]$entry.Value
    }
    foreach ($argument in $Arguments) { [void]$start.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $start
    $started = $false
    try {
        if (-not $process.Start()) {
            throw "Could not start validation child program '$Program'."
        }
        $started = $true
        if ($null -ne $ProcessStarted) { $ProcessStarted.Value = $true }
        if ($ProcessKind -ceq "local-topology" -and
            $CertificationFault -ceq "local-topology-after-start") {
            throw "Injected failure after local topology process start."
        }
        $process.StandardInput.Close()
        $process
    } catch {
        $startFailure = $_.Exception
        $cleanupFailures = [Collections.Generic.List[Exception]]::new()
        if ($started) {
            try { Stop-TessaraLifecycleProcess -Process $process } catch {
                $cleanupFailures.Add($_.Exception)
            }
        }
        try { $process.Dispose() } catch { $cleanupFailures.Add($_.Exception) }
        if ($cleanupFailures.Count -ne 0) {
            $failures = [Collections.Generic.List[Exception]]::new()
            $failures.Add($startFailure)
            foreach ($cleanupFailure in $cleanupFailures) {
                $failures.Add($cleanupFailure)
            }
            throw [AggregateException]::new(
                "Validation child acquisition failed and cleanup was incomplete.",
                [Exception[]]$failures.ToArray()
            )
        }
        throw $startFailure
    }
}

function Get-TessaraLifecycleCommandBinding {
    param(
        [Parameter(Mandatory)][string]$Id,
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [AllowEmptyCollection()][string[]]$InputPaths = @()
    )
    $resolvedProgram = $null
    $programCandidate = if ([IO.Path]::IsPathRooted($Program)) {
        [IO.Path]::GetFullPath($Program)
    } else {
        [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $Program))
    }
    if (Test-Path -LiteralPath $programCandidate -PathType Leaf) {
        $resolvedProgram = $programCandidate
    } else {
        $command = Get-Command -Name $Program -CommandType Application,ExternalScript `
            -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -ne $command -and -not [string]::IsNullOrWhiteSpace([string]$command.Source)) {
            $resolvedProgram = [IO.Path]::GetFullPath([string]$command.Source)
        }
    }
    if ($null -eq $resolvedProgram -or
        -not (Test-Path -LiteralPath $resolvedProgram -PathType Leaf)) {
        throw "Validation command '$Id' program '$Program' could not be resolved before execution."
    }
    [pscustomobject][ordered]@{
        id = $Id
        resolved_program = $resolvedProgram
        program_path_sha256 = Get-TessaraLifecycleSha256Text -Text $resolvedProgram
        program_sha256 = (
            Get-FileHash -Algorithm SHA256 -LiteralPath $resolvedProgram
        ).Hash.ToLowerInvariant()
        arguments_sha256 = Get-TessaraLifecycleSha256Text `
            -Text ((@($Arguments | ForEach-Object { [string]$_ }) -join "`0") + "`n")
        input_paths = @(Get-TessaraLifecycleCommandInputObservations `
            -InputPaths $InputPaths -RepositoryRoot $RepositoryRoot)
    }
}

function Assert-TessaraLifecycleCommandBindingUnchanged {
    param(
        [Parameter(Mandatory)]$ExpectedBinding,
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments,
        [Parameter(Mandatory)][string]$RepositoryRoot
    )
    $expectedInputPaths = @($ExpectedBinding.input_paths | ForEach-Object { [string]$_.path })
    $actualBinding = Get-TessaraLifecycleCommandBinding `
        -Id ([string]$ExpectedBinding.id) -Program $Program `
        -Arguments $Arguments -RepositoryRoot $RepositoryRoot `
        -InputPaths $expectedInputPaths
    $expectedComparable = [pscustomobject][ordered]@{
        id = [string]$ExpectedBinding.id
        resolved_program = [string]$ExpectedBinding.resolved_program
        program_path_sha256 = [string]$ExpectedBinding.program_path_sha256
        program_sha256 = [string]$ExpectedBinding.program_sha256
        arguments_sha256 = [string]$ExpectedBinding.arguments_sha256
        input_paths = @($ExpectedBinding.input_paths)
    }
    $expectedCanonical = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $expectedComparable
    $actualCanonical = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $actualBinding
    $expectedFingerprint = Get-TessaraLifecycleSha256Text -Text (
        ($expectedCanonical | ConvertTo-Json -Depth 50 -Compress) + "`n"
    )
    $actualFingerprint = Get-TessaraLifecycleSha256Text -Text (
        ($actualCanonical | ConvertTo-Json -Depth 50 -Compress) + "`n"
    )
    if ($actualFingerprint -cne $expectedFingerprint) {
        throw "Validation command '$([string]$ExpectedBinding.id)' identity changed immediately before invocation."
    }
    $actualBinding
}

function Stop-TessaraLifecycleProcess {
    param([AllowNull()][Diagnostics.Process]$Process)
    if ($null -ne $Process -and -not $Process.HasExited) {
        $Process.Kill($true)
        if (-not $Process.WaitForExit(10000) -or -not $Process.HasExited) {
            throw "Validation child process tree did not terminate within 10 seconds."
        }
    }
}

function Invoke-TessaraLifecycleProgram {
    param(
        [Parameter(Mandatory)][string]$Program,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$Arguments,
        [Parameter(Mandatory)][string]$WorkingDirectory,
        [Parameter(Mandatory)][int]$TimeoutSeconds,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [ref]$ProcessStarted,
        [string]$CancellationPath,
        [string]$ProcessKind = "generic",
        [string]$CertificationFault
    )
    $process = $null
    $stdoutCapture = $null
    $stderrCapture = $null
    try {
        $startInvocation = @{
            Program = $Program
            Arguments = $Arguments
            WorkingDirectory = $WorkingDirectory
            Environment = $Environment
            ProcessKind = $ProcessKind
            CertificationFault = $CertificationFault
        }
        if ($null -ne $ProcessStarted) {
            $startInvocation.ProcessStarted = $ProcessStarted
        }
        $process = Start-TessaraLifecycleProcess @startInvocation
        if ($ProcessKind -ceq "action" -and
            $CertificationFault -ceq "assertion-after-process-acquisition") {
            throw "Injected failure after assertion process start."
        }
        if ($ProcessKind -ceq "action" -and
            $CertificationFault -ceq "assertion-cancellation-after-process-acquisition") {
            throw [OperationCanceledException]::new(
                "Injected cancellation after assertion process start."
            )
        }
        $stdoutCapture = [Tessara.Validation.BoundedCaptureStream]::new(
            $script:MaximumCapturedStreamBytes
        )
        $stderrCapture = [Tessara.Validation.BoundedCaptureStream]::new(
            $script:MaximumCapturedStreamBytes
        )
        $stdoutTask = $process.StandardOutput.BaseStream.CopyToAsync($stdoutCapture)
        $stderrTask = $process.StandardError.BaseStream.CopyToAsync($stderrCapture)
        $deadline = [DateTimeOffset]::UtcNow.AddSeconds($TimeoutSeconds)
        $terminal = "completed"
        while (-not $process.HasExited) {
            if (-not [string]::IsNullOrWhiteSpace($CancellationPath) -and
                (Test-Path -LiteralPath $CancellationPath -PathType Leaf)) {
                $terminal = "interrupted"
                Stop-TessaraLifecycleProcess -Process $process
                break
            }
            if ([DateTimeOffset]::UtcNow -ge $deadline) {
                $terminal = "timed_out"
                Stop-TessaraLifecycleProcess -Process $process
                break
            }
            Start-Sleep -Milliseconds 100
        }
        $process.WaitForExit()
        [void]$stdoutTask.GetAwaiter().GetResult()
        [void]$stderrTask.GetAwaiter().GetResult()
        if ($terminal -ceq "completed" -and
            ($stdoutCapture.Overflowed -or $stderrCapture.Overflowed)) {
            $terminal = "output_overflow"
        }
        [pscustomobject][ordered]@{
            terminal = $terminal
            exit_code = if ($terminal -ceq "completed") { $process.ExitCode } else { $null }
            stdout = $stdoutCapture.GetUtf8Text()
            stderr = $stderrCapture.GetUtf8Text()
            output_limit_bytes = $script:MaximumCapturedStreamBytes
            stdout_overflow = $stdoutCapture.Overflowed
            stderr_overflow = $stderrCapture.Overflowed
        }
    } finally {
        try {
            Stop-TessaraLifecycleProcess -Process $process
        } finally {
            if ($null -ne $process) { $process.Dispose() }
            if ($null -ne $stdoutCapture) { $stdoutCapture.Dispose() }
            if ($null -ne $stderrCapture) { $stderrCapture.Dispose() }
        }
    }
}

function Invoke-TessaraLifecycleDeclaredAction {
    param(
        [Parameter(Mandatory)]$Action,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [Parameter(Mandatory)]$ExpectedBinding,
        [AllowEmptyCollection()][string[]]$RedactedValues = @(),
        [ref]$ProcessStarted,
        [string]$CancellationPath,
        [string]$CertificationFault
    )
    $started = [DateTimeOffset]::UtcNow
    $program = Expand-TessaraLifecycleValue -Value ([string]$Action.program) -Context $Context
    $arguments = @($Action.arguments | ForEach-Object {
        Expand-TessaraLifecycleValue -Value ([string]$_) -Context $Context
    })
    $actualBinding = Assert-TessaraLifecycleCommandBindingUnchanged -ExpectedBinding $ExpectedBinding `
        -Program $program -Arguments $arguments -RepositoryRoot $RepositoryRoot
    $programInvocation = @{
        Program = [string]$actualBinding.resolved_program
        Arguments = $arguments
        WorkingDirectory = $RepositoryRoot
        TimeoutSeconds = [int]$Action.timeout_seconds
        Environment = $Environment
        CancellationPath = $CancellationPath
        ProcessKind = "action"
        CertificationFault = $CertificationFault
    }
    if ($null -ne $ProcessStarted) {
        $programInvocation.ProcessStarted = $ProcessStarted
    }
    $result = Invoke-TessaraLifecycleProgram @programInvocation
    $stdout = Write-TessaraLifecycleNewText `
        -EvidenceRoot $EvidenceRoot `
        -Path (Join-Path $AttemptRoot "actions/$($Action.id).stdout.log") `
        -Text (Protect-TessaraLifecycleText -Text ([string]$result.stdout) `
            -SensitiveValues $RedactedValues)
    $stderr = Write-TessaraLifecycleNewText `
        -EvidenceRoot $EvidenceRoot `
        -Path (Join-Path $AttemptRoot "actions/$($Action.id).stderr.log") `
        -Text (Protect-TessaraLifecycleText -Text ([string]$result.stderr) `
            -SensitiveValues $RedactedValues)
    [pscustomobject][ordered]@{
        id = [string]$Action.id
        stage = [string]$Action.stage
        implementation_target = if ($Action.PSObject.Properties.Name -contains
            "implementation_target") { [string]$Action.implementation_target } else { $null }
        proof_classes = if ($Action.PSObject.Properties.Name -contains
            "proof_classes") { @($Action.proof_classes | ForEach-Object { [string]$_ }) } else { @() }
        terminal = [string]$result.terminal
        exit_code = $result.exit_code
        output_limit_bytes = $result.output_limit_bytes
        stdout_overflow = [bool]$result.stdout_overflow
        stderr_overflow = [bool]$result.stderr_overflow
        started_at = $started.ToString("O")
        completed_at = [DateTimeOffset]::UtcNow.ToString("O")
        stdout = $stdout
        stderr = $stderr
    }
}

function Assert-TessaraLifecycleActionPassed {
    param([Parameter(Mandatory)]$ActionResult)
    if ([string]$ActionResult.terminal -ceq "interrupted") {
        throw [OperationCanceledException]::new("Action '$($ActionResult.id)' was interrupted.")
    }
    if ([string]$ActionResult.terminal -ceq "timed_out") {
        throw [TimeoutException]::new("Action '$($ActionResult.id)' timed out.")
    }
    if ([string]$ActionResult.terminal -ceq "output_overflow") {
        throw "Action '$($ActionResult.id)' exceeded the bounded output limit."
    }
    if ([int]$ActionResult.exit_code -ne 0) {
        throw "Action '$($ActionResult.id)' exited $($ActionResult.exit_code)."
    }
}

function Wait-TessaraLifecycleTcpReadiness {
    param(
        [Parameter(Mandatory)][int]$Port,
        [Parameter(Mandatory)][int]$TimeoutSeconds,
        [Parameter(Mandatory)][string]$Capability,
        [string]$CancellationPath,
        [Diagnostics.Process]$Process
    )
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (-not [string]::IsNullOrWhiteSpace($CancellationPath) -and
            (Test-Path -LiteralPath $CancellationPath -PathType Leaf)) {
            throw [OperationCanceledException]::new("Validation lane was interrupted during readiness.")
        }
        if ($null -ne $Process -and $Process.HasExited) {
            throw "Topology process exited before readiness with code $($Process.ExitCode)."
        }
        $client = [Net.Sockets.TcpClient]::new()
        try {
            $task = $client.ConnectAsync([Net.IPAddress]::Loopback, $Port)
            if ($task.Wait(250) -and $client.Connected) {
                $client.ReceiveTimeout = 500
                $client.SendTimeout = 500
                $nonce = [Convert]::ToHexString(
                    [Security.Cryptography.RandomNumberGenerator]::GetBytes(16)
                ).ToLowerInvariant()
                $writer = [IO.StreamWriter]::new(
                    $client.GetStream(), [Text.UTF8Encoding]::new($false), 1024, $true
                )
                $reader = [IO.StreamReader]::new(
                    $client.GetStream(), [Text.UTF8Encoding]::new($false, $true),
                    $false, 1024, $true
                )
                try {
                    $writer.NewLine = "`n"
                    $writer.AutoFlush = $true
                    $writer.WriteLine($nonce)
                    $response = $reader.ReadLine()
                    $payload = [Text.UTF8Encoding]::new($false).GetBytes(
                        "$Capability`n$nonce"
                    )
                    $expected = [Convert]::ToHexString(
                        [Security.Cryptography.SHA256]::HashData($payload)
                    ).ToLowerInvariant()
                    if ([string]$response -ceq $expected) { return }
                } finally {
                    $reader.Dispose()
                    $writer.Dispose()
                }
            }
        } catch {
        } finally {
            $client.Dispose()
        }
        Start-Sleep -Milliseconds 100
    } while ([DateTimeOffset]::UtcNow -lt $deadline)
    throw "Topology did not become ready on loopback port $Port within $TimeoutSeconds seconds."
}

function Invoke-TessaraLifecycleDocker {
    param(
        [Parameter(Mandatory)][string[]]$DockerCommand,
        [Parameter(Mandatory)][string[]]$Arguments,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [Parameter(Mandatory)]$ExpectedBinding,
        [string]$CancellationPath,
        [switch]$AllowFailure
    )
    $program = $DockerCommand[0]
    [string[]]$prefix = @()
    if ($DockerCommand.Count -gt 1) {
        $prefix = @($DockerCommand[1..($DockerCommand.Count - 1)])
    }
    $actualBinding = Assert-TessaraLifecycleCommandBindingUnchanged -ExpectedBinding $ExpectedBinding `
        -Program $program -Arguments $prefix -RepositoryRoot $RepositoryRoot
    $result = Invoke-TessaraLifecycleProgram -Program ([string]$actualBinding.resolved_program) `
        -Arguments @($prefix + $Arguments) `
        -WorkingDirectory $RepositoryRoot -TimeoutSeconds 120 -Environment $Environment `
        -CancellationPath $CancellationPath
    if ($result.terminal -cne "completed") {
        throw "Docker Compose command did not complete: $($Arguments -join ' ')"
    }
    if (-not $AllowFailure -and $result.exit_code -ne 0) {
        throw "Docker Compose command exited $($result.exit_code): $($Arguments -join ' ')`n$($result.stderr)"
    }
    $result
}

function Get-TessaraLifecycleComposeArguments {
    param([Parameter(Mandatory)]$Topology, [Parameter(Mandatory)]$Context)
    $arguments = @("compose", "--env-file", [string]$Context.compose_env_file, `
        "-f", (Resolve-TessaraLifecycleRepositoryPath `
        -RepositoryRoot $Context.repository_root -Path ([string]$Topology.compose_file) `
        -Label "Compose file"), "-p", [string]$Context.compose_project)
    foreach ($profile in @($Topology.profiles)) { $arguments += @("--profile", [string]$profile) }
    $arguments
}

function Assert-TessaraLifecycleComposeHealthy {
    param(
        [Parameter(Mandatory)]$Topology,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [Parameter(Mandatory)][string[]]$DockerCommand,
        [string]$CancellationPath
    )
    $base = Get-TessaraLifecycleComposeArguments -Topology $Topology -Context $Context
    $deadline = [DateTimeOffset]::UtcNow.AddSeconds(
        [Math]::Max(1, [int]$Topology.readiness.timeout_seconds)
    )
    $states = @()
    while ($true) {
        Assert-TessaraLifecycleNotCancelled -CancellationPath $CancellationPath `
            -Boundary "Docker Compose health wait"
        $result = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
            -Arguments @($base + @("ps", "--all", "--format", "json")) `
            -RepositoryRoot $Context.repository_root -Environment $Environment `
            -ExpectedBinding $Context.command_bindings["docker"] `
            -CancellationPath $CancellationPath
        $text = [string]$result.stdout
        try {
            $parsed = $text | ConvertFrom-Json -Depth 30
            $states = @($parsed)
        } catch {
            $states = @($text -split "`r?`n" | Where-Object { $_ } | ForEach-Object {
                $_ | ConvertFrom-Json -Depth 30
            })
        }
        foreach ($state in $states) {
            $stateProperties = @($state.PSObject.Properties.Name)
            foreach ($requiredProperty in @("ID", "Service", "State", "Health", "Image")) {
                if ($stateProperties -cnotcontains $requiredProperty) {
                    throw "Compose topology status omitted required '$requiredProperty' provenance."
                }
            }
        }
        $ready = $true
        foreach ($serviceName in @($Topology.required_services)) {
            $matches = @($states | Where-Object { [string]($_.Service) -ceq [string]$serviceName })
            $match = if ($matches.Count -eq 1) { $matches[0] } else { $null }
            $matchImage = if ($null -eq $match) { $null } else {
                [string]($match.PSObject.Properties["Image"].Value)
            }
            if ($null -ne $match -and
                $matchImage -notin @($Topology.provider_identity.allowed_images)) {
                throw "Compose topology '$serviceName' runs an image outside its authenticated provider contract."
            }
            if ($matches.Count -ne 1 -or [string]($matches[0].State) -cne "running" -or
                [string]($matches[0].Health) -cne "healthy" -or
                [string]($matches[0].ID) -cnotmatch '^[A-Za-z0-9._:-]{8,128}$') {
                $ready = $false
            }
        }
        if ($ready) { break }
        if ([DateTimeOffset]::UtcNow -ge $deadline) {
            throw "Compose topology did not become healthy before its readiness deadline."
        }
        Start-Sleep -Milliseconds 200
    }
    $runtimeValue = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value @(
        $states | Sort-Object Service | ForEach-Object {
            $runtimeState = $_
            [pscustomobject][ordered]@{
                id = [string]($runtimeState.ID)
                service = [string]($runtimeState.Service)
                image = [string]($runtimeState.PSObject.Properties["Image"].Value)
                state = [string]($runtimeState.State)
                health = [string]($runtimeState.Health)
            }
        }
    )
    Get-TessaraLifecycleSha256Text -Text (
        (($runtimeValue | ConvertTo-Json -Depth 20 -Compress) + "`n")
    )
}

function Get-TessaraLifecycleDockerProviderIdentity {
    param(
        [Parameter(Mandatory)]$Topology,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [Parameter(Mandatory)][string[]]$DockerCommand,
        [string]$CancellationPath
    )
    $providerInfo = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
        -Arguments @("info", "--format", "json") `
        -RepositoryRoot $Context.repository_root -Environment $Environment `
        -ExpectedBinding $Context.command_bindings["docker"] `
        -CancellationPath $CancellationPath
    $contextResult = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
        -Arguments @("context", "show") -RepositoryRoot $Context.repository_root `
        -Environment $Environment -ExpectedBinding $Context.command_bindings["docker"] `
        -CancellationPath $CancellationPath
    try {
        $providerDocument = [string]$providerInfo.stdout | ConvertFrom-Json -Depth 30
    } catch {
        throw "Docker provider identity is not valid JSON."
    }
    $identity = [pscustomobject][ordered]@{
        context = ([string]$contextResult.stdout).Trim()
        daemon_id = [string]$providerDocument.ID
        server_version = [string]$providerDocument.ServerVersion
    }
    if ([string]$identity.context -cne [string]$Topology.provider_identity.context -or
        [string]$identity.daemon_id -cne [string]$Topology.provider_identity.daemon_id -or
        [string]$identity.server_version -cne [string]$Topology.provider_identity.server_version) {
        throw "Docker provider identity does not match the adapter's authenticated daemon contract."
    }
    [pscustomobject][ordered]@{
        document = $identity
        fingerprint = Get-TessaraLifecycleSha256Text -Text (
            (((ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $identity) |
                ConvertTo-Json -Depth 20 -Compress) + "`n")
        )
    }
}

function Get-TessaraLifecycleComposeConfiguration {
    param(
        [Parameter(Mandatory)]$Topology,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [Parameter(Mandatory)][string[]]$DockerCommand,
        [string]$CancellationPath
    )
    $base = Get-TessaraLifecycleComposeArguments -Topology $Topology -Context $Context
    $result = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
        -Arguments @($base + @("config", "--format", "json")) `
        -RepositoryRoot $Context.repository_root -Environment $Environment `
        -ExpectedBinding $Context.command_bindings["docker"] `
        -CancellationPath $CancellationPath
    try {
        $configuration = [string]$result.stdout | ConvertFrom-Json -Depth 50
    } catch {
        throw "Docker Compose configuration is not valid JSON."
    }
    if ([string]$configuration.name -cne [string]$Context.compose_project) {
        throw "Docker Compose configuration substituted project '$($configuration.name)'."
    }
    $serviceNames = @($configuration.services.PSObject.Properties.Name)
    foreach ($requiredService in @($Topology.required_services)) {
        if ([string]$requiredService -notin $serviceNames) {
            throw "Docker Compose configuration omits required service '$requiredService'."
        }
    }
    foreach ($serviceProperty in @($configuration.services.PSObject.Properties)) {
        $service = $serviceProperty.Value
        foreach ($forbidden in @(
                "build", "volumes", "env_file", "privileged", "cap_add", "devices",
                "network_mode", "pid", "ipc", "uts", "secrets", "configs", "develop"
            )) {
            if ($service.PSObject.Properties.Name -contains $forbidden) {
                throw "Docker Compose service '$($serviceProperty.Name)' uses forbidden host or daemon capability '$forbidden'."
            }
        }
        if ([string]$service.image -notin @($Topology.provider_identity.allowed_images)) {
            throw "Docker Compose service '$($serviceProperty.Name)' does not use an allowed immutable image digest."
        }
        if (-not [bool]$service.read_only -or -not [bool]$service.init -or
            [string]::IsNullOrWhiteSpace([string]$service.user) -or
            [string]$service.user -in @("0", "0:0", "root")) {
            throw "Docker Compose service '$($serviceProperty.Name)' is not read-only, init-owned, and non-root."
        }
        if ($null -eq $service.environment -or
            [string]$service.environment.TESSARA_VALIDATION_READINESS_CAPABILITY -cne
                [string]$Environment["TESSARA_VALIDATION_READINESS_CAPABILITY"]) {
            throw "Docker Compose service '$($serviceProperty.Name)' does not bind authenticated readiness."
        }
    }
    foreach ($globalSection in @("secrets", "configs")) {
        if ($configuration.PSObject.Properties.Name -contains $globalSection -and
            $null -ne $configuration.$globalSection) {
            throw "Docker Compose configuration declares forbidden global '$globalSection'."
        }
    }
    $expectedLabels = [ordered]@{
        "tessara.validation.lease" = [string]$Context.topology_lease
        "tessara.validation.attempt" = [string]$Context.topology_attempt_id
    }
    $labelOwners = [Collections.Generic.List[object]]::new()
    foreach ($serviceProperty in @($configuration.services.PSObject.Properties)) {
        $labelOwners.Add([pscustomobject]@{
            label = "service '$($serviceProperty.Name)'"
            value = $serviceProperty.Value
        })
    }
    foreach ($sectionName in @("volumes", "networks")) {
        if ($configuration.PSObject.Properties.Name -contains $sectionName -and
            $null -ne $configuration.$sectionName) {
            foreach ($property in @($configuration.$sectionName.PSObject.Properties)) {
                $expectedOwnedName = "$([string]$Context.compose_project)_$([string]$property.Name)"
                if (($property.Value.PSObject.Properties.Name -contains "external" -and
                        [bool]$property.Value.external) -or
                    ($property.Value.PSObject.Properties.Name -contains "name" -and
                        -not [string]::IsNullOrWhiteSpace([string]$property.Value.name) -and
                        [string]$property.Value.name -cne $expectedOwnedName)) {
                    throw "Docker Compose $sectionName '$($property.Name)' uses an external or custom global name."
                }
                $labelOwners.Add([pscustomobject]@{
                    label = "$sectionName '$($property.Name)'"
                    value = $property.Value
                })
            }
        }
    }
    foreach ($owner in $labelOwners) {
        if (-not ($owner.value.PSObject.Properties.Name -contains "labels") -or
            $null -eq $owner.value.labels) {
            throw "Docker Compose $($owner.label) lacks validation ownership labels."
        }
        foreach ($expectedLabel in $expectedLabels.GetEnumerator()) {
            $actualProperty = @($owner.value.labels.PSObject.Properties | Where-Object {
                [string]$_.Name -ceq [string]$expectedLabel.Key
            })
            if ($actualProperty.Count -ne 1 -or
                [string]$actualProperty[0].Value -cne [string]$expectedLabel.Value) {
                throw "Docker Compose $($owner.label) has an invalid '$($expectedLabel.Key)' ownership label."
            }
        }
    }
    $normalizedValue = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $configuration
    $normalized = ($normalizedValue | ConvertTo-Json -Depth 50 -Compress)
    [pscustomobject][ordered]@{
        document = $configuration
        normalized_sha256 = Get-TessaraLifecycleSha256Text -Text ($normalized + "`n")
    }
}

function ConvertTo-TessaraLifecycleCanonicalJsonValue {
    param([AllowNull()]$Value)
    if ($null -eq $Value -or $Value -is [string] -or
        $Value -is [bool] -or $Value -is [ValueType]) {
        return $Value
    }
    if ($Value -is [Collections.IDictionary]) {
        $result = [ordered]@{}
        foreach ($key in @($Value.Keys | ForEach-Object { [string]$_ } | Sort-Object)) {
            $result[$key] = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $Value[$key]
        }
        return $result
    }
    if ($Value -is [Collections.IEnumerable]) {
        return @($Value | ForEach-Object {
            ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $_
        })
    }
    $objectResult = [ordered]@{}
    foreach ($property in @($Value.PSObject.Properties | Sort-Object Name)) {
        $objectResult[$property.Name] = ConvertTo-TessaraLifecycleCanonicalJsonValue `
            -Value $property.Value
    }
    $objectResult
}

function Remove-TessaraLifecycleComposeTopology {
    param(
        [Parameter(Mandatory)]$Topology,
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][Collections.IDictionary]$Environment,
        [Parameter(Mandatory)][string[]]$DockerCommand
    )
    $null = Get-TessaraLifecycleComposeConfiguration -Topology $Topology `
        -Context $Context -Environment $Environment -DockerCommand $DockerCommand
    $base = Get-TessaraLifecycleComposeArguments -Topology $Topology -Context $Context
    $ownedInventory = [ordered]@{}
    foreach ($resource in @("container", "volume", "network")) {
        $listArguments = @($resource, "ls")
        if ($resource -ceq "container") { $listArguments += "--all" }
        $projectResult = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
            -Arguments @($listArguments + @(
                "--filter", "label=com.docker.compose.project=$($Context.compose_project)",
                "--format", "{{.ID}}"
            )) -RepositoryRoot $Context.repository_root -Environment $Environment `
            -ExpectedBinding $Context.command_bindings["docker"] -AllowFailure
        $ownedResult = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
            -Arguments @($listArguments + @(
                "--filter", "label=com.docker.compose.project=$($Context.compose_project)",
                "--filter", "label=tessara.validation.lease=$($Context.topology_lease)",
                "--filter", "label=tessara.validation.attempt=$($Context.topology_attempt_id)",
                "--format", "{{.ID}}"
            )) -RepositoryRoot $Context.repository_root -Environment $Environment `
            -ExpectedBinding $Context.command_bindings["docker"] -AllowFailure
        if ($projectResult.exit_code -ne 0 -or $ownedResult.exit_code -ne 0) {
            throw "Compose ownership inventory failed for '$($Context.compose_project)'."
        }
        $projectIds = @(([string]$projectResult.stdout -split "`r?`n") | Where-Object { $_ } | Sort-Object)
        $ownedIds = @(([string]$ownedResult.stdout -split "`r?`n") | Where-Object { $_ } | Sort-Object)
        if (($projectIds -join "`n") -cne ($ownedIds -join "`n")) {
            throw "Compose project '$($Context.compose_project)' contains resources outside its ownership capability."
        }
        $ownedInventory[$resource] = @($ownedIds)
    }
    $down = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
        -Arguments @($base + @("down", "--volumes", "--remove-orphans")) `
        -RepositoryRoot $Context.repository_root -Environment $Environment `
        -ExpectedBinding $Context.command_bindings["docker"] -AllowFailure
    $remaining = [ordered]@{}
    $inspectionFailed = $false
    foreach ($resource in @("container", "volume", "network")) {
        $listArguments = @($resource, "ls")
        if ($resource -ceq "container") { $listArguments += "--all" }
        $result = Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
            -Arguments @($listArguments + @("--filter", "label=com.docker.compose.project=$($Context.compose_project)", "--format", "{{.ID}}")) `
            -RepositoryRoot $Context.repository_root -Environment $Environment `
            -ExpectedBinding $Context.command_bindings["docker"] -AllowFailure
        if ($result.exit_code -ne 0) { $inspectionFailed = $true }
        $remaining[$resource] = @(([string]$result.stdout -split "`r?`n") | Where-Object { $_ })
    }
    if ($down.exit_code -ne 0 -or $inspectionFailed -or
        @($remaining.Values | ForEach-Object { @($_) }).Count -ne 0) {
        throw "Compose teardown failed or retained resources for '$($Context.compose_project)'."
    }
    [pscustomobject][ordered]@{
        state = "passed"
        mode = "destroyed"
        owned_inventory = $ownedInventory
        remaining = $remaining
    }
}

function Initialize-TessaraLifecycleOwnedEvidenceDirectory {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Path
    )
    $root = [IO.Path]::GetFullPath($EvidenceRoot)
    $target = [IO.Path]::GetFullPath($Path)
    $relative = [IO.Path]::GetRelativePath($root, $target)
    if ($relative -eq "." -or $relative -eq ".." -or
        [IO.Path]::IsPathRooted($relative) -or
        $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
        $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
        throw "Evidence directory is outside its owned evidence root: $target"
    }
    $rootEntry = Get-Item -Force -LiteralPath $root
    if (-not $rootEntry.PSIsContainer -or
        ($rootEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Evidence root is not one ordinary owned directory: $root"
    }
    $current = $root
    foreach ($segment in @(Split-TessaraLifecycleNativePath -Path $relative)) {
        $current = Join-Path $current $segment
        if ([IO.File]::Exists($current)) {
            throw "Evidence directory path collides with a file: $current"
        }
        if (-not [IO.Directory]::Exists($current)) {
            [IO.Directory]::CreateDirectory($current) | Out-Null
        }
        $entry = Get-Item -Force -LiteralPath $current
        if (-not $entry.PSIsContainer -or
            ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Evidence directory crosses a reparse point: $current"
        }
    }
    $target
}

function Write-TessaraLifecycleNewText {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Text,
        [AllowNull()][scriptblock]$AfterDataCreate,
        [switch]$RetainDataOnFailure
    )
    $Path = [IO.Path]::GetFullPath($Path)
    $parent = Initialize-TessaraLifecycleOwnedEvidenceDirectory `
        -EvidenceRoot $EvidenceRoot -Path (Split-Path -Parent $Path)
    $sidecarPath = "$Path.sha256"
    $dataCreated = $false
    $sidecarCreated = $false
    try {
        $stream = [IO.File]::Open(
            $Path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::Read
        )
        $dataCreated = $true
        try {
            $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
            $stream.Write($bytes, 0, $bytes.Length)
            $stream.Flush($true)
        } finally { $stream.Dispose() }
        if ($null -ne $AfterDataCreate) {
            & $AfterDataCreate $Path | Out-Null
        }
        $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
        $null = Initialize-TessaraLifecycleOwnedEvidenceDirectory `
            -EvidenceRoot $EvidenceRoot -Path $parent
        $sidecar = [IO.File]::Open(
            $sidecarPath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::Read
        )
        $sidecarCreated = $true
        try {
            $sidecarBytes = [Text.UTF8Encoding]::new($false).GetBytes(
                "$hash  $([IO.Path]::GetFileName($Path))`n"
            )
            $sidecar.Write($sidecarBytes, 0, $sidecarBytes.Length)
            $sidecar.Flush($true)
        } finally { $sidecar.Dispose() }
        return [pscustomobject][ordered]@{ path = $Path; sha256 = $hash }
    } catch {
        $publicationFailure = $_
        if ($dataCreated) {
            $publicationFailure.Exception.Data["TessaraEvidenceDataCreated"] = $true
        }
        $rollbackFailures = [Collections.Generic.List[Exception]]::new()
        foreach ($ownedPath in @(
                if ($sidecarCreated) { $sidecarPath }
                if ($dataCreated -and -not $RetainDataOnFailure) { $Path }
            )) {
            try {
                $null = Initialize-TessaraLifecycleOwnedEvidenceDirectory `
                    -EvidenceRoot $EvidenceRoot -Path (Split-Path -Parent ([string]$ownedPath))
                [IO.File]::Delete([string]$ownedPath)
                if (Test-Path -LiteralPath ([string]$ownedPath)) {
                    throw "Evidence publication rollback left '$ownedPath' behind."
                }
            } catch {
                $rollbackFailures.Add($_.Exception)
            }
        }
        if ($rollbackFailures.Count -ne 0) {
            $failures = [Collections.Generic.List[Exception]]::new()
            $failures.Add($publicationFailure.Exception)
            foreach ($rollbackFailure in $rollbackFailures) {
                $failures.Add($rollbackFailure)
            }
            $aggregate = [AggregateException]::new(
                "Evidence publication failed and its partial pair could not be removed.",
                [Exception[]]$failures.ToArray()
            )
            if ($dataCreated) {
                $aggregate.Data["TessaraEvidenceDataCreated"] = $true
            }
            throw $aggregate
        }
        throw
    }
}

function Write-TessaraLifecycleNewJson {
    param(
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)]$Document,
        [AllowNull()][scriptblock]$AfterDataCreate,
        [switch]$RetainDataOnFailure
    )
    Write-TessaraLifecycleNewText -EvidenceRoot $EvidenceRoot -Path $Path `
        -Text (($Document | ConvertTo-Json -Depth 100 -Compress) + "`n") `
        -AfterDataCreate $AfterDataCreate -RetainDataOnFailure:$RetainDataOnFailure
}

function Write-TessaraLifecycleExecutionRevocation {
    param(
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Reason,
        [string]$CertificationFault
    )
    if ($CertificationFault -ceq "execution-revocation-publication") {
        throw "Injected execution-revocation publication failure."
    }
    Write-TessaraLifecycleNewJson `
        -EvidenceRoot $EvidenceRoot `
        -Path (Join-Path $AttemptRoot "execution-revoked.json") `
        -Document ([pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.execution-revocation"
            state = "revoked"
            reason = $Reason
        })
}

function Read-TessaraLifecycleHashedJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$TrustedRoot
    )
    $resolved = [IO.Path]::GetFullPath($Path)
    $sidecarPath = "$resolved.sha256"
    $root = [IO.Path]::GetFullPath($TrustedRoot)
    foreach ($candidatePath in @($resolved, $sidecarPath)) {
        $relative = [IO.Path]::GetRelativePath($root, $candidatePath)
        if ($relative -eq "." -or $relative -eq ".." -or
            [IO.Path]::IsPathRooted($relative) -or
            $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
            $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
            throw "$Label path is outside its trusted evidence root."
        }
    }
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf) -or
        -not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
        throw "$Label pair is missing: $resolved"
    }
    foreach ($candidate in @(
            [pscustomobject]@{ path = $resolved; maximum = 16MB },
            [pscustomobject]@{ path = $sidecarPath; maximum = 1024 }
        )) {
        $relative = [IO.Path]::GetRelativePath($root, [string]$candidate.path)
        if ($relative -eq "." -or $relative -eq ".." -or
            [IO.Path]::IsPathRooted($relative) -or
            $relative.StartsWith("..$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal) -or
            $relative.StartsWith("..$([IO.Path]::AltDirectorySeparatorChar)", [StringComparison]::Ordinal)) {
            throw "$Label path is outside its trusted evidence root."
        }
        $current = $root
        $rootEntry = Get-Item -Force -LiteralPath $root
        if (($rootEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "$Label trusted evidence root is a reparse path."
        }
        foreach ($segment in @(Split-TessaraLifecycleNativePath -Path $relative)) {
            $current = Join-Path $current $segment
            $entry = Get-Item -Force -LiteralPath $current
            if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "$Label path crosses a reparse point."
            }
        }
        $leaf = Get-Item -Force -LiteralPath ([string]$candidate.path)
        if ($leaf.PSIsContainer -or [int64]$leaf.Length -gt [int64]$candidate.maximum -or
            ($leaf.PSObject.Properties.Name -contains 'UnixMode' -and
                -not ([string]$leaf.UnixMode).StartsWith('-', [StringComparison]::Ordinal))) {
            throw "$Label pair contains a non-ordinary or oversized file."
        }
    }
    $bytes = [IO.File]::ReadAllBytes($resolved)
    $actual = [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($bytes)
    ).ToLowerInvariant()
    $sidecar = [IO.File]::ReadAllText($sidecarPath, [Text.UTF8Encoding]::new($false))
    $expectedSidecar = "$actual  $([IO.Path]::GetFileName($resolved))`n"
    if ($sidecar -cne $expectedSidecar) {
        throw "$Label digest sidecar is invalid."
    }
    $text = [Text.UTF8Encoding]::new($false, $true).GetString($bytes)
    try {
        $document = $text | ConvertFrom-Json -Depth 100
    } catch {
        throw "$Label is not valid UTF-8 JSON."
    }
    [pscustomobject][ordered]@{
        path = $resolved
        sha256 = $actual
        document = $document
    }
}

function Ensure-TessaraLifecycleExecutionIntegrityCommit {
    param(
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)]$CheckpointReference,
        [Parameter(Mandatory)][string]$ExpectedLaneId,
        [Parameter(Mandatory)][string]$ExpectedAttemptId,
        [Parameter(Mandatory)][string]$CandidateFingerprint,
        [Parameter(Mandatory)]$LaneIdentity,
        [Parameter(Mandatory)][string]$ExecutionPlatformFingerprint,
        [Parameter(Mandatory)][string]$EvidenceRootFingerprint,
        [switch]$AllowCreate
    )
    $checkpointPath = [IO.Path]::GetFullPath([string]$CheckpointReference.path)
    $checkpoint = Read-TessaraLifecycleHashedJson -Path $checkpointPath `
        -Label "Execution checkpoint" -TrustedRoot $EvidenceRoot
    $result = $checkpoint.document.result
    if ([string]$checkpoint.sha256 -cne [string]$CheckpointReference.sha256 -or
        $null -eq $result -or
        $result.PSObject.Properties.Name -notcontains "source_integrity_failure" -or
        $null -ne $result.source_integrity_failure -or
        [string]$result.lane_id -cne $ExpectedLaneId -or
        [string]$result.attempt_id -cne $ExpectedAttemptId -or
        [string]$result.candidate_fingerprint -cne $CandidateFingerprint -or
        [string]$result.lane_compatibility_fingerprint -cne
            [string]$LaneIdentity.compatibility_fingerprint -or
        [string]$result.platform_execution_fingerprint -cne
            $ExecutionPlatformFingerprint -or
        [string]$result.evidence_root_fingerprint -cne $EvidenceRootFingerprint -or
        [bool]$result.authoritative) {
        throw "Execution checkpoint cannot receive a positive integrity commit because its raw result identity or source-integrity state is invalid."
    }

    $document = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.execution-integrity-commit"
        state = "verified"
        lane_id = $ExpectedLaneId
        attempt_id = $ExpectedAttemptId
        candidate_fingerprint = $CandidateFingerprint
        candidate_source_identity = $LaneIdentity.candidate_source_identity
        lane_compatibility_fingerprint = [string]$LaneIdentity.compatibility_fingerprint
        compatibility_plan_fingerprint = [string]$LaneIdentity.compatibility_plan_fingerprint
        platform_execution_fingerprint = $ExecutionPlatformFingerprint
        evidence_root_fingerprint = $EvidenceRootFingerprint
        execution_checkpoint = [pscustomobject][ordered]@{
            path = [string]$checkpoint.path
            sha256 = [string]$checkpoint.sha256
        }
    }
    $path = Join-Path $AttemptRoot "execution-integrity-verified.json"
    $sidecarPath = "$path.sha256"
    if (-not (Test-Path -LiteralPath $path -PathType Leaf) -and
        -not (Test-Path -LiteralPath $sidecarPath -PathType Leaf)) {
        if (-not $AllowCreate) {
            throw "Execution integrity commit is missing; finalization-only recovery cannot create it."
        }
        $null = Write-TessaraLifecycleNewJson -EvidenceRoot $EvidenceRoot `
            -Path $path -Document $document
    }
    $snapshot = Read-TessaraLifecycleHashedJson -Path $path `
        -Label "Execution integrity commit" -TrustedRoot $EvidenceRoot
    $actual = $snapshot.document
    $expectedProperties = @(
        "schema_version", "contract", "state", "lane_id", "attempt_id",
        "candidate_fingerprint", "candidate_source_identity",
        "lane_compatibility_fingerprint", "compatibility_plan_fingerprint",
        "platform_execution_fingerprint", "evidence_root_fingerprint",
        "execution_checkpoint"
    ) | Sort-Object -CaseSensitive
    $actualProperties = @($actual.PSObject.Properties.Name | Sort-Object -CaseSensitive)
    if (($actualProperties -join "`n") -cne ($expectedProperties -join "`n") -or
        [string]$actual.schema_version -cne "1" -or
        [string]$actual.contract -cne "tessara.validation.execution-integrity-commit" -or
        [string]$actual.state -cne "verified" -or
        [string]$actual.lane_id -cne $ExpectedLaneId -or
        [string]$actual.attempt_id -cne $ExpectedAttemptId -or
        [string]$actual.candidate_fingerprint -cne $CandidateFingerprint -or
        ((ConvertTo-TessaraLifecycleCanonicalJsonValue $actual.candidate_source_identity) |
            ConvertTo-Json -Depth 50 -Compress) -cne
            ((ConvertTo-TessaraLifecycleCanonicalJsonValue $LaneIdentity.candidate_source_identity) |
                ConvertTo-Json -Depth 50 -Compress) -or
        [string]$actual.lane_compatibility_fingerprint -cne
            [string]$LaneIdentity.compatibility_fingerprint -or
        [string]$actual.compatibility_plan_fingerprint -cne
            [string]$LaneIdentity.compatibility_plan_fingerprint -or
        [string]$actual.platform_execution_fingerprint -cne
            $ExecutionPlatformFingerprint -or
        [string]$actual.evidence_root_fingerprint -cne $EvidenceRootFingerprint -or
        -not (Test-TessaraLifecycleReferenceBinding `
            -Reference $actual.execution_checkpoint `
            -ExpectedPath ([string]$checkpoint.path) `
            -ExpectedSha256 ([string]$checkpoint.sha256))) {
        throw "Execution integrity commit does not bind the exact current checkpoint and execution identity."
    }
    $snapshot
}

function Test-TessaraLifecycleReferenceBinding {
    param(
        [AllowNull()]$Reference,
        [Parameter(Mandatory)][string]$ExpectedPath,
        [Parameter(Mandatory)][string]$ExpectedSha256
    )
    if ($null -eq $Reference -or
        $Reference.PSObject.Properties.Name -notcontains "path" -or
        $Reference.PSObject.Properties.Name -notcontains "sha256") {
        return $false
    }
    [string]$Reference.path -ceq [IO.Path]::GetFullPath($ExpectedPath) -and
        [string]$Reference.sha256 -ceq $ExpectedSha256
}

function Read-TessaraLifecycleFinalizationCommit {
    param(
        [Parameter(Mandatory)][string]$AttemptRoot,
        [Parameter(Mandatory)]$ResultSnapshot,
        [Parameter(Mandatory)][string]$ExpectedLaneId,
        [Parameter(Mandatory)][string]$ExpectedAttemptId,
        [Parameter(Mandatory)][string]$CurrentFinalizationFingerprint,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$Label
    )
    if ($CurrentFinalizationFingerprint -cnotmatch '^[0-9a-f]{64}$') {
        throw "Current platform finalization fingerprint is not canonical SHA-256."
    }
    $root = [IO.Path]::GetFullPath($AttemptRoot)
    $expectedResultPath = [IO.Path]::GetFullPath([string]$ResultSnapshot.path)
    $expectedIndexPath = [IO.Path]::GetFullPath((Join-Path $root "attempt-index.json"))
    $expectedCheckpointPath = [IO.Path]::GetFullPath(
        (Join-Path $root "execution-complete.json")
    )
    $expectedAttestationPath = [IO.Path]::GetFullPath(
        (Join-Path $root `
            "finalization-attestation.$CurrentFinalizationFingerprint.json")
    )
    $indexSnapshot = Read-TessaraLifecycleHashedJson -Path $expectedIndexPath `
        -Label "$Label attempt index" -TrustedRoot $EvidenceRoot
    $checkpointSnapshot = Read-TessaraLifecycleHashedJson -Path $expectedCheckpointPath `
        -Label "$Label execution checkpoint" -TrustedRoot $EvidenceRoot
    $attestationSnapshot = Read-TessaraLifecycleHashedJson -Path $expectedAttestationPath `
        -Label "$Label finalization attestation" -TrustedRoot $EvidenceRoot

    $index = $indexSnapshot.document
    $checkpoint = $checkpointSnapshot.document
    $attestation = $attestationSnapshot.document
    $indexProperties = @(
        "schema_version", "contract", "state", "attempt_id",
        "execution_checkpoint", "result", "files"
    )
    $checkpointProperties = @(
        "schema_version", "contract", "state", "result", "artifacts"
    )
    $attestationProperties = @(
        "schema_version", "contract", "state", "attempt_id",
        "finalization_platform_fingerprint", "execution_checkpoint",
        "result", "attempt_index"
    )
    $missingIndexProperty = $null -eq $index -or @($indexProperties | Where-Object {
            $index.PSObject.Properties.Name -notcontains $_
        }).Count -ne 0
    $missingCheckpointProperty = $null -eq $checkpoint -or
        @($checkpointProperties | Where-Object {
                $checkpoint.PSObject.Properties.Name -notcontains $_
            }).Count -ne 0
    $missingAttestationProperty = $null -eq $attestation -or
        @($attestationProperties | Where-Object {
                $attestation.PSObject.Properties.Name -notcontains $_
            }).Count -ne 0
    if ($missingIndexProperty -or $missingCheckpointProperty -or
        $missingAttestationProperty) {
        throw "$Label finalization commit authentication failed."
    }
    $indexFilesCanonical = ConvertTo-TessaraLifecycleCanonicalJsonValue `
        -Value @($index.files)
    $checkpointArtifactsCanonical = ConvertTo-TessaraLifecycleCanonicalJsonValue `
        -Value @($checkpoint.artifacts)
    $indexFilesText = $indexFilesCanonical | ConvertTo-Json -Depth 100 -Compress
    $checkpointArtifactsText = $checkpointArtifactsCanonical |
        ConvertTo-Json -Depth 100 -Compress
    $checkpointResult = $checkpoint.result
    if ($null -eq $checkpointResult -or
        $checkpointResult.PSObject.Properties.Name -notcontains "lane_id" -or
        $checkpointResult.PSObject.Properties.Name -notcontains "attempt_id" -or
        $checkpointResult.PSObject.Properties.Name -notcontains "authoritative" -or
        [string]$index.schema_version -cne "1" -or
        [string]$index.contract -cne "tessara.validation.attempt-index" -or
        [string]$index.state -cne "passed" -or
        [string]$index.attempt_id -cne $ExpectedAttemptId -or
        [string]$indexFilesText -cne [string]$checkpointArtifactsText -or
        -not (Test-TessaraLifecycleReferenceBinding -Reference $index.result `
            -ExpectedPath $expectedResultPath -ExpectedSha256 ([string]$ResultSnapshot.sha256)) -or
        -not (Test-TessaraLifecycleReferenceBinding -Reference $index.execution_checkpoint `
            -ExpectedPath $expectedCheckpointPath `
            -ExpectedSha256 ([string]$checkpointSnapshot.sha256)) -or
        [string]$checkpoint.schema_version -cne "1" -or
        [string]$checkpoint.contract -cne "tessara.validation.execution-checkpoint" -or
        [string]$checkpoint.state -cne "execution_complete" -or
        [string]$checkpointResult.lane_id -cne $ExpectedLaneId -or
        [string]$checkpointResult.attempt_id -cne $ExpectedAttemptId -or
        [bool]$checkpointResult.authoritative -or
        [string]$attestation.schema_version -cne "1" -or
        [string]$attestation.contract -cne "tessara.validation.finalization-attestation" -or
        [string]$attestation.state -cne "complete" -or
        [string]$attestation.attempt_id -cne $ExpectedAttemptId -or
        [string]$attestation.finalization_platform_fingerprint -cne
            $CurrentFinalizationFingerprint -or
        -not (Test-TessaraLifecycleReferenceBinding `
            -Reference $attestation.execution_checkpoint `
            -ExpectedPath $expectedCheckpointPath `
            -ExpectedSha256 ([string]$checkpointSnapshot.sha256)) -or
        -not (Test-TessaraLifecycleReferenceBinding -Reference $attestation.result `
            -ExpectedPath $expectedResultPath -ExpectedSha256 ([string]$ResultSnapshot.sha256)) -or
        -not (Test-TessaraLifecycleReferenceBinding -Reference $attestation.attempt_index `
            -ExpectedPath $expectedIndexPath -ExpectedSha256 ([string]$indexSnapshot.sha256))) {
        throw "$Label finalization commit authentication failed."
    }
    [pscustomobject][ordered]@{
        index = $indexSnapshot
        checkpoint = $checkpointSnapshot
        attestation = $attestationSnapshot
    }
}

function Stop-TessaraLifecycleBlocked {
    param([Parameter(Mandatory)][string]$Message)
    $exception = [InvalidOperationException]::new($Message)
    $exception.Data["TessaraLaneState"] = "blocked"
    throw $exception
}

function Assert-TessaraLifecycleNotCancelled {
    param(
        [string]$CancellationPath,
        [Parameter(Mandatory)][string]$Boundary
    )
    if (-not [string]::IsNullOrWhiteSpace($CancellationPath) -and
        (Test-Path -LiteralPath $CancellationPath -PathType Leaf)) {
        throw [OperationCanceledException]::new(
            "Validation lane was interrupted at $Boundary."
        )
    }
}

function Get-TessaraLifecycleDeadlineAction {
    param(
        [Parameter(Mandatory)]$Action,
        [Parameter(Mandatory)][DateTimeOffset]$Deadline
    )
    $remaining = [int][Math]::Floor(($Deadline - [DateTimeOffset]::UtcNow).TotalSeconds)
    if ($remaining -lt 1) {
        throw [TimeoutException]::new("Validation lane exceeded its aggregate deadline.")
    }
    $copy = [ordered]@{}
    foreach ($property in $Action.PSObject.Properties) {
        $copy[$property.Name] = $property.Value
    }
    $copy.timeout_seconds = [Math]::Min([int]$Action.timeout_seconds, $remaining)
    [pscustomobject]$copy
}

function Invoke-TessaraValidationLifecycleLane {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Adapter,
        [Parameter(Mandatory)]$Lane,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$EvidenceRoot,
        [Parameter(Mandatory)][string]$CandidateFingerprint,
        [Parameter(Mandatory)][string]$AggregateAcceptanceFingerprint,
        [Parameter(Mandatory)][string]$AggregateAdapterFingerprint,
        [Parameter(Mandatory)]$LaneIdentity,
        [Parameter(Mandatory)][object[]]$AllLaneIdentities,
        [Parameter(Mandatory)]$PlatformIdentity,
        [string]$TopologyReceiptPath,
        [AllowEmptyCollection()][string[]]$PrerequisiteResultPaths = @(),
        [string]$FinalizeCheckpointPath,
        [string]$CancellationPath,
        [string[]]$DockerCommand = @("docker"),
        [AllowEmptyCollection()][string[]]$DockerCommandInputPaths = @(),
        [string]$CertificationFault
    )
    $repository = [IO.Path]::GetFullPath($RepositoryRoot)
    $evidence = [IO.Path]::GetFullPath($EvidenceRoot)
    Assert-TessaraLifecycleLaneInputsUnchanged -LaneIdentity $LaneIdentity `
        -RepositoryRoot $repository -PlatformIdentity $PlatformIdentity
    $null = Assert-TessaraLifecycleNoReparsePath -RepositoryRoot $repository `
        -Path $evidence -Label "Validation evidence root"
    $evidenceRootFingerprint = Get-TessaraLifecycleSha256Text -Text $evidence
    [IO.Directory]::CreateDirectory($evidence) | Out-Null
    $null = Assert-TessaraLifecycleNoReparsePath -RepositoryRoot $repository `
        -Path $evidence -Label "Validation evidence root"
    $AcceptanceFingerprint = [string]$LaneIdentity.acceptance_fingerprint
    $AdapterFingerprint = [string]$LaneIdentity.adapter_fingerprint
    $PlatformExecutionFingerprint = [string]$LaneIdentity.platform_execution_fingerprint
    $laneDeadline = [DateTimeOffset]::UtcNow.AddSeconds([int]$Lane.deadline_seconds)
    $laneIdentityMap = @{}
    foreach ($identity in @($AllLaneIdentities)) {
        $laneIdentityMap[[string]$identity.lane_id] = $identity
    }
    if (-not $laneIdentityMap.ContainsKey([string]$Lane.id)) {
        throw "Validation lifecycle did not receive the current lane identity."
    }
    $targetLaneIdentity = $null
    if ($Lane.topology.PSObject.Properties.Name -contains "handoff_to") {
        $targetId = [string]$Lane.topology.handoff_to
        if (-not $laneIdentityMap.ContainsKey($targetId)) {
            throw "Validation lifecycle did not receive topology target identity '$targetId'."
        }
        $targetLaneIdentity = $laneIdentityMap[$targetId]
    }
    $sourceLaneIdentity = $null
    if ($Lane.topology.PSObject.Properties.Name -contains "from_lane") {
        $sourceId = [string]$Lane.topology.from_lane
        if (-not $laneIdentityMap.ContainsKey($sourceId)) {
            throw "Validation lifecycle did not receive topology source identity '$sourceId'."
        }
        $sourceLaneIdentity = $laneIdentityMap[$sourceId]
    }
    if (-not [string]::IsNullOrWhiteSpace($FinalizeCheckpointPath)) {
        if (-not [string]::IsNullOrWhiteSpace($TopologyReceiptPath) -or
            @($PrerequisiteResultPaths).Count -ne 0 -or
            -not [string]::IsNullOrWhiteSpace($CancellationPath)) {
            throw "Finalization-only recovery cannot consume topology, prerequisite, or cancellation inputs."
        }
        Assert-TessaraLifecycleLaneInputsUnchanged -LaneIdentity $LaneIdentity `
            -RepositoryRoot $repository -PlatformIdentity $PlatformIdentity
        $recoveryCheckpoint = Read-TessaraLifecycleHashedJson `
            -Path $FinalizeCheckpointPath -Label "Execution checkpoint" `
            -TrustedRoot $evidence
        $recoveryAttemptRoot = Split-Path -Parent ([string]$recoveryCheckpoint.path)
        $null = Ensure-TessaraLifecycleExecutionIntegrityCommit `
            -AttemptRoot $recoveryAttemptRoot -EvidenceRoot $evidence `
            -CheckpointReference $recoveryCheckpoint `
            -ExpectedLaneId ([string]$Lane.id) `
            -ExpectedAttemptId ([string]$recoveryCheckpoint.document.result.attempt_id) `
            -CandidateFingerprint $CandidateFingerprint -LaneIdentity $LaneIdentity `
            -ExecutionPlatformFingerprint $PlatformExecutionFingerprint `
            -EvidenceRootFingerprint $evidenceRootFingerprint
        $finalizedRecovery = Complete-TessaraValidationEvidence -CheckpointPath $FinalizeCheckpointPath `
            -EvidenceRoot $evidence -LaneId ([string]$Lane.id) `
            -CandidateFingerprint $CandidateFingerprint -LaneIdentity $LaneIdentity `
            -ExecutionPlatformFingerprint $PlatformExecutionFingerprint `
            -FinalizationPlatformFingerprint ([string]$PlatformIdentity.finalization_fingerprint)
        if ([string]$finalizedRecovery.state -cne "passed") {
            throw "Validation lane '$($Lane.id)' finalization recovered a '$($finalizedRecovery.state)' execution; evidence: $($finalizedRecovery.evidence_path)."
        }
        return $finalizedRecovery
    }
    $attemptId = "{0}-{1}" -f [DateTimeOffset]::UtcNow.ToString("yyyyMMddTHHmmssfffZ"), ([guid]::NewGuid().ToString("N").Substring(0, 8))
    $attemptRoot = Join-Path $evidence "lanes/$($Lane.id)/attempts/$attemptId"
    $attemptRoot = Initialize-TessaraLifecycleOwnedEvidenceDirectory `
        -EvidenceRoot $evidence -Path $attemptRoot
    $attemptTempRoot = Initialize-TessaraLifecycleOwnedEvidenceDirectory `
        -EvidenceRoot $evidence -Path (Join-Path $attemptRoot "temp")
    $topology = $Lane.topology
    $topologyPorts = @(if ($topology.PSObject.Properties.Name -contains "ports") {
        @($topology.ports)
    } else { @() })
    $adapterProjectId = ([string]$Adapter.adapter_id).Substring(
        0,
        [Math]::Min(16, ([string]$Adapter.adapter_id).Length)
    )
    $topologyLease = if ([string]$topology.provider -ceq "docker-compose" -and
        [string]$topology.mode -ceq "create") {
        [Convert]::ToHexString([Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).ToLowerInvariant()
    } else { $null }
    $readinessCapability = if ([string]$topology.provider -ceq "none") {
        $null
    } elseif ([string]$topology.mode -ceq "create") {
        [Convert]::ToHexString(
            [Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
        ).ToLowerInvariant()
    } else { $null }
    $topologyAttemptId = $attemptId
    $project = if ($null -ne $topologyLease) {
        "tessara-vp-$adapterProjectId-$($topologyLease.Substring(0, 32))"
    } else {
        "tessara-vp-$adapterProjectId-$($attemptId.Substring($attemptId.Length - 8))"
    }
    $portLease = $null
    $ports = [ordered]@{}
    $topologyProcess = $null
    $topologyOwned = $false
    $cleanup = [pscustomobject][ordered]@{ state = "not_required"; mode = "none" }
    $restoration = [pscustomobject][ordered]@{ state = "pending" }
    $failure = $null
    $integrityFailureMessage = $null
    $failureStage = $null
    $currentStage = "setup"
    $currentActionId = $null
    $state = "passed"
    $actionResults = [Collections.Generic.List[object]]::new()
    $assertionActionCount = @($Lane.actions | Where-Object { [string]$_.stage -ceq "assertion" }).Count
    $assertionActionsCompleted = 0
    $assertionsStarted = $false
    $environmentPlan = $null
    $toolBindings = @()
    $commandBindings = @{}
    $toolFingerprint = $null
    $toolObservationFingerprint = $null
    $processEnvironment = Get-TessaraLifecycleBaseProcessEnvironment
    $composeConfigurationSha256 = $null
    $composeRuntimeIdentitySha256 = $null
    $dockerProviderIdentity = $null
    $composeFileSha256 = $null
    $topologyReceiptSnapshot = $null
    $topologyClaimRef = $null
    $handoffPending = $false
    $suppressOuterHandoffCleanup = $false
    $prerequisiteResults = [Collections.Generic.List[object]]::new()
    $context = [pscustomobject][ordered]@{
        repository_root = $repository
        attempt_root = $attemptRoot
        temp_root = $attemptTempRoot
        candidate_fingerprint = $CandidateFingerprint
        candidate_source_identity = $LaneIdentity.candidate_source_identity
        acceptance_fingerprint = $AcceptanceFingerprint
        adapter_fingerprint = $AdapterFingerprint
        platform_fingerprint = $PlatformExecutionFingerprint
        compose_project = $project
        topology_lease = $topologyLease
        topology_attempt_id = $topologyAttemptId
        docker_config = Join-Path $attemptRoot "docker-config"
        compose_env_file = Join-Path $attemptRoot "compose-environment.empty"
        ports = $ports
    }
    Write-TessaraLifecycleNewJson -EvidenceRoot $evidence `
        -Path (Join-Path $attemptRoot "attempt-intent.json") `
        -Document ([pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.attempt-intent"
            state = "declared"
            adapter_id = [string]$Adapter.adapter_id
            lane_id = [string]$Lane.id
            phase = [string]$LaneIdentity.phase
            attempt_id = $attemptId
            candidate_fingerprint = $CandidateFingerprint
            acceptance_fingerprint = $AcceptanceFingerprint
            adapter_fingerprint = $AdapterFingerprint
            platform_execution_fingerprint = $PlatformExecutionFingerprint
            lane_compatibility_fingerprint = [string]$LaneIdentity.compatibility_fingerprint
            evidence_root_fingerprint = $evidenceRootFingerprint
            actions = @($Lane.actions | ForEach-Object { [pscustomobject][ordered]@{
                id = [string]$_.id
                stage = [string]$_.stage
            } })
        }) | Out-Null
    try {
        $topologyPrerequisite = if ([string]$topology.mode -ceq "consume") {
            [string]$topology.from_lane
        } else { $null }
        $ordinaryPrerequisites = @($Lane.prerequisites | Where-Object {
            [string]$_ -cne $topologyPrerequisite
        } | ForEach-Object { [string]$_ } | Sort-Object)
        if (@($PrerequisiteResultPaths).Count -ne $ordinaryPrerequisites.Count) {
            Stop-TessaraLifecycleBlocked -Message (
                "Lane '$($Lane.id)' requires exact passing results for prerequisites: " +
                (($ordinaryPrerequisites -join ", ") -replace '^$', '<none>')
            )
        }
        $seenPrerequisites = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($prerequisitePath in @($PrerequisiteResultPaths)) {
            $snapshot = Read-TessaraLifecycleHashedJson -Path $prerequisitePath `
                -Label "Prerequisite lane result" -TrustedRoot $evidence
            $relative = [IO.Path]::GetRelativePath($evidence, [string]$snapshot.path)
            $segments = @(Split-TessaraLifecycleNativePath -Path $relative)
            if ($segments.Count -ne 5 -or $segments[0] -cne "lanes" -or
                $segments[2] -cne "attempts" -or
                $segments[3] -cnotmatch '^\d{8}T\d{9}Z-[0-9a-f]{8}$' -or
                $segments[4] -cne "lane-result.json") {
                Stop-TessaraLifecycleBlocked -Message (
                    "Prerequisite result is not a canonical lane attempt under this evidence root."
                )
            }
            $prerequisiteId = [string]$segments[1]
            $prerequisiteAttemptRoot = Split-Path -Parent ([string]$snapshot.path)
            $prerequisiteRevocationPath = Join-Path $prerequisiteAttemptRoot `
                "execution-revoked.json"
            if ((Test-Path -LiteralPath $prerequisiteRevocationPath -PathType Leaf) -or
                (Test-Path -LiteralPath "$prerequisiteRevocationPath.sha256" -PathType Leaf)) {
                Stop-TessaraLifecycleBlocked -Message (
                    "Prerequisite result '$prerequisiteId' belongs to a revoked execution."
                )
            }
            if ($prerequisiteId -notin $ordinaryPrerequisites -or
                -not $seenPrerequisites.Add($prerequisiteId)) {
                Stop-TessaraLifecycleBlocked -Message (
                    "Prerequisite result '$prerequisiteId' is unexpected or duplicated for lane '$($Lane.id)'."
                )
            }
            $expectedIdentity = $laneIdentityMap[$prerequisiteId]
            $result = $snapshot.document
            $resultPhase = [string]$expectedIdentity.phase
            $requiresCandidateMatch = $resultPhase -in @("validation-preflight", "sit", "uat") -or
                [string]$LaneIdentity.phase -in @("validation-preflight", "sit", "uat")
            $null = Read-TessaraLifecycleFinalizationCommit `
                -AttemptRoot $prerequisiteAttemptRoot -ResultSnapshot $snapshot `
                -ExpectedLaneId $prerequisiteId -ExpectedAttemptId ([string]$segments[3]) `
                -CurrentFinalizationFingerprint ([string]$PlatformIdentity.finalization_fingerprint) `
                -EvidenceRoot $evidence `
                -Label "Prerequisite"
            if ([string]$result.contract -cne "tessara.validation.lane-result" -or
                [string]$result.state -cne "passed" -or -not [bool]$result.authoritative -or
                [string]$result.lane_id -cne $prerequisiteId -or
                [string]$result.attempt_id -cne [string]$segments[3] -or
                [string]$result.adapter_fingerprint -cne [string]$expectedIdentity.adapter_fingerprint -or
                [string]$result.acceptance_fingerprint -cne [string]$expectedIdentity.acceptance_fingerprint -or
                [string]$result.lane_compatibility_fingerprint -cne
                    [string]$expectedIdentity.compatibility_fingerprint -or
                [string]$result.platform_execution_fingerprint -cne $PlatformExecutionFingerprint -or
                [string]$result.evidence_root_fingerprint -cne $evidenceRootFingerprint -or
                ($requiresCandidateMatch -and
                    [string]$result.candidate_fingerprint -cne $CandidateFingerprint)) {
                Stop-TessaraLifecycleBlocked -Message (
                    "Prerequisite '$prerequisiteId' is not one authoritative compatible passing attempt."
                )
            }
            $prerequisiteResults.Add([pscustomobject][ordered]@{
                lane_id = $prerequisiteId
                attempt_id = [string]$result.attempt_id
                result = [pscustomobject][ordered]@{
                    path = [string]$snapshot.path
                    sha256 = [string]$snapshot.sha256
                }
                compatibility_fingerprint = [string]$result.lane_compatibility_fingerprint
            })
        }
        if ($seenPrerequisites.Count -ne $ordinaryPrerequisites.Count) {
            Stop-TessaraLifecycleBlocked -Message "Lane '$($Lane.id)' is missing a direct prerequisite result."
        }
        Assert-TessaraLifecycleNotCancelled -CancellationPath $CancellationPath `
            -Boundary "topology ownership acquisition"
        if ([string]$topology.mode -ceq "consume") {
            if ([string]::IsNullOrWhiteSpace($TopologyReceiptPath)) {
                Stop-TessaraLifecycleBlocked -Message `
                    "Lane '$($Lane.id)' requires a topology handoff prerequisite receipt."
            }
            $topologyReceiptSnapshot = Read-TessaraLifecycleHashedJson `
                -Path $TopologyReceiptPath -Label "Topology receipt" `
                -TrustedRoot $evidence
            $receiptPath = [string]$topologyReceiptSnapshot.path
            $relativeReceipt = [IO.Path]::GetRelativePath($evidence, $receiptPath)
            $receiptSegments = @(Split-TessaraLifecycleNativePath -Path $relativeReceipt)
            $expectedSourceLane = [string]$topology.from_lane
            if ($receiptSegments.Count -ne 5 -or
                $receiptSegments[0] -cne "lanes" -or
                $receiptSegments[1] -cne $expectedSourceLane -or
                $receiptSegments[2] -cne "attempts" -or
                $receiptSegments[3] -cnotmatch '^\d{8}T\d{9}Z-[0-9a-f]{8}$' -or
                $receiptSegments[4] -cne "topology-receipt.json") {
                throw "Topology receipt is not the canonical source-attempt receipt for this handoff."
            }
            $sourceAttemptId = $receiptSegments[3]
            $receipt = $topologyReceiptSnapshot.document
            $receivedLease = [string]$receipt.topology_lease
            if ($receivedLease -cnotmatch '^[0-9a-f]{64}$') {
                throw "Topology receipt does not contain a valid ownership capability."
            }
            $receivedLeaseSha256 = Get-TessaraLifecycleSha256Text -Text $receivedLease
            $expectedProject = "tessara-vp-$adapterProjectId-$($receivedLease.Substring(0, 32))"
            $receiptPortNames = @($receipt.ports.PSObject.Properties.Name | Sort-Object)
            $declaredPortNames = @($topologyPorts | ForEach-Object { [string]$_.name } | Sort-Object)
            if ([int]$receipt.schema_version -ne 1 -or
                [string]$receipt.contract -cne "tessara.validation.topology-receipt" -or
                [string]$receipt.state -cne "retained" -or
                [string]$receipt.provider -cne "docker-compose" -or
                [string]$receipt.from_lane -cne $expectedSourceLane -or
                [string]$receipt.source_attempt_id -cne $sourceAttemptId -or
                [string]$receipt.handoff_to -cne [string]$Lane.id -or
                [string]$receipt.compose_project -cne $expectedProject -or
                [string]$receipt.topology_lease_sha256 -cne $receivedLeaseSha256 -or
                [string]$receipt.readiness_capability -cnotmatch '^[0-9a-f]{64}$' -or
                [string]$receipt.evidence_root_fingerprint -cne $evidenceRootFingerprint -or
                ($receiptPortNames -join "`n") -cne ($declaredPortNames -join "`n") -or
                [string]$receipt.source_adapter_fingerprint -cne
                    [string]$sourceLaneIdentity.adapter_fingerprint -or
                [string]$receipt.source_acceptance_fingerprint -cne
                    [string]$sourceLaneIdentity.acceptance_fingerprint -or
                [string]$receipt.source_compatibility_fingerprint -cne
                    [string]$sourceLaneIdentity.compatibility_fingerprint -or
                [string]$receipt.target_adapter_fingerprint -cne $AdapterFingerprint -or
                [string]$receipt.target_acceptance_fingerprint -cne $AcceptanceFingerprint -or
                [string]$receipt.platform_execution_fingerprint -cne $PlatformExecutionFingerprint -or
                [string]$receipt.candidate_fingerprint -cne $CandidateFingerprint -or
                [string]$receipt.target_compatibility_fingerprint -cne
                    [string]$LaneIdentity.compatibility_fingerprint) {
                throw "Topology receipt does not bind the exact source, target, project, ports, and identity tuple."
            }
            foreach ($property in $receipt.ports.PSObject.Properties) {
                $portValue = [int]$property.Value
                if ($portValue -lt 1 -or $portValue -gt 65535) {
                    throw "Topology receipt contains an invalid port for '$($property.Name)'."
                }
                $ports[$property.Name] = $portValue
            }
            $context.compose_project = $expectedProject
            $context.topology_lease = $receivedLease
            $context.topology_attempt_id = $sourceAttemptId
            $topologyLease = $receivedLease
            $topologyAttemptId = $sourceAttemptId
            $sourceAttemptRoot = Split-Path -Parent $receiptPath
            $sourceRevocationPath = Join-Path $sourceAttemptRoot "execution-revoked.json"
            if ((Test-Path -LiteralPath $sourceRevocationPath -PathType Leaf) -or
                (Test-Path -LiteralPath "$sourceRevocationPath.sha256" -PathType Leaf)) {
                throw "Topology source attempt was revoked before handoff consumption."
            }
            $sourceResultSnapshot = Read-TessaraLifecycleHashedJson `
                -Path (Join-Path $sourceAttemptRoot "lane-result.json") `
                -Label "Topology source lane result" -TrustedRoot $evidence
            $sourceResult = $sourceResultSnapshot.document
            $null = Read-TessaraLifecycleFinalizationCommit `
                -AttemptRoot $sourceAttemptRoot -ResultSnapshot $sourceResultSnapshot `
                -ExpectedLaneId $expectedSourceLane -ExpectedAttemptId $sourceAttemptId `
                -CurrentFinalizationFingerprint ([string]$PlatformIdentity.finalization_fingerprint) `
                -EvidenceRoot $evidence `
                -Label "Topology source"
            if ([string]$sourceResult.state -cne "passed" -or
                -not [bool]$sourceResult.authoritative -or
                [string]$sourceResult.lane_id -cne $expectedSourceLane -or
                [string]$sourceResult.attempt_id -cne $sourceAttemptId -or
                [string]$sourceResult.adapter_fingerprint -cne
                    [string]$sourceLaneIdentity.adapter_fingerprint -or
                [string]$sourceResult.acceptance_fingerprint -cne
                    [string]$sourceLaneIdentity.acceptance_fingerprint -or
                [string]$sourceResult.lane_compatibility_fingerprint -cne
                    [string]$sourceLaneIdentity.compatibility_fingerprint -or
                [string]$sourceResult.platform_execution_fingerprint -cne
                    $PlatformExecutionFingerprint -or
                [string]$sourceResult.evidence_root_fingerprint -cne $evidenceRootFingerprint -or
                [string]$sourceResult.candidate_fingerprint -cne $CandidateFingerprint -or
                [string]$sourceResult.topology_lease_sha256 -cne $receivedLeaseSha256 -or
                [string]$sourceResult.cleanup_restoration.mode -cne "retained-for-handoff" -or
                [string]$sourceResult.cleanup_restoration.receipt.sha256 -cne $topologyReceiptSnapshot.sha256) {
                throw "Topology receipt is not bound to one passing indexed source attempt."
            }
            $prerequisiteResults.Add([pscustomobject][ordered]@{
                lane_id = $expectedSourceLane
                attempt_id = $sourceAttemptId
                result = [pscustomobject][ordered]@{
                    path = [string]$sourceResultSnapshot.path
                    sha256 = [string]$sourceResultSnapshot.sha256
                }
                compatibility_fingerprint = [string]$sourceResult.lane_compatibility_fingerprint
            })
        } elseif ([string]$topology.mode -ceq "create") {
            $portLease = New-TessaraLifecyclePortLease -Ports $topologyPorts
            foreach ($name in @($portLease.values.Keys)) { $ports[$name] = $portLease.values[$name] }
        }

        if ([string]$topology.provider -ceq "docker-compose") {
            $null = Initialize-TessaraLifecycleOwnedEvidenceDirectory `
                -EvidenceRoot $evidence -Path ([string]$context.docker_config)
            Write-TessaraLifecycleNewText -EvidenceRoot $evidence `
                -Path ([string]$context.compose_env_file) `
                -Text "" | Out-Null
        }
        $environmentPlan = New-TessaraLifecycleProcessEnvironmentPlan `
            -Lane $Lane -Topology $topology -Context $context -TopologyPorts $topologyPorts
        $actualEnvironmentObservationFingerprint = `
            Get-TessaraLifecycleEnvironmentObservationFingerprint `
                -EnvironmentPlan $environmentPlan
        if ($actualEnvironmentObservationFingerprint -cne
            [string]$LaneIdentity.environment_observation_fingerprint) {
            throw "Validation environment changed between lane planning and execution."
        }
        $processEnvironment = $environmentPlan.environment
        if ([string]$topology.mode -ceq "consume") {
            $readinessCapability = [string]$topologyReceiptSnapshot.document.readiness_capability
        }
        if ([string]$topology.provider -ne "none") {
            if ($readinessCapability -cnotmatch '^[0-9a-f]{64}$') {
                throw "Topology readiness capability is missing or malformed."
            }
            $processEnvironment["TESSARA_VALIDATION_READINESS_CAPABILITY"] = $readinessCapability
            $environmentPlan.redactions = @($environmentPlan.redactions) + @($readinessCapability)
        }
        $bindings = [Collections.Generic.List[object]]::new()
        if ([string]$topology.provider -ceq "local-process") {
            $bindings.Add((Get-TessaraLifecycleCommandBinding -Id "topology" `
                -Program (Expand-TessaraLifecycleValue -Value ([string]$topology.command.program) -Context $context) `
                -Arguments @($topology.command.arguments | ForEach-Object {
                    Expand-TessaraLifecycleValue -Value ([string]$_) -Context $context
                }) -RepositoryRoot $repository `
                -InputPaths @($topology.command.input_paths)))
            foreach ($declaredTool in @($topology.command.tools)) {
                $bindings.Add((Get-TessaraLifecycleCommandBinding `
                    -Id "topology-tool:$([string]$declaredTool.id)" `
                    -Program ([string]$declaredTool.program) -Arguments @() `
                    -RepositoryRoot $repository -InputPaths @($declaredTool.input_paths)))
            }
        } elseif ([string]$topology.provider -ceq "docker-compose") {
            [string[]]$dockerPrefix = @()
            if ($DockerCommand.Count -gt 1) {
                $dockerPrefix = @($DockerCommand[1..($DockerCommand.Count - 1)])
            }
            $bindings.Add((Get-TessaraLifecycleCommandBinding -Id "docker" `
                -Program ([string]$DockerCommand[0]) -Arguments $dockerPrefix `
                -RepositoryRoot $repository -InputPaths $DockerCommandInputPaths))
        }
        foreach ($declaredAction in @($Lane.actions)) {
            $bindings.Add((Get-TessaraLifecycleCommandBinding `
                -Id "action:$([string]$declaredAction.id)" `
                -Program (Expand-TessaraLifecycleValue -Value ([string]$declaredAction.program) -Context $context) `
                -Arguments @($declaredAction.arguments | ForEach-Object {
                    Expand-TessaraLifecycleValue -Value ([string]$_) -Context $context
                }) -RepositoryRoot $repository `
                -InputPaths @($declaredAction.input_paths)))
            foreach ($declaredTool in @($declaredAction.tools)) {
                $bindings.Add((Get-TessaraLifecycleCommandBinding `
                    -Id "action:$([string]$declaredAction.id):tool:$([string]$declaredTool.id)" `
                    -Program ([string]$declaredTool.program) -Arguments @() `
                    -RepositoryRoot $repository -InputPaths @($declaredTool.input_paths)))
            }
        }
        $toolBindings = @($bindings)
        $toolDirectories = @($toolBindings | ForEach-Object {
            Split-Path -Parent ([string]$_.resolved_program)
        } | Sort-Object -CaseSensitive -Unique)
        $processEnvironment["PATH"] = $toolDirectories -join [IO.Path]::PathSeparator
        $processEnvironment["PSModulePath"] = Join-Path $PSHOME "Modules"
        foreach ($binding in $toolBindings) {
            $commandBindings[[string]$binding.id] = $binding
        }
        $context | Add-Member -NotePropertyName command_bindings `
            -NotePropertyValue $commandBindings
        $actualProgramObservations = @($toolBindings | Sort-Object id | ForEach-Object {
            [pscustomobject][ordered]@{
                id = [string]$_.id
                present = $true
                program_path_sha256 = [string]$_.program_path_sha256
                program_sha256 = [string]$_.program_sha256
                input_paths = @($_.input_paths)
            }
        })
        $dockerPrefixObservation = if ([string]$topology.provider -ceq "docker-compose" -and
            $DockerCommand.Count -gt 1) { @($DockerCommand[1..($DockerCommand.Count - 1)]) } else { @() }
        $toolObservationValue = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value (
            [pscustomobject][ordered]@{
                programs = @($actualProgramObservations)
                docker_prefix = @($dockerPrefixObservation)
            }
        )
        $toolObservationFingerprint = Get-TessaraLifecycleSha256Text -Text (
            ($toolObservationValue | ConvertTo-Json -Depth 50 -Compress) + "`n"
        )
        if ($toolObservationFingerprint -cne [string]$LaneIdentity.tool_observation_fingerprint) {
            throw "Validation tool identity changed between lane planning and execution."
        }
        $toolFingerprintValue = ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $toolBindings
        $toolFingerprint = Get-TessaraLifecycleSha256Text -Text (
            ($toolFingerprintValue | ConvertTo-Json -Depth 50 -Compress) + "`n"
        )
        if ([string]$topology.provider -ceq "docker-compose") {
            $dockerProviderIdentity = Get-TessaraLifecycleDockerProviderIdentity `
                -Topology $topology -Context $context -Environment $processEnvironment `
                -DockerCommand $DockerCommand -CancellationPath $CancellationPath
            if ([string]$topology.mode -ceq "consume" -and
                ([string]$topologyReceiptSnapshot.document.docker_provider_fingerprint -cne
                    [string]$dockerProviderIdentity.fingerprint -or
                    [string]$topologyReceiptSnapshot.document.compose_runtime_identity_sha256 -cnotmatch
                        '^[0-9a-f]{64}$')) {
                throw "Topology receipt Docker provider provenance does not match the current daemon."
            }
        }
        $startDocument = [pscustomobject][ordered]@{
            schema_version = 1
            contract = "tessara.validation.attempt-start"
            state = "started"
            adapter_id = [string]$Adapter.adapter_id
            lane_id = [string]$Lane.id
            attempt_id = $attemptId
            candidate_fingerprint = $CandidateFingerprint
            acceptance_fingerprint = $AcceptanceFingerprint
            adapter_fingerprint = $AdapterFingerprint
            platform_execution_fingerprint = $PlatformExecutionFingerprint
            lane_compatibility_fingerprint = [string]$LaneIdentity.compatibility_fingerprint
            evidence_root_fingerprint = $evidenceRootFingerprint
            environment_fingerprint = [string]$environmentPlan.fingerprint
            environment_bindings = @($environmentPlan.bindings)
            tool_fingerprint = $toolFingerprint
            tool_observation_fingerprint = $toolObservationFingerprint
            tool_bindings = @($toolBindings)
            docker_provider_identity = $dockerProviderIdentity
            prerequisite_results = @($prerequisiteResults)
            actions = @($Lane.actions | ForEach-Object { [pscustomobject][ordered]@{
                id = [string]$_.id
                stage = [string]$_.stage
            } })
        }
        Write-TessaraLifecycleNewJson -EvidenceRoot $evidence `
            -Path (Join-Path $attemptRoot "attempt-start.json") `
            -Document $startDocument | Out-Null
        if (@($environmentPlan.errors).Count -ne 0) {
            throw (@($environmentPlan.errors) -join " ")
        }

        if ([string]$topology.provider -ceq "docker-compose") {
            $composePath = Resolve-TessaraLifecycleRepositoryPath `
                -RepositoryRoot $repository -Path ([string]$topology.compose_file) `
                -Label "Compose file"
            $composeFileSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $composePath).Hash.ToLowerInvariant()
            if ([string]$topology.mode -ceq "consume" -and
                $composeFileSha256 -cne [string]$topologyReceiptSnapshot.document.compose_file_sha256) {
                throw "Topology receipt Compose file identity does not match the consuming adapter."
            }
        }

        if ([string]$topology.provider -ceq "local-process") {
            $topologyProgram = Expand-TessaraLifecycleValue `
                -Value ([string]$topology.command.program) -Context $context
            $topologyArguments = @($topology.command.arguments | ForEach-Object {
                Expand-TessaraLifecycleValue -Value ([string]$_) -Context $context
            })
            $topologyLaunchBinding = Assert-TessaraLifecycleCommandBindingUnchanged `
                -ExpectedBinding $commandBindings["topology"] `
                -Program $topologyProgram -Arguments $topologyArguments `
                -RepositoryRoot $repository
        } elseif ([string]$topology.provider -ceq "docker-compose") {
            [string[]]$dockerPrefix = @()
            if ($DockerCommand.Count -gt 1) {
                $dockerPrefix = @($DockerCommand[1..($DockerCommand.Count - 1)])
            }
            $null = Assert-TessaraLifecycleCommandBindingUnchanged `
                -ExpectedBinding $commandBindings["docker"] `
                -Program ([string]$DockerCommand[0]) -Arguments $dockerPrefix `
                -RepositoryRoot $repository
        }

        if ([string]$topology.mode -ceq "consume") {
            # Resolve and authenticate every read-only consumer input before
            # publishing the one-time claim. A malformed or drifted consumer
            # must not consume (and consequently tear down) a reusable source
            # topology merely because its configuration is invalid.
            if ($CertificationFault -ceq "consumer-compose-config-substitution") {
                $processEnvironment["TESSARA_VP_DOCKER_SHIM_FOREIGN_CONFIG"] = "1"
            }
            $configuration = Get-TessaraLifecycleComposeConfiguration `
                -Topology $topology -Context $context -Environment $processEnvironment `
                -DockerCommand $DockerCommand -CancellationPath $CancellationPath
            $composeConfigurationSha256 = [string]$configuration.normalized_sha256
            if ($composeConfigurationSha256 -cne
                [string]$topologyReceiptSnapshot.document.compose_configuration_sha256) {
                throw "Topology receipt Compose configuration identity does not match the consumer."
            }
        }

        Assert-TessaraLifecycleNotCancelled -CancellationPath $CancellationPath `
            -Boundary "topology claim publication"
        if ([string]$topology.mode -ceq "consume") {
            $claimPath = Join-Path $evidence "topology-claims/$receivedLeaseSha256.json"
            $claimPublicationHook = if (
                $CertificationFault -ceq "partial-claim-publication"
            ) {
                { throw "Injected failure after topology claim data creation." }
            } else { $null }
            try {
                $topologyClaimRef = Write-TessaraLifecycleNewJson `
                    -EvidenceRoot $evidence -Path $claimPath `
                    -AfterDataCreate $claimPublicationHook -RetainDataOnFailure -Document (
                    [pscustomobject][ordered]@{
                        schema_version = 1
                        contract = "tessara.validation.topology-consumption-claim"
                        receipt_sha256 = [string]$topologyReceiptSnapshot.sha256
                        topology_lease_sha256 = $receivedLeaseSha256
                        from_lane = [string]$topology.from_lane
                        to_lane = [string]$Lane.id
                        consumer_attempt_id = $attemptId
                        evidence_root_fingerprint = $evidenceRootFingerprint
                    }
                )
            } catch {
                if ([bool]$_.Exception.Data["TessaraEvidenceDataCreated"]) {
                    $topologyOwned = $true
                }
                throw
            }
            if ($CertificationFault -ceq "after-consumer-claim") {
                throw "Injected failure after complete topology claim publication."
            }
            $topologyOwned = $true
        }

        # Setup actions are the first adapter-controlled operations that may
        # mutate an already-running handoff topology. Consumers therefore
        # acquire the single-use claim before setup; creators still perform
        # setup before starting their new topology.
        foreach ($action in @($Lane.actions | Where-Object {
                [string]$_.stage -ceq "setup"
            })) {
            $currentStage = "setup"
            $currentActionId = [string]$action.id
            foreach ($declaredTool in @($action.tools)) {
                $null = Assert-TessaraLifecycleCommandBindingUnchanged `
                    -ExpectedBinding $commandBindings["action:$([string]$action.id):tool:$([string]$declaredTool.id)"] `
                    -Program ([string]$declaredTool.program) -Arguments @() `
                    -RepositoryRoot $repository
            }
            $deadlineAction = Get-TessaraLifecycleDeadlineAction -Action $action `
                -Deadline $laneDeadline
            $actionResult = Invoke-TessaraLifecycleDeclaredAction -Action $deadlineAction `
                -Context $context -RepositoryRoot $repository -EvidenceRoot $evidence `
                -AttemptRoot $attemptRoot `
                -Environment $processEnvironment `
                -ExpectedBinding $commandBindings["action:$([string]$action.id)"] `
                -RedactedValues @($environmentPlan.redactions) `
                -CancellationPath $CancellationPath `
                -CertificationFault $CertificationFault
            $actionResults.Add($actionResult)
            Assert-TessaraLifecycleActionPassed -ActionResult $actionResult
        }
        $currentActionId = $null
        Assert-TessaraLifecycleNotCancelled -CancellationPath $CancellationPath `
            -Boundary "post-setup ownership gate"
        Assert-TessaraLifecycleLaneInputsUnchanged -LaneIdentity $LaneIdentity `
            -RepositoryRoot $repository -PlatformIdentity $PlatformIdentity
        if ($CertificationFault -ceq "after-post-setup-verification") {
            throw "Validation command 'topology' identity changed immediately before invocation."
        }

        if ([string]$topology.mode -ceq "create") {
            Assert-TessaraLifecycleNotCancelled -CancellationPath $CancellationPath `
                -Boundary "topology start"
            if ([string]$topology.provider -ceq "local-process") {
                foreach ($declaredTool in @($topology.command.tools)) {
                    $null = Assert-TessaraLifecycleCommandBindingUnchanged `
                        -ExpectedBinding $commandBindings["topology-tool:$([string]$declaredTool.id)"] `
                        -Program ([string]$declaredTool.program) -Arguments @() `
                        -RepositoryRoot $repository
                }
                $topologyProgram = Expand-TessaraLifecycleValue `
                    -Value ([string]$topology.command.program) -Context $context
                $topologyArguments = @($topology.command.arguments | ForEach-Object {
                    Expand-TessaraLifecycleValue -Value ([string]$_) -Context $context
                })
                $topologyLaunchBinding = Assert-TessaraLifecycleCommandBindingUnchanged `
                    -ExpectedBinding $commandBindings["topology"] `
                    -Program $topologyProgram -Arguments $topologyArguments `
                    -RepositoryRoot $repository
            }
            if ([string]$topology.provider -ne "none") {
                $topologyOwned = $true
                Write-TessaraLifecycleNewJson -EvidenceRoot $evidence `
                    -Path (Join-Path $attemptRoot "topology-ownership-acquired.json") `
                    -Document ([pscustomobject][ordered]@{
                        schema_version = 1
                        contract = "tessara.validation.topology-ownership"
                        state = "acquired"
                        provider = [string]$topology.provider
                        compose_project = if ([string]$topology.provider -ceq "docker-compose") { [string]$context.compose_project } else { $null }
                        topology_lease_sha256 = if ($null -eq $topologyLease) {
                            $null
                        } else { Get-TessaraLifecycleSha256Text -Text $topologyLease }
                        attempt_id = $attemptId
                        candidate_fingerprint = $CandidateFingerprint
                        lane_compatibility_fingerprint = [string]$LaneIdentity.compatibility_fingerprint
                        platform_execution_fingerprint = $PlatformExecutionFingerprint
                        evidence_root_fingerprint = $evidenceRootFingerprint
                        ports = [pscustomobject]$ports
                    }) | Out-Null
            }
            Close-TessaraLifecyclePortLease -Lease $portLease
            $portLease = $null
            if ([string]$topology.provider -ceq "local-process") {
                $topologyProcess = Start-TessaraLifecycleProcess `
                    -Program ([string]$topologyLaunchBinding.resolved_program) `
                    -Arguments $topologyArguments -WorkingDirectory $repository `
                    -Environment $processEnvironment -ProcessKind "local-topology" `
                    -CertificationFault $CertificationFault
                $topologyStdoutCapture = [Tessara.Validation.BoundedCaptureStream]::new(
                    $script:MaximumCapturedStreamBytes
                )
                $topologyStderrCapture = [Tessara.Validation.BoundedCaptureStream]::new(
                    $script:MaximumCapturedStreamBytes
                )
                $topologyProcess | Add-Member -NotePropertyName TessaraStdoutCapture `
                    -NotePropertyValue $topologyStdoutCapture
                $topologyProcess | Add-Member -NotePropertyName TessaraStderrCapture `
                    -NotePropertyValue $topologyStderrCapture
                $topologyProcess | Add-Member -NotePropertyName TessaraStdoutTask `
                    -NotePropertyValue $topologyProcess.StandardOutput.BaseStream.CopyToAsync(
                        $topologyStdoutCapture
                    )
                $topologyProcess | Add-Member -NotePropertyName TessaraStderrTask `
                    -NotePropertyValue $topologyProcess.StandardError.BaseStream.CopyToAsync(
                        $topologyStderrCapture
                    )
            } elseif ([string]$topology.provider -ceq "docker-compose") {
                $configuration = Get-TessaraLifecycleComposeConfiguration `
                    -Topology $topology -Context $context -Environment $processEnvironment `
                    -DockerCommand $DockerCommand -CancellationPath $CancellationPath
                $composeConfigurationSha256 = [string]$configuration.normalized_sha256
                $base = Get-TessaraLifecycleComposeArguments -Topology $topology -Context $context
                Invoke-TessaraLifecycleDocker -DockerCommand $DockerCommand `
                    -Arguments @($base + @("up", "-d")) -RepositoryRoot $repository `
                    -Environment $processEnvironment `
                    -ExpectedBinding $commandBindings["docker"] `
                    -CancellationPath $CancellationPath | Out-Null
            }
        }
        if ([string]$topology.provider -ceq "docker-compose") {
            $composeRuntimeIdentitySha256 = Assert-TessaraLifecycleComposeHealthy `
                -Topology $topology -Context $context `
                -Environment $processEnvironment -DockerCommand $DockerCommand `
                -CancellationPath $CancellationPath
        }
        if ([string]$topology.provider -ne "none") {
            $readinessPort = [int]$ports[[string]$topology.readiness.port]
            $remainingReadiness = [int][Math]::Floor(
                ($laneDeadline - [DateTimeOffset]::UtcNow).TotalSeconds
            )
            if ($remainingReadiness -lt 1) {
                throw [TimeoutException]::new("Validation lane exceeded its aggregate deadline before readiness.")
            }
            Wait-TessaraLifecycleTcpReadiness -Port $readinessPort `
                -TimeoutSeconds ([Math]::Min(
                    [int]$topology.readiness.timeout_seconds, $remainingReadiness
                )) -Capability $readinessCapability `
                -CancellationPath $CancellationPath -Process $topologyProcess
        }

        foreach ($action in @($Lane.actions | Where-Object {
                [string]$_.stage -cne "setup"
            })) {
            $currentStage = [string]$action.stage
            $currentActionId = [string]$action.id
            $assertionProcessStarted = $false
            try {
                foreach ($declaredTool in @($action.tools)) {
                    $null = Assert-TessaraLifecycleCommandBindingUnchanged `
                        -ExpectedBinding $commandBindings["action:$([string]$action.id):tool:$([string]$declaredTool.id)"] `
                        -Program ([string]$declaredTool.program) -Arguments @() `
                        -RepositoryRoot $repository
                }
                $deadlineAction = Get-TessaraLifecycleDeadlineAction -Action $action `
                    -Deadline $laneDeadline
                $actionResult = Invoke-TessaraLifecycleDeclaredAction -Action $deadlineAction `
                    -Context $context -RepositoryRoot $repository -EvidenceRoot $evidence `
                    -AttemptRoot $attemptRoot `
                    -Environment $processEnvironment `
                    -ExpectedBinding $commandBindings["action:$([string]$action.id)"] `
                    -RedactedValues @($environmentPlan.redactions) `
                    -ProcessStarted ([ref]$assertionProcessStarted) `
                    -CancellationPath $CancellationPath `
                    -CertificationFault $CertificationFault
            } finally {
                if ($currentStage -ceq "assertion" -and $assertionProcessStarted) {
                    $assertionsStarted = $true
                }
            }
            $actionResults.Add($actionResult)
            Assert-TessaraLifecycleActionPassed -ActionResult $actionResult
            if ($currentStage -ceq "assertion") { $assertionActionsCompleted++ }
        }
        $currentActionId = $null
        Assert-TessaraLifecycleNotCancelled -CancellationPath $CancellationPath `
            -Boundary "post-assertion handoff gate"
        if ([string]$topology.provider -ceq "docker-compose" -and
            [string]$topology.on_success -ceq "handoff") {
            $configuration = Get-TessaraLifecycleComposeConfiguration `
                -Topology $topology -Context $context -Environment $processEnvironment `
                -DockerCommand $DockerCommand -CancellationPath $CancellationPath
            if ([string]$configuration.normalized_sha256 -cne $composeConfigurationSha256) {
                throw "Docker Compose configuration changed before topology handoff."
            }
            $handoffRuntimeIdentity = Assert-TessaraLifecycleComposeHealthy `
                -Topology $topology -Context $context `
                -Environment $processEnvironment -DockerCommand $DockerCommand `
                -CancellationPath $CancellationPath
            if ([string]$handoffRuntimeIdentity -cne $composeRuntimeIdentitySha256) {
                throw "Docker Compose runtime container identity changed before handoff."
            }
        }
    } catch [OperationCanceledException] {
        $state = "interrupted"
        $failure = $_
        $failureStage = $currentStage
    } catch {
        $state = if ($_.Exception.Data["TessaraLaneState"] -ceq "blocked") {
            "blocked"
        } else { "failed" }
        $failure = $_
        $failureStage = $currentStage
    } finally {
        Close-TessaraLifecyclePortLease -Lease $portLease
        $claimOwnsTopology = [string]$topology.mode -ceq "consume" -and
            $null -ne $topologyClaimRef
        $mustDestroy = ($topologyOwned -or $claimOwnsTopology) -and (
            $state -cne "passed" -or [string]$topology.on_success -ceq "destroy"
        )
        try {
            if ($mustDestroy) {
                if ([string]$topology.provider -ceq "local-process") {
                    Stop-TessaraLifecycleProcess -Process $topologyProcess
                    $topologyLogs = if ($null -eq $topologyProcess) { $null } else {
                        [void]$topologyProcess.TessaraStdoutTask.GetAwaiter().GetResult()
                        [void]$topologyProcess.TessaraStderrTask.GetAwaiter().GetResult()
                        [pscustomobject][ordered]@{
                            stdout = Write-TessaraLifecycleNewText -EvidenceRoot $evidence `
                                -Path (Join-Path $attemptRoot "topology.stdout.log") `
                                -Text (Protect-TessaraLifecycleText `
                                    -Text ([string]$topologyProcess.TessaraStdoutCapture.GetUtf8Text()) `
                                    -SensitiveValues @($environmentPlan.redactions))
                            stderr = Write-TessaraLifecycleNewText -EvidenceRoot $evidence `
                                -Path (Join-Path $attemptRoot "topology.stderr.log") `
                                -Text (Protect-TessaraLifecycleText `
                                    -Text ([string]$topologyProcess.TessaraStderrCapture.GetUtf8Text()) `
                                    -SensitiveValues @($environmentPlan.redactions))
                            output_limit_bytes = $script:MaximumCapturedStreamBytes
                            stdout_overflow = [bool]$topologyProcess.TessaraStdoutCapture.Overflowed
                            stderr_overflow = [bool]$topologyProcess.TessaraStderrCapture.Overflowed
                        }
                    }
                    if ($null -ne $topologyProcess -and
                        ($topologyProcess.TessaraStdoutCapture.Overflowed -or
                            $topologyProcess.TessaraStderrCapture.Overflowed)) {
                        throw "Topology process exceeded the bounded output limit."
                    }
                    $cleanup = [pscustomobject][ordered]@{
                        state = "passed"
                        mode = "local-process-tree-destroyed"
                        logs = $topologyLogs
                    }
                } elseif ([string]$topology.provider -ceq "docker-compose") {
                    $cleanup = Remove-TessaraLifecycleComposeTopology -Topology $topology `
                        -Context $context -Environment $processEnvironment `
                        -DockerCommand $DockerCommand
                }
                $topologyOwned = $false
            } elseif ($topologyOwned -and [string]$topology.on_success -ceq "handoff") {
                if ([string]::IsNullOrWhiteSpace([string]$topology.handoff_to)) {
                    throw "Topology handoff has no target lane."
                }
                $currentStage = "finalization"
                try {
                    $handoff = [pscustomobject][ordered]@{
                        schema_version = 1
                        contract = "tessara.validation.topology-receipt"
                        state = "retained"
                        provider = [string]$topology.provider
                        source_attempt_id = $attemptId
                        from_lane = [string]$Lane.id
                        handoff_to = [string]$topology.handoff_to
                        compose_project = [string]$context.compose_project
                        topology_lease = $topologyLease
                        topology_lease_sha256 = Get-TessaraLifecycleSha256Text -Text $topologyLease
                        readiness_capability = $readinessCapability
                        evidence_root_fingerprint = $evidenceRootFingerprint
                        ports = [pscustomobject]$ports
                        compose_file_sha256 = $composeFileSha256
                        compose_configuration_sha256 = $composeConfigurationSha256
                        compose_runtime_identity_sha256 = $composeRuntimeIdentitySha256
                        docker_provider_fingerprint = [string]$dockerProviderIdentity.fingerprint
                        candidate_fingerprint = $CandidateFingerprint
                        source_acceptance_fingerprint = $AcceptanceFingerprint
                        source_adapter_fingerprint = $AdapterFingerprint
                        source_compatibility_fingerprint = [string]$LaneIdentity.compatibility_fingerprint
                        target_acceptance_fingerprint = [string]$targetLaneIdentity.acceptance_fingerprint
                        target_adapter_fingerprint = [string]$targetLaneIdentity.adapter_fingerprint
                        target_compatibility_fingerprint = [string]$targetLaneIdentity.compatibility_fingerprint
                        platform_execution_fingerprint = $PlatformExecutionFingerprint
                        aggregate_acceptance_fingerprint = $AggregateAcceptanceFingerprint
                        aggregate_adapter_fingerprint = $AggregateAdapterFingerprint
                        aggregate_platform_fingerprint = [string]$PlatformIdentity.platform_fingerprint
                    }
                    $handoffRef = Write-TessaraLifecycleNewJson -EvidenceRoot $evidence `
                        -Path (Join-Path $attemptRoot "topology-receipt.json") `
                        -Document $handoff
                    $cleanup = [pscustomobject][ordered]@{
                        state = "passed"
                        mode = "retained-for-handoff"
                        receipt = $handoffRef
                    }
                    $handoffPending = $true
                } catch {
                    $publicationFailure = $_
                    try {
                        $null = Remove-TessaraLifecycleComposeTopology -Topology $topology `
                            -Context $context -Environment $processEnvironment `
                            -DockerCommand $DockerCommand
                        $cleanup = [pscustomobject][ordered]@{
                            state = "failed"
                            mode = "handoff-publication-failed-topology-destroyed"
                            message = $publicationFailure.Exception.Message
                        }
                        $topologyOwned = $false
                    } catch {
                        $cleanup = [pscustomobject][ordered]@{
                            state = "failed"
                            mode = "handoff-publication-and-cleanup-failed"
                            message = "$($publicationFailure.Exception.Message) Cleanup: $($_.Exception.Message)"
                        }
                    }
                    throw $publicationFailure
                }
            }
        } catch {
            $cleanupFailure = $_
            if ([string]$cleanup.state -cne "failed") {
                $cleanup = [pscustomobject][ordered]@{
                    state = "failed"
                    mode = "cleanup-failed"
                    message = $_.Exception.Message
                }
            }
            if ($null -eq $failure) {
                $failure = $cleanupFailure
                $failureStage = if ($currentStage -ceq "finalization") {
                    "finalization"
                } else { "cleanup" }
                $state = "failed"
            }
        } finally {
            # The platform never writes lane bindings into the caller process.
            # Restoring a snapshot here would overwrite legitimate concurrent
            # runspace changes that the lane neither consumed nor caused.
            $restoration = [pscustomobject][ordered]@{
                state = "passed"
                mode = "child-process-isolation-no-caller-write"
            }
            if ($null -ne $topologyProcess) {
                if ($topologyProcess.PSObject.Properties.Name -contains "TessaraStdoutCapture") {
                    $topologyProcess.TessaraStdoutCapture.Dispose()
                    $topologyProcess.TessaraStderrCapture.Dispose()
                }
                $topologyProcess.Dispose()
            }
        }
    }

    try {
        Assert-TessaraLifecycleLaneInputsUnchanged -LaneIdentity $LaneIdentity `
            -RepositoryRoot $repository -PlatformIdentity $PlatformIdentity
    } catch {
        $integrityFailureMessage = Protect-TessaraLifecycleText `
            -Text ([string]$_.Exception.Message) `
            -SensitiveValues @(if ($null -eq $environmentPlan) { @() } else {
                @($environmentPlan.redactions)
            })
        if ($null -eq $failure) {
            $failure = $_
            $failureStage = "source-integrity"
        }
        $state = "failed"
    }
    if ($state -ceq "passed") {
        try {
            Assert-TessaraLifecycleNotCancelled -CancellationPath $CancellationPath `
                -Boundary "pre-checkpoint publication"
        } catch [OperationCanceledException] {
            $state = "interrupted"
            $failure = $_
            $failureStage = "finalization"
        }
    }
    if ($handoffPending -and $topologyOwned -and $state -cne "passed") {
        try {
            $null = Remove-TessaraLifecycleComposeTopology -Topology $topology `
                -Context $context -Environment $processEnvironment `
                -DockerCommand $DockerCommand
            $cleanup = [pscustomobject][ordered]@{
                state = "passed"
                mode = "handoff-invalidated-topology-destroyed"
                receipt = $handoffRef
            }
            $topologyOwned = $false
            $handoffPending = $false
        } catch {
            $cleanup = [pscustomobject][ordered]@{
                state = "failed"
                mode = "handoff-invalidation-cleanup-failed"
                receipt = $handoffRef
                message = $_.Exception.Message
            }
        }
    }
    try {
    $cleanup | Add-Member -NotePropertyName environment_restoration `
        -NotePropertyValue $restoration
    if ($cleanup.PSObject.Properties.Name -contains "message") {
        $cleanup.message = Protect-TessaraLifecycleText -Text ([string]$cleanup.message) `
            -SensitiveValues @(if ($null -eq $environmentPlan) { @() } else {
                @($environmentPlan.redactions)
            })
    }
    $safeFailureMessage = if ($null -eq $failure) { $null } else {
        Protect-TessaraLifecycleText -Text (([string]$failure.Exception.Message) +
            "`n" + ([string]$failure.ScriptStackTrace)) `
            -SensitiveValues @(if ($null -eq $environmentPlan) { @() } else {
                @($environmentPlan.redactions)
            })
    }
    $rawExecutionIdentity = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.raw-execution-identity"
        adapter_id = [string]$Adapter.adapter_id
        lane_id = [string]$Lane.id
        attempt_id = $attemptId
        evidence_root_sha256 = $evidenceRootFingerprint
        candidate_fingerprint = $CandidateFingerprint
        candidate_source_identity = $LaneIdentity.candidate_source_identity
        lane_compatibility_fingerprint = [string]$LaneIdentity.compatibility_fingerprint
        compatibility_plan_fingerprint = [string]$LaneIdentity.compatibility_plan_fingerprint
        environment_fingerprint = if ($null -eq $environmentPlan) { $null } else {
            [string]$environmentPlan.fingerprint
        }
        tool_fingerprint = $toolFingerprint
        tool_observation_fingerprint = $toolObservationFingerprint
        prerequisite_result_hashes = @($prerequisiteResults | ForEach-Object {
            [string]$_.result.sha256
        } | Sort-Object)
        compose_project = if ([string]$topology.provider -ceq "docker-compose") {
            [string]$context.compose_project
        } else { $null }
        compose_file_sha256 = $composeFileSha256
        compose_configuration_sha256 = $composeConfigurationSha256
        compose_runtime_identity_sha256 = $composeRuntimeIdentitySha256
        docker_provider_fingerprint = if ($null -eq $dockerProviderIdentity) {
            $null
        } else { [string]$dockerProviderIdentity.fingerprint }
        topology_lease_sha256 = if ($null -eq $topologyLease) { $null } else {
            Get-TessaraLifecycleSha256Text -Text $topologyLease
        }
        ports = [pscustomobject]$ports
    }
    $rawExecutionFingerprint = Get-TessaraLifecycleSha256Text -Text (
        ((ConvertTo-TessaraLifecycleCanonicalJsonValue -Value $rawExecutionIdentity) |
            ConvertTo-Json -Depth 100 -Compress) + "`n"
    )
    $document = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.lane-result"
        state = $state
        authoritative = $false
        diagnostic = $false
        adapter_id = [string]$Adapter.adapter_id
        lane_id = [string]$Lane.id
        phase = [string]$LaneIdentity.phase
        attempt_id = $attemptId
        candidate_fingerprint = $CandidateFingerprint
        candidate_source_identity = $LaneIdentity.candidate_source_identity
        acceptance_fingerprint = $AcceptanceFingerprint
        adapter_fingerprint = $AdapterFingerprint
        aggregate_acceptance_fingerprint = $AggregateAcceptanceFingerprint
        aggregate_adapter_fingerprint = $AggregateAdapterFingerprint
        lane_compatibility_fingerprint = [string]$LaneIdentity.compatibility_fingerprint
        compatibility_plan_fingerprint = [string]$LaneIdentity.compatibility_plan_fingerprint
        platform_execution_fingerprint = $PlatformExecutionFingerprint
        evidence_root_fingerprint = $evidenceRootFingerprint
        platform = $PlatformIdentity
        contract_lane_fingerprint = [string]$LaneIdentity.contract_lane_fingerprint
        dependency_fingerprints = @($LaneIdentity.dependency_fingerprints)
        fixture_fingerprint = [string]$LaneIdentity.fixture_fingerprint
        harness_fingerprint = [string]$LaneIdentity.harness_fingerprint
        environment_contract_fingerprint = [string]$LaneIdentity.environment_contract_fingerprint
        environment_fingerprint = if ($null -eq $environmentPlan) { $null } else { [string]$environmentPlan.fingerprint }
        tool_fingerprint = $toolFingerprint
        tool_observation_fingerprint = $toolObservationFingerprint
        raw_execution_fingerprint = $rawExecutionFingerprint
        immutable_raw_results_complete = $true
        assertions_started = $assertionsStarted
        assertions_completed = $assertionActionCount -gt 0 -and $assertionActionsCompleted -eq $assertionActionCount
        failure_stage = $failureStage
        failed_action_id = if ($null -eq $failure) { $null } else { $currentActionId }
        compose_project = if ([string]$topology.provider -ceq "docker-compose") { [string]$context.compose_project } else { $null }
        compose_file_sha256 = $composeFileSha256
        compose_configuration_sha256 = $composeConfigurationSha256
        compose_runtime_identity_sha256 = $composeRuntimeIdentitySha256
        docker_provider_fingerprint = if ($null -eq $dockerProviderIdentity) {
            $null
        } else { [string]$dockerProviderIdentity.fingerprint }
        topology_lease_sha256 = if ($null -eq $topologyLease) { $null } else {
            Get-TessaraLifecycleSha256Text -Text $topologyLease
        }
        ports = [pscustomobject]$ports
        topology_claim = $topologyClaimRef
        prerequisite_results = @($prerequisiteResults)
        actions = @($actionResults)
        cleanup_restoration = $cleanup
        failure = if ($null -eq $failure) { $null } else { [pscustomobject][ordered]@{
            message = $safeFailureMessage
            category = [string]$failure.CategoryInfo.Category
            stage = $failureStage
            assertions_started = $assertionsStarted
        } }
        source_integrity_failure = $integrityFailureMessage
    }
    $artifactManifest = @(
        Get-ChildItem -LiteralPath $attemptRoot -Recurse -File -Force |
            Where-Object { $_.Name -notlike "*.sha256" } |
            Sort-Object FullName |
            ForEach-Object { [pscustomobject][ordered]@{
                path = [IO.Path]::GetRelativePath($attemptRoot, $_.FullName).Replace('\', '/')
                sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
            } }
    )
    $checkpointDocument = [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.validation.execution-checkpoint"
        state = "execution_complete"
        result = $document
        artifacts = @($artifactManifest)
    }
    $checkpointRef = Write-TessaraLifecycleNewJson -EvidenceRoot $evidence `
        -Path (Join-Path $attemptRoot "execution-complete.json") -Document $checkpointDocument
    try {
        Assert-TessaraLifecycleLaneInputsUnchanged -LaneIdentity $LaneIdentity `
            -RepositoryRoot $repository -PlatformIdentity $PlatformIdentity
    } catch {
        $inputIntegrityFailure = $_
        $revocationCleanup = $null
        if ($handoffPending -and $topologyOwned) {
            try {
                $null = Remove-TessaraLifecycleComposeTopology -Topology $topology `
                    -Context $context -Environment $processEnvironment `
                    -DockerCommand $DockerCommand
                $topologyOwned = $false
                $handoffPending = $false
            } catch {
                $revocationCleanup = $_.Exception.Message
            }
        }
        $revocationPublicationFailure = $null
        try {
            $null = Write-TessaraLifecycleExecutionRevocation -AttemptRoot $attemptRoot `
                -EvidenceRoot $evidence `
                -Reason "source-integrity-changed-after-checkpoint" `
                -CertificationFault $CertificationFault
        } catch {
            $revocationPublicationFailure = $_.Exception.Message
        }
        $cleanupSuffix = if ($null -eq $revocationCleanup) { "" } else {
            " Cleanup failed: $revocationCleanup"
        }
        $revocationSuffix = if ($null -eq $revocationPublicationFailure) { "" } else {
            " Revocation publication failed: $revocationPublicationFailure"
        }
        $lateMessage = "Lane inputs changed after checkpoint creation; the checkpoint was revoked and the lane must rerun. $($inputIntegrityFailure.Exception.Message)$cleanupSuffix$revocationSuffix"
        throw (Protect-TessaraLifecycleText -Text $lateMessage `
            -SensitiveValues @($environmentPlan.redactions))
    }
    if ($null -ne $integrityFailureMessage) {
        $revocationPublicationFailure = $null
        try {
            $null = Write-TessaraLifecycleExecutionRevocation -AttemptRoot $attemptRoot `
                -EvidenceRoot $evidence `
                -Reason "source-integrity-recorded-before-checkpoint" `
                -CertificationFault $CertificationFault
        } catch {
            $revocationPublicationFailure = $_.Exception.Message
        }
        $revocationSuffix = if ($null -eq $revocationPublicationFailure) { "" } else {
            " Revocation publication failed: $revocationPublicationFailure"
        }
        $lateMessage = "Execution recorded a source-integrity failure, so its checkpoint is ineligible for finalization and the lane must rerun. $integrityFailureMessage$revocationSuffix"
        throw (Protect-TessaraLifecycleText -Text $lateMessage `
            -SensitiveValues @($environmentPlan.redactions))
    }
    $integrityCommitComplete = $false
    $integrityCommitPath = Join-Path $attemptRoot "execution-integrity-verified.json"
    try {
        $null = Ensure-TessaraLifecycleExecutionIntegrityCommit `
            -AttemptRoot $attemptRoot -EvidenceRoot $evidence `
            -CheckpointReference $checkpointRef -ExpectedLaneId ([string]$Lane.id) `
            -ExpectedAttemptId $attemptId -CandidateFingerprint $CandidateFingerprint `
            -LaneIdentity $LaneIdentity `
            -ExecutionPlatformFingerprint $PlatformExecutionFingerprint `
            -EvidenceRootFingerprint $evidenceRootFingerprint -AllowCreate
        $integrityCommitComplete = $true
        $finalized = Complete-TessaraValidationEvidence `
            -CheckpointPath $checkpointRef.path -EvidenceRoot $evidence `
            -LaneId ([string]$Lane.id) -CandidateFingerprint $CandidateFingerprint `
            -LaneIdentity $LaneIdentity -ExecutionPlatformFingerprint $PlatformExecutionFingerprint `
            -FinalizationPlatformFingerprint ([string]$PlatformIdentity.finalization_fingerprint)
        if ($CertificationFault -ceq "after-handoff-finalization") {
            throw "Injected failure after authoritative handoff finalization."
        }
        if ($handoffPending) {
            # Complete published the authoritative handoff commit. Drop local
            # cleanup authority inside the same guarded region before any
            # later fault can enter the outer pre-transfer teardown path.
            $topologyOwned = $false
            $handoffPending = $false
        }
    } catch {
        $finalizationFailure = $_
        if ($handoffPending -and $topologyOwned) {
            $integrityCommitMayExist = $integrityCommitComplete -or
                (Test-Path -LiteralPath $integrityCommitPath) -or
                (Test-Path -LiteralPath "$integrityCommitPath.sha256")
            $revocationPublicationFailure = $null
            try {
                $null = Write-TessaraLifecycleExecutionRevocation -AttemptRoot $attemptRoot `
                    -EvidenceRoot $evidence `
                    -Reason "handoff-finalization-failed" `
                    -CertificationFault $CertificationFault
            } catch {
                $revocationPublicationFailure = $_.Exception.Message
            }
            $handoffCleanupFailure = $null
            if ($null -eq $revocationPublicationFailure -or
                -not $integrityCommitMayExist) {
                try {
                    $null = Remove-TessaraLifecycleComposeTopology -Topology $topology `
                        -Context $context -Environment $processEnvironment `
                        -DockerCommand $DockerCommand
                    $topologyOwned = $false
                    $handoffPending = $false
                } catch {
                    $handoffCleanupFailure = $_.Exception.Message
                }
            } else {
                # A still-eligible retained-topology checkpoint must never describe
                # a topology that this failure path already destroyed.
                $suppressOuterHandoffCleanup = $true
            }
            $cleanupSuffix = if ($null -eq $handoffCleanupFailure) { "" } else {
                " Cleanup failed: $handoffCleanupFailure"
            }
            $revocationSuffix = if ($null -eq $revocationPublicationFailure) { "" } else {
                " Revocation publication failed: $revocationPublicationFailure"
            }
            $retentionSuffix = if ($suppressOuterHandoffCleanup) {
                " The retained topology was left intact because revocation could not commit."
            } else { "" }
            $lateMessage = "Handoff evidence finalization failed; authoritative transfer did not occur. $($finalizationFailure.Exception.Message)$cleanupSuffix$revocationSuffix$retentionSuffix"
            throw (Protect-TessaraLifecycleText -Text $lateMessage `
                -SensitiveValues @($environmentPlan.redactions))
        }
        if (-not $integrityCommitComplete) {
            $lateMessage = "Lane execution completed, but its positive integrity commit could not be published. The checkpoint is ineligible and the lane must rerun. $($finalizationFailure.Exception.Message)"
            throw (Protect-TessaraLifecycleText -Text $lateMessage `
                -SensitiveValues @($environmentPlan.redactions))
        }
        $lateMessage = "Lane execution completed, but evidence finalization failed. Reuse checkpoint '$($checkpointRef.path)' without rerunning actions. $($finalizationFailure.Exception.Message)"
        throw (Protect-TessaraLifecycleText -Text $lateMessage `
            -SensitiveValues @($environmentPlan.redactions))
    }
    if ($state -cne "passed") {
        $cleanupSuffix = if ([string]$cleanup.state -ceq "failed") {
            " Cleanup failed: $([string]$cleanup.message)"
        } else { "" }
        throw "Validation lane '$($Lane.id)' finished '$state'; evidence: $($finalized.evidence_path). $safeFailureMessage$cleanupSuffix"
    }
    $finalized
    } catch {
        $publicationFailure = $_
        if ($handoffPending -and $topologyOwned -and -not $suppressOuterHandoffCleanup) {
            $lateCleanupFailure = $null
            try {
                $null = Remove-TessaraLifecycleComposeTopology -Topology $topology `
                    -Context $context -Environment $processEnvironment `
                    -DockerCommand $DockerCommand
                $topologyOwned = $false
                $handoffPending = $false
            } catch {
                $lateCleanupFailure = $_.Exception.Message
            }
            $cleanupSuffix = if ($null -eq $lateCleanupFailure) { "" } else {
                " Cleanup failed: $lateCleanupFailure"
            }
            $lateMessage = "Retained-topology evidence publication failed before ownership transfer. $($publicationFailure.Exception.Message)$cleanupSuffix"
            throw (Protect-TessaraLifecycleText -Text $lateMessage `
                -SensitiveValues @(if ($null -eq $environmentPlan) { @() } else {
                    @($environmentPlan.redactions)
                }))
        }
        throw
    }
}

Export-ModuleMember -Function @(
    "Get-TessaraValidationLifecycleIdentity",
    "Invoke-TessaraValidationLifecycleLane"
)

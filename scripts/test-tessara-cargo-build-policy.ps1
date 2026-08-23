[CmdletBinding()]
param([switch]$SelfTest)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not $SelfTest) {
    throw "This is a policy self-test. Invoke it with -SelfTest."
}

$modulePath = [IO.Path]::GetFullPath(
    (Join-Path $PSScriptRoot "tessara-cargo-build-policy.psm1")
)
$module = @(
    Get-Module |
        Where-Object {
            -not [string]::IsNullOrWhiteSpace($_.Path) -and
            [IO.Path]::GetFullPath($_.Path) -eq $modulePath
        }
) | Select-Object -First 1
if ($null -eq $module) {
    $module = Import-Module $modulePath -PassThru
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
    throw "Expected rejection containing '$ExpectedMessage'."
}

function Copy-PolicyState {
    param([Parameter(Mandatory)][psobject]$State)

    $copy = [pscustomobject][ordered]@{}
    foreach ($property in $State.PSObject.Properties) {
        $copy | Add-Member `
            -NotePropertyName $property.Name `
            -NotePropertyValue $property.Value
    }
    return $copy
}

function Remove-SyntheticRoot {
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }
    $resolvedPath = [IO.Path]::GetFullPath($Path)
    $resolvedTemp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    $relativePath = [IO.Path]::GetRelativePath($resolvedTemp, $resolvedPath)
    $parentPrefix = "..$([IO.Path]::DirectorySeparatorChar)"
    if ($relativePath -eq "." -or
        $relativePath -eq ".." -or
        [IO.Path]::IsPathRooted($relativePath) -or
        $relativePath.StartsWith($parentPrefix, [StringComparison]::Ordinal)) {
        throw "Refusing to remove synthetic Cargo policy root outside the system temp directory: $resolvedPath"
    }
    $item = Get-Item -LiteralPath $resolvedPath -Force
    if (-not $item.PSIsContainer -or
        ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Refusing to remove a synthetic Cargo policy root that is not an ordinary directory: $resolvedPath"
    }
    Remove-Item -LiteralPath $resolvedPath -Recurse -Force
}

function Set-FreeSpaceProbe {
    param([Parameter(Mandatory)][scriptblock]$Probe)

    & $module {
        param([scriptblock]$Replacement)
        $script:CargoFreeSpaceProbe = $Replacement
    } $Probe
}

function Reset-FreeSpaceProbe {
    & $module {
        $script:CargoFreeSpaceProbe = $script:DefaultCargoFreeSpaceProbe
    }
}

$temporaryRoot = Join-Path ([IO.Path]::GetTempPath()) (
    "tessara-cargo-build-policy-selftest-$([guid]::NewGuid().ToString('N'))"
)
$syntheticRepository = Join-Path $temporaryRoot "synthetic-repository"
$syntheticSource = Join-Path $syntheticRepository "src"
$validationTargetRoot = Join-Path $temporaryRoot "validation-targets"
$diagnosticTargetRoot = Join-Path $temporaryRoot "diagnostic-targets"
$previousTarget = $env:CARGO_TARGET_DIR
$previousIncremental = $env:CARGO_INCREMENTAL
$previousTestDebug = $env:CARGO_PROFILE_TEST_DEBUG
$validationState = $null
$failureState = $null
$diagnosticState = $null
$junctionPath = $null

try {
    New-Item -ItemType Directory -Path $syntheticSource -Force | Out-Null
    Set-Content `
        -LiteralPath (Join-Path $syntheticRepository "Cargo.toml") `
        -Value @(
            "[package]",
            'name = "tessara-cargo-policy-selftest"',
            'version = "0.0.0"',
            'edition = "2024"'
        )
    Set-Content `
        -LiteralPath (Join-Path $syntheticSource "lib.rs") `
        -Value "pub fn marker() {}"

    $identity = Get-TessaraCargoBuildPolicyIdentity
    Assert-Equal $identity.schema_version 1 "Policy identity schema"
    Assert-Equal `
        $identity.contract `
        "tessara.validation.cargo-build-policy" `
        "Policy identity contract"
    Assert-Equal $identity.release_version "1.0.0" "Policy release"
    Assert-Equal `
        $identity.module_sha256 `
        (Get-FileHash -Algorithm SHA256 -LiteralPath $modulePath).Hash.ToLowerInvariant() `
        "Policy module fingerprint"

    $development = Get-TessaraCargoBuildPolicy -Mode Development
    Assert-Equal $development.incremental $true "Development incremental policy"
    Assert-Equal $development.automatic_cleanup $false "Development cleanup policy"

    $validation = Get-TessaraCargoBuildPolicy -Mode Validation
    Assert-Equal $validation.incremental $false "Validation incremental policy"
    Assert-Equal $validation.test_debug "0" "Validation test debug policy"
    Assert-Equal $validation.automatic_cleanup $true "Validation cleanup policy"
    Assert-Equal `
        $validation.policy_fingerprint `
        $identity.module_sha256 `
        "Validation policy fingerprint"

    $diagnostic = Get-TessaraCargoBuildPolicy -Mode Diagnostic
    Assert-Equal $diagnostic.test_debug "1" "Diagnostic test debug policy"
    Assert-Equal $diagnostic.automatic_cleanup $false "Diagnostic cleanup policy"

    Assert-Rejected `
        -Action {
            Enter-TessaraCargoBuildPolicy `
                -Lane "self-test" `
                -RepositoryRoot $syntheticRepository `
                -Mode Validation `
                -RetainTarget
        } `
        -ExpectedMessage "only in explicit Diagnostic mode"

    Assert-Rejected `
        -Action {
            Enter-TessaraCargoBuildPolicy `
                -Lane "self-test" `
                -RepositoryRoot $syntheticRepository `
                -Mode Diagnostic
        } `
        -ExpectedMessage "requires explicit target retention"

    Set-FreeSpaceProbe -Probe { param([string]$Path) return 1GB }
    Assert-Rejected `
        -Action {
            Enter-TessaraCargoBuildPolicy `
                -Lane "self-test" `
                -RepositoryRoot $syntheticRepository `
                -Mode Validation `
                -TargetRoot $validationTargetRoot `
                -MinimumFreeSpaceGB 2
        } `
        -ExpectedMessage "requires at least 2 GB free"
    Reset-FreeSpaceProbe

    if ([Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
            [Runtime.InteropServices.OSPlatform]::Windows
        )) {
        $junctionTarget = Join-Path $temporaryRoot "junction-target"
        $junctionPath = Join-Path $temporaryRoot "junction-root"
        $junctionDescendant = Join-Path $junctionPath "ordinary-target-root"
        New-Item -ItemType Directory -Path $junctionTarget | Out-Null
        New-Item -ItemType Junction -Path $junctionPath -Target $junctionTarget | Out-Null
        Assert-Rejected `
            -Action {
                Enter-TessaraCargoBuildPolicy `
                    -Lane "self-test" `
                    -RepositoryRoot $syntheticRepository `
                    -Mode Validation `
                    -TargetRoot $junctionDescendant `
                    -MinimumFreeSpaceGB 0
            } `
            -ExpectedMessage "traverses a reparse point"
        Remove-Item -LiteralPath $junctionPath -Force
        $junctionPath = $null
    }

    $sentinelTarget = Join-Path $temporaryRoot "sentinel-target"
    $env:CARGO_TARGET_DIR = $sentinelTarget
    $env:CARGO_INCREMENTAL = "1"
    $env:CARGO_PROFILE_TEST_DEBUG = "2"

    $validationState = Enter-TessaraCargoBuildPolicy `
        -Lane "self-test" `
        -RepositoryRoot $syntheticRepository `
        -Mode Validation `
        -TargetRoot $validationTargetRoot `
        -MinimumFreeSpaceGB 0

    Assert-Equal `
        $env:CARGO_TARGET_DIR `
        $validationState.target_directory `
        "Isolated target"
    Assert-Equal $env:CARGO_INCREMENTAL "0" "Incremental environment"
    Assert-Equal $env:CARGO_PROFILE_TEST_DEBUG "0" "Test debug environment"
    Assert-Equal `
        (Test-Path -LiteralPath (
                Join-Path $validationState.target_directory ".tessara-cargo-target.json"
            )) `
        $true `
        "Leased target marker"
    Assert-Rejected `
        -Action {
            Enter-TessaraCargoBuildPolicy `
                -Lane "nested" `
                -RepositoryRoot $syntheticRepository `
                -Mode Validation `
                -TargetRoot $validationTargetRoot `
                -MinimumFreeSpaceGB 0
        } `
        -ExpectedMessage "lease is already active"

    $unrelatedDirectory = Join-Path $validationTargetRoot "unrelated-directory"
    New-Item -ItemType Directory -Path $unrelatedDirectory | Out-Null
    Set-Content `
        -LiteralPath (Join-Path $unrelatedDirectory "must-survive.txt") `
        -Value "preserved"
    $forgedState = Copy-PolicyState -State $validationState
    $forgedState.target_directory = $unrelatedDirectory
    Assert-Rejected `
        -Action { Exit-TessaraCargoBuildPolicy -State $forgedState } `
        -ExpectedMessage "does not match its active lease"
    Assert-Equal `
        (Test-Path -LiteralPath (Join-Path $unrelatedDirectory "must-survive.txt")) `
        $true `
        "Forged in-root target preservation"
    Assert-Equal $env:CARGO_TARGET_DIR $sentinelTarget "Rejected-state target restoration"

    $markerPath = Join-Path `
        $validationState.target_directory `
        ".tessara-cargo-target.json"
    $markerBytes = [IO.File]::ReadAllBytes($markerPath)
    Set-Content -LiteralPath $markerPath -Value '{"tampered":true}'
    Assert-Rejected `
        -Action { Exit-TessaraCargoBuildPolicy -State $validationState } `
        -ExpectedMessage "marker digest does not match"
    [IO.File]::WriteAllBytes($markerPath, $markerBytes)

    $ephemeralTarget = $validationState.target_directory
    Set-Content `
        -LiteralPath (Join-Path $ephemeralTarget "policy-marker.txt") `
        -Value "ephemeral"
    $cleanupInvokerCapture = [pscustomobject]@{ count = 0; arguments = @() }
    Exit-TessaraCargoBuildPolicy -State $validationState -CargoInvoker {
        param([string[]]$Arguments)
        $cleanupInvokerCapture.count++
        $cleanupInvokerCapture.arguments = @($Arguments)
        & ([string]$validationState.cargo_executable_path) @Arguments
    }
    Assert-Equal $cleanupInvokerCapture.count 1 "Authenticated Cargo cleanup invocation count"
    Assert-Equal ([string]$cleanupInvokerCapture.arguments[0]) "clean" `
        "Authenticated Cargo cleanup command"
    $completedValidationState = $validationState
    $validationState = $null

    Assert-Equal (Test-Path -LiteralPath $ephemeralTarget) $false "Ephemeral target cleanup"
    Assert-Equal $env:CARGO_TARGET_DIR $sentinelTarget "Target restoration"
    Assert-Equal $env:CARGO_INCREMENTAL "1" "Incremental restoration"
    Assert-Equal $env:CARGO_PROFILE_TEST_DEBUG "2" "Test debug restoration"
    Assert-Rejected `
        -Action { Exit-TessaraCargoBuildPolicy -State $completedValidationState } `
        -ExpectedMessage "does not identify an active policy lease"

    $failureState = Enter-TessaraCargoBuildPolicy `
        -Lane "failure-finally" `
        -RepositoryRoot $syntheticRepository `
        -Mode Validation `
        -TargetRoot $validationTargetRoot `
        -MinimumFreeSpaceGB 0
    $failureTarget = $failureState.target_directory
    try {
        throw "synthetic validation failure"
    } catch {
        Assert-Equal $_.Exception.Message "synthetic validation failure" "Synthetic failure"
    } finally {
        Exit-TessaraCargoBuildPolicy -State $failureState
        $failureState = $null
    }
    Assert-Equal `
        (Test-Path -LiteralPath $failureTarget) `
        $false `
        "Failed-lane finally cleanup"

    $diagnosticState = Enter-TessaraCargoBuildPolicy `
        -Lane "diagnostic-retained" `
        -RepositoryRoot $syntheticRepository `
        -Mode Diagnostic `
        -TargetRoot $diagnosticTargetRoot `
        -MinimumFreeSpaceGB 0 `
        -RetainTarget
    $diagnosticTarget = $diagnosticState.target_directory
    Exit-TessaraCargoBuildPolicy -State $diagnosticState
    $diagnosticState = $null
    Assert-Equal `
        (Test-Path -LiteralPath $diagnosticTarget) `
        $true `
        "Explicit diagnostic retention"
    Assert-Equal $env:CARGO_TARGET_DIR $sentinelTarget "Diagnostic target restoration"
    & cargo clean `
        --manifest-path (Join-Path $syntheticRepository "Cargo.toml") `
        --target-dir $diagnosticTarget
    if ($LASTEXITCODE -ne 0) {
        throw "Synthetic diagnostic cleanup failed with exit code $LASTEXITCODE."
    }
    Assert-Equal `
        (Test-Path -LiteralPath $diagnosticTarget) `
        $false `
        "Explicit diagnostic follow-up cleanup"
} finally {
    Reset-FreeSpaceProbe
    if ($null -ne $validationState) {
        try { Exit-TessaraCargoBuildPolicy -State $validationState } catch {}
    }
    if ($null -ne $failureState) {
        try { Exit-TessaraCargoBuildPolicy -State $failureState } catch {}
    }
    if ($null -ne $diagnosticState) {
        try { Exit-TessaraCargoBuildPolicy -State $diagnosticState } catch {}
    }
    $env:CARGO_TARGET_DIR = $previousTarget
    $env:CARGO_INCREMENTAL = $previousIncremental
    $env:CARGO_PROFILE_TEST_DEBUG = $previousTestDebug
    if ($null -ne $junctionPath -and (Test-Path -LiteralPath $junctionPath)) {
        Remove-Item -LiteralPath $junctionPath -Force
    }
    Remove-SyntheticRoot -Path $temporaryRoot
}

Write-Host (
    "Cargo build policy self-test passed: release={0}; fingerprint={1}" -f
        $identity.release_version,
        $identity.module_sha256
) -ForegroundColor Green

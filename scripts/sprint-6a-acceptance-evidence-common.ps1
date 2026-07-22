$script:Sprint6AAcceptanceEvidenceSchemaVersion = 1
$script:Sprint6AAcceptanceEvidenceBuiltInSeedSha256 = '2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600'
$script:Sprint6ADemoSeedRefusalMessage = 'Demo seed requires an empty database. Recreate the local database or run local launch with -FreshData before seeding.'
$script:Sprint6ASmokeAcceptanceChecks = @(
    'health_and_deployment_identity',
    'protected_server_rendered_shells',
    'module_inventory_policy_and_navigation',
    'role_authorization_boundaries',
    'seeded_product_and_visual_flows'
)
$script:Sprint6AUatAcceptanceChecks = @(
    'module_inventory_policy_and_navigation',
    'protected_server_rendered_routes',
    'organization_forms_and_dataset_flows',
    'component_execution_and_visuals',
    'dashboard_composition_and_exact_versions',
    'role_authorization_boundaries'
)
$script:Sprint6AUatComponentKinds = @('bar', 'donut', 'line', 'pie', 'stat_card', 'table')
$script:Sprint6AUatAuthorizationRoles = @('operator', 'respondent')

function Resolve-Sprint6AAcceptanceEvidencePath {
    param(
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$Path
    )

    if ([IO.Path]::IsPathRooted($Path)) {
        return [IO.Path]::GetFullPath($Path)
    }
    [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $Path))
}

function Get-Sprint6ASha256Text {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)

    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
        ([BitConverter]::ToString($algorithm.ComputeHash($bytes))).Replace('-', '').ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Get-Sprint6ASecurePathInfo {
    param(
        [Parameter(Mandatory)][string]$Path,
        [switch]$RequireLeaf,
        [switch]$RequireContainer,
        [switch]$AllowMissingLeaf
    )

    if ([string]::IsNullOrWhiteSpace($Path) -or $Path.IndexOf([char]0) -ge 0) {
        throw 'A non-empty filesystem path without NUL characters is required.'
    }
    $fullPath = [IO.Path]::GetFullPath($Path)
    if ($fullPath.StartsWith('\\?\', [StringComparison]::OrdinalIgnoreCase) -or
        $fullPath.StartsWith('\\.\', [StringComparison]::OrdinalIgnoreCase)) {
        throw "Device and extended-length paths are not allowed for Sprint 6A evidence: '$fullPath'."
    }
    $root = [IO.Path]::GetPathRoot($fullPath)
    if ([string]::IsNullOrWhiteSpace($root)) {
        throw "Could not resolve a filesystem root for '$fullPath'."
    }
    $remainder = $fullPath.Substring($root.Length)
    if ($remainder.Contains(':')) {
        throw "Alternate data streams are not allowed for Sprint 6A evidence paths: '$fullPath'."
    }

    $segments = @($remainder -split '[\\/]' | Where-Object { -not [string]::IsNullOrEmpty($_) })
    $current = $root
    $lastExistingItem = Get-Item -LiteralPath $root -Force -ErrorAction Stop
    for ($index = 0; $index -lt $segments.Count; $index++) {
        $current = Join-Path $current $segments[$index]
        if (-not (Test-Path -LiteralPath $current)) {
            break
        }
        $item = Get-Item -LiteralPath $current -Force -ErrorAction Stop
        $linkType = if ($null -eq $item.PSObject.Properties['LinkType']) { '' } else { [string]$item.LinkType }
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
            $linkType -in @('SymbolicLink', 'Junction')) {
            throw "Reparse points, junctions, and symbolic links are not allowed in Sprint 6A evidence paths: '$($item.FullName)'."
        }
        if ($linkType -eq 'HardLink') {
            throw "Hard-linked files are not allowed in Sprint 6A evidence paths: '$($item.FullName)'."
        }
        $lastExistingItem = $item
    }

    $exists = Test-Path -LiteralPath $fullPath
    if ($RequireLeaf -and -not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Required file '$fullPath' does not exist as a regular file."
    }
    if ($RequireContainer -and -not (Test-Path -LiteralPath $fullPath -PathType Container)) {
        throw "Required directory '$fullPath' does not exist as a directory."
    }
    if (-not $exists -and -not $AllowMissingLeaf) {
        throw "Required path '$fullPath' does not exist."
    }
    if (-not $exists -and $AllowMissingLeaf) {
        $parent = Split-Path -Parent $fullPath
        if (-not (Test-Path -LiteralPath $parent -PathType Container)) {
            throw "The evidence output parent must already exist as a safe directory: '$parent'."
        }
        $parentInfo = Get-Sprint6ASecurePathInfo -Path $parent -RequireContainer
        $physicalPath = Join-Path $parentInfo.physical_path (Split-Path -Leaf $fullPath)
    } else {
        $physicalPath = [string](Get-Item -LiteralPath $fullPath -Force -ErrorAction Stop).FullName
    }

    [pscustomobject][ordered]@{
        lexical_path = $fullPath
        physical_path = [IO.Path]::GetFullPath($physicalPath)
        exists = [bool]$exists
    }
}

function Assert-Sprint6APathSetsDistinct {
    param(
        [Parameter(Mandatory)][object[]]$Left,
        [Parameter(Mandatory)][object[]]$Right,
        [Parameter(Mandatory)][string]$Context
    )

    foreach ($leftPath in $Left) {
        foreach ($rightPath in $Right) {
            if ([string]::Equals([string]$leftPath.lexical_path, [string]$rightPath.lexical_path, [StringComparison]::OrdinalIgnoreCase) -or
                [string]::Equals([string]$leftPath.physical_path, [string]$rightPath.physical_path, [StringComparison]::OrdinalIgnoreCase)) {
                throw "$Context paths must be lexically and physically distinct: '$($leftPath.lexical_path)' aliases '$($rightPath.lexical_path)'."
            }
        }
    }
}

function Get-Sprint6AAcceptanceEvidencePathSet {
    param(
        [Parameter(Mandatory)][string]$EvidencePath,
        [switch]$RequireExisting
    )

    $fullPath = [IO.Path]::GetFullPath($EvidencePath)
    if ($RequireExisting) {
        $evidence = Get-Sprint6ASecurePathInfo -Path $fullPath -RequireLeaf
        $digest = Get-Sprint6ASecurePathInfo -Path "$fullPath.sha256" -RequireLeaf
    } else {
        $evidence = Get-Sprint6ASecurePathInfo -Path $fullPath -AllowMissingLeaf
        $digest = Get-Sprint6ASecurePathInfo -Path "$fullPath.sha256" -AllowMissingLeaf
        foreach ($candidate in @($evidence, $digest)) {
            if ($candidate.exists -and -not (Test-Path -LiteralPath $candidate.lexical_path -PathType Leaf)) {
                throw "Evidence output target '$($candidate.lexical_path)' exists but is not a regular file."
            }
        }
    }
    Assert-Sprint6APathSetsDistinct -Left @($evidence) -Right @($digest) -Context 'Evidence and SHA-256 sidecar'
    [pscustomobject][ordered]@{ evidence = $evidence; digest = $digest }
}

function Assert-Sprint6AAcceptanceEvidenceTargetAvailable {
    param(
        [Parameter(Mandatory)][string]$EvidencePath,
        [Parameter(Mandatory)][string]$DeploymentEvidencePath,
        [switch]$Overwrite
    )

    $acceptance = Get-Sprint6AAcceptanceEvidencePathSet -EvidencePath $EvidencePath
    $deployment = Get-Sprint6AAcceptanceEvidencePathSet -EvidencePath $DeploymentEvidencePath -RequireExisting
    Assert-Sprint6APathSetsDistinct `
        -Left @($acceptance.evidence, $acceptance.digest) `
        -Right @($deployment.evidence, $deployment.digest) `
        -Context 'Acceptance and deployment evidence'

    $existingPaths = @(
        @($acceptance.evidence, $acceptance.digest) |
            Where-Object { $_.exists } |
            ForEach-Object { $_.lexical_path }
    )
    if ($existingPaths.Count -gt 0 -and -not $Overwrite) {
        throw "Acceptance evidence publishing refuses to replace an existing transactional artifact pair without -OverwriteAcceptanceEvidence: $($existingPaths -join ', ')"
    }
    [pscustomobject][ordered]@{
        evidence_path = [string]$acceptance.evidence.lexical_path
        digest_path = [string]$acceptance.digest.lexical_path
        deployment_evidence_path = [string]$deployment.evidence.lexical_path
        deployment_digest_path = [string]$deployment.digest.lexical_path
    }
}

function Get-Sprint6ADeploymentEvidenceBinding {
    param([Parameter(Mandatory)][string]$DeploymentEvidencePath)

    $paths = Get-Sprint6AAcceptanceEvidencePathSet -EvidencePath $DeploymentEvidencePath -RequireExisting
    $expectedDigest = (Get-Content -LiteralPath $paths.digest.lexical_path -Raw).Trim()
    $actualDigest = (Get-FileHash -LiteralPath $paths.evidence.lexical_path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expectedDigest -cnotmatch '^[0-9a-f]{64}$' -or $expectedDigest -cne $actualDigest) {
        throw "Deployment evidence SHA-256 verification failed for '$($paths.evidence.lexical_path)'."
    }
    try {
        $raw = Get-Content -LiteralPath $paths.evidence.lexical_path -Raw
        $document = $raw | ConvertFrom-Json -DateKind String -NoEnumerate
    } catch {
        throw "Deployment evidence is not valid JSON: $($_.Exception.Message)"
    }
    [pscustomobject][ordered]@{
        path = [string]$paths.evidence.lexical_path
        digest_path = [string]$paths.digest.lexical_path
        sha256 = $actualDigest
        document = $document
    }
}

function Assert-Sprint6AExactProperties {
    param(
        [Parameter(Mandatory)][object]$Object,
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][string]$Context
    )

    if ($Object -isnot [pscustomobject]) {
        throw "$Context must be a JSON object/PSCustomObject."
    }
    $actual = @($Object.PSObject.Properties.Name)
    if (($actual -join "`0") -cne ($Expected -join "`0")) {
        throw "$Context fields must be exactly '$($Expected -join ',')'; found '$($actual -join ',')'."
    }
}

function Assert-Sprint6AExactString {
    param([object]$Value, [Parameter(Mandatory)][string]$Context)
    if ($Value -isnot [string]) { throw "$Context must be an exact JSON/CLR string." }
}

function Assert-Sprint6ADemoSeedRefusal {
    param(
        [Parameter(Mandatory)][int]$StatusCode,
        [Parameter(Mandatory)][AllowEmptyString()][string]$ResponseBody,
        [Parameter(Mandatory)][string]$Context
    )

    if ($StatusCode -ne 400) {
        throw "$Context failed with HTTP $StatusCode instead of the exact populated-database demo-seed refusal."
    }
    try {
        $errorDocument = $ResponseBody | ConvertFrom-Json -NoEnumerate
    } catch {
        throw "$Context returned a non-JSON HTTP 400 response: $($_.Exception.Message)"
    }
    Assert-Sprint6AExactProperties `
        -Object $errorDocument `
        -Expected @('code', 'message', 'error') `
        -Context "$Context refusal"
    foreach ($field in @('code', 'message', 'error')) {
        Assert-Sprint6AExactString -Value $errorDocument.$field -Context "$Context refusal.$field"
    }
    if ($errorDocument.code -cne 'bad_request' -or
        $errorDocument.message -cne $script:Sprint6ADemoSeedRefusalMessage -or
        $errorDocument.error -cne $script:Sprint6ADemoSeedRefusalMessage) {
        throw "$Context did not return the exact populated-database demo-seed refusal contract."
    }
}

function Assert-Sprint6ADemoSeedRefusalErrorRecord {
    param(
        [Parameter(Mandatory)][System.Management.Automation.ErrorRecord]$ErrorRecord,
        [Parameter(Mandatory)][string]$Context
    )

    $responseProperty = $ErrorRecord.Exception.PSObject.Properties['Response']
    $statusCode = 0
    if ($null -ne $responseProperty -and $null -ne $responseProperty.Value) {
        $statusProperty = $responseProperty.Value.PSObject.Properties['StatusCode']
        if ($null -ne $statusProperty -and $null -ne $statusProperty.Value) {
            $statusCode = [int]$statusProperty.Value
        }
    }
    Assert-Sprint6ADemoSeedRefusal `
        -StatusCode $statusCode `
        -ResponseBody ([string]$ErrorRecord.ErrorDetails.Message) `
        -Context $Context
}

function Test-Sprint6AShouldInvokeDemoSeed {
    param([AllowNull()][AllowEmptyString()][string]$ExpectedDataState)

    if (-not [string]::IsNullOrWhiteSpace($ExpectedDataState) -and
        $ExpectedDataState -cne 'fresh') {
        throw "Demo seed strategy received unsupported data state '$ExpectedDataState'."
    }
    $true
}

function Assert-Sprint6AExactInteger {
    param([object]$Value, [Parameter(Mandatory)][string]$Context)
    if ($null -eq $Value -or $Value.GetType().FullName -notin @(
        'System.SByte', 'System.Byte', 'System.Int16', 'System.UInt16',
        'System.Int32', 'System.UInt32', 'System.Int64', 'System.UInt64'
    )) { throw "$Context must be an exact JSON integral number." }
}

function Assert-Sprint6AExactStringArray {
    param(
        [object]$Value,
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][string]$Context
    )
    if ($Value -isnot [array]) { throw "$Context must be an exact JSON/CLR array." }
    $items = @($Value)
    foreach ($item in $items) { Assert-Sprint6AExactString -Value $item -Context "$Context item" }
    if (($items -join "`0") -cne ($Expected -join "`0")) {
        throw "$Context must be exactly '$($Expected -join ',')'."
    }
}

function Get-Sprint6AAcceptanceExpectedChecks {
    param([Parameter(Mandatory)][ValidateSet('smoke', 'uat')][string]$Kind)
    if ($Kind -eq 'smoke') { return @($script:Sprint6ASmokeAcceptanceChecks) }
    @($script:Sprint6AUatAcceptanceChecks)
}

function Assert-Sprint6AAcceptanceEvidenceDocument {
    param(
        [Parameter(Mandatory)][object]$Evidence,
        [Parameter(Mandatory)][string]$DeploymentEvidencePath,
        [Parameter(Mandatory)][string]$RunnerFilePath
    )

    Assert-Sprint6AExactProperties -Object $Evidence -Expected @(
        'schema_version', 'evidence_kind', 'status', 'completed_at_utc',
        'expected_data_state', 'base_url', 'runner', 'checks', 'deployment', 'result'
    ) -Context 'Acceptance evidence'
    Assert-Sprint6AExactInteger -Value $Evidence.schema_version -Context 'schema_version'
    foreach ($field in @('evidence_kind', 'status', 'completed_at_utc', 'expected_data_state', 'base_url')) {
        Assert-Sprint6AExactString -Value $Evidence.$field -Context $field
    }
    if ([int64]$Evidence.schema_version -ne $script:Sprint6AAcceptanceEvidenceSchemaVersion -or
        $Evidence.status -cne 'passed' -or
        $Evidence.expected_data_state -cne 'fresh') {
        throw 'Acceptance evidence has an invalid schema version, status, or data state.'
    }

    $kind = switch ($Evidence.evidence_kind) {
        'tessara.sprint-6a.smoke' { 'smoke' }
        'tessara.sprint-6a.uat' { 'uat' }
        default { throw "Unsupported acceptance evidence kind '$($Evidence.evidence_kind)'." }
    }
    $expectedRunner = if ($kind -eq 'smoke') { 'scripts/smoke.ps1' } else { 'scripts/uat-sprint.ps1' }
    $runnerInfo = Get-Sprint6ASecurePathInfo -Path $RunnerFilePath -RequireLeaf
    $normalizedRunner = $runnerInfo.physical_path.Replace('/', '\')
    $normalizedExpectedRunner = $expectedRunner.Replace('/', '\')
    if (-not $normalizedRunner.EndsWith("\$normalizedExpectedRunner", [StringComparison]::OrdinalIgnoreCase)) {
        throw "Runner file '$($runnerInfo.physical_path)' is not the exact $expectedRunner runner."
    }

    Assert-Sprint6AExactProperties -Object $Evidence.runner -Expected @('path', 'sha256') -Context 'runner'
    Assert-Sprint6AExactString -Value $Evidence.runner.path -Context 'runner.path'
    Assert-Sprint6AExactString -Value $Evidence.runner.sha256 -Context 'runner.sha256'
    $actualRunnerDigest = (Get-FileHash -LiteralPath $runnerInfo.lexical_path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Evidence.runner.path -cne $expectedRunner -or
        $Evidence.runner.sha256 -cne $actualRunnerDigest) {
        throw 'Acceptance evidence runner name or SHA-256 does not match the exact executing runner.'
    }
    Assert-Sprint6AExactStringArray `
        -Value $Evidence.checks `
        -Expected (Get-Sprint6AAcceptanceExpectedChecks -Kind $kind) `
        -Context 'checks'

    $timestamp = [DateTimeOffset]::MinValue
    if ($Evidence.completed_at_utc -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{7}Z$' -or
        -not [DateTimeOffset]::TryParseExact(
            $Evidence.completed_at_utc,
            'o',
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind,
            [ref]$timestamp
        ) -or $timestamp.Offset -ne [TimeSpan]::Zero) {
        throw 'completed_at_utc must be an exact round-trip UTC timestamp string.'
    }
    $baseUri = $null
    if (-not [uri]::TryCreate($Evidence.base_url, [UriKind]::Absolute, [ref]$baseUri) -or
        $baseUri.Scheme -notin @('http', 'https') -or -not $baseUri.IsLoopback -or
        -not [string]::IsNullOrEmpty($baseUri.UserInfo)) {
        throw 'base_url must be an absolute credential-free loopback HTTP(S) URL.'
    }

    Assert-Sprint6AExactProperties -Object $Evidence.deployment -Expected @(
        'evidence_sha256', 'data_state', 'source_commit', 'source_tree', 'image_id',
        'database_name', 'installation_id', 'catalog_entries', 'built_in_seed_sha256'
    ) -Context 'deployment'
    foreach ($field in @(
        'evidence_sha256', 'data_state', 'source_commit', 'source_tree', 'image_id',
        'database_name', 'installation_id', 'built_in_seed_sha256'
    )) { Assert-Sprint6AExactString -Value $Evidence.deployment.$field -Context "deployment.$field" }
    Assert-Sprint6AExactInteger -Value $Evidence.deployment.catalog_entries -Context 'deployment.catalog_entries'

    $deploymentBinding = Get-Sprint6ADeploymentEvidenceBinding -DeploymentEvidencePath $DeploymentEvidencePath
    $snapshot = $deploymentBinding.document.snapshot
    $installationId = [guid]::Empty
    if (-not [guid]::TryParse($Evidence.deployment.installation_id, [ref]$installationId) -or
        $installationId -eq [guid]::Empty -or
        $Evidence.deployment.installation_id -cne $installationId.ToString('D').ToLowerInvariant() -or
        $Evidence.deployment.evidence_sha256 -cne $deploymentBinding.sha256 -or
        $Evidence.deployment.data_state -cne $Evidence.expected_data_state -or
        $Evidence.deployment.data_state -cne [string]$snapshot.data.state -or
        $Evidence.base_url -cne [string]$snapshot.base_url -or
        $Evidence.deployment.source_commit -cne [string]$snapshot.source.commit -or
        $Evidence.deployment.source_tree -cne [string]$snapshot.source.tree -or
        $Evidence.deployment.image_id -cne [string]$snapshot.release_image.image_id -or
        $Evidence.deployment.database_name -cne [string]$snapshot.database_runtime.current_database -or
        $Evidence.deployment.installation_id -cne [string]$snapshot.installation.id -or
        [int64]$Evidence.deployment.catalog_entries -ne [int64]$snapshot.catalog.definition_count -or
        [int64]$Evidence.deployment.catalog_entries -ne 7 -or
        $Evidence.deployment.built_in_seed_sha256 -cne [string]$snapshot.built_in_seed.canonical_sha256 -or
        $Evidence.deployment.built_in_seed_sha256 -cne $script:Sprint6AAcceptanceEvidenceBuiltInSeedSha256) {
        throw 'Acceptance evidence deployment identity does not exactly match the retained deployment evidence.'
    }

    if ($Evidence.deployment.evidence_sha256 -cnotmatch '^[0-9a-f]{64}$' -or
        $Evidence.deployment.source_commit -cnotmatch '^[0-9a-f]{40}$' -or
        $Evidence.deployment.source_tree -cnotmatch '^[0-9a-f]{40}$' -or
        $Evidence.deployment.image_id -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw 'Acceptance evidence deployment digests and identities are malformed.'
    }

    if ($kind -eq 'smoke') {
        Assert-Sprint6AExactProperties -Object $Evidence.result -Expected @(
            'dataset_rows', 'component_rows', 'seeded_visual_points', 'visual_points'
        ) -Context 'smoke result'
        foreach ($field in @('dataset_rows', 'component_rows', 'seeded_visual_points', 'visual_points')) {
            Assert-Sprint6AExactInteger -Value $Evidence.result.$field -Context "result.$field"
            if ([int64]$Evidence.result.$field -lt 1) { throw "result.$field must be positive." }
        }
    } else {
        Assert-Sprint6AExactProperties -Object $Evidence.result -Expected @(
            'seed_version', 'dashboard_placements', 'component_kinds', 'authorization_roles_checked'
        ) -Context 'UAT result'
        Assert-Sprint6AExactString -Value $Evidence.result.seed_version -Context 'result.seed_version'
        Assert-Sprint6AExactInteger -Value $Evidence.result.dashboard_placements -Context 'result.dashboard_placements'
        Assert-Sprint6AExactStringArray -Value $Evidence.result.component_kinds -Expected $script:Sprint6AUatComponentKinds -Context 'result.component_kinds'
        Assert-Sprint6AExactStringArray -Value $Evidence.result.authorization_roles_checked -Expected $script:Sprint6AUatAuthorizationRoles -Context 'result.authorization_roles_checked'
        if ($Evidence.result.seed_version -cne 'uat-demo-v2' -or [int64]$Evidence.result.dashboard_placements -ne 9) {
            throw 'UAT result does not match the exact Sprint 6A seeded acceptance contract.'
        }
    }

    $serialized = $Evidence | ConvertTo-Json -Depth 30 -Compress
    if ($serialized -match '(?i)(bearer\s|tessara-dev-|password|set-cookie|session[_-]?token)') {
        throw 'Acceptance evidence contains credential- or session-like raw material.'
    }
}

function Assert-Sprint6AAcceptanceEvidenceDigest {
    param(
        [Parameter(Mandatory)][string]$EvidencePath,
        [string]$DigestPath = "$EvidencePath.sha256",
        [Parameter(Mandatory)][string]$DeploymentEvidencePath,
        [Parameter(Mandatory)][string]$RunnerFilePath
    )

    $evidenceInfo = Get-Sprint6ASecurePathInfo -Path $EvidencePath -RequireLeaf
    $digestInfo = Get-Sprint6ASecurePathInfo -Path $DigestPath -RequireLeaf
    Assert-Sprint6APathSetsDistinct -Left @($evidenceInfo) -Right @($digestInfo) -Context 'Acceptance evidence and digest'
    $expectedDigest = (Get-Content -LiteralPath $digestInfo.lexical_path -Raw).Trim()
    $actualDigest = (Get-FileHash -LiteralPath $evidenceInfo.lexical_path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expectedDigest -cnotmatch '^[0-9a-f]{64}$' -or $expectedDigest -cne $actualDigest) {
        throw "Acceptance evidence SHA-256 verification failed for '$($evidenceInfo.lexical_path)'."
    }
    try {
        $raw = Get-Content -LiteralPath $evidenceInfo.lexical_path -Raw
        $evidence = $raw | ConvertFrom-Json -DateKind String -NoEnumerate
    } catch {
        throw "Acceptance evidence '$($evidenceInfo.lexical_path)' is not valid JSON: $($_.Exception.Message)"
    }
    Assert-Sprint6AAcceptanceEvidenceDocument `
        -Evidence $evidence `
        -DeploymentEvidencePath $DeploymentEvidencePath `
        -RunnerFilePath $RunnerFilePath
    $actualDigest
}

function Write-Sprint6AFileExclusive {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][AllowEmptyString()][string]$Content
    )

    $info = Get-Sprint6ASecurePathInfo -Path $Path -AllowMissingLeaf
    if ($info.exists) { throw "Exclusive file creation refused existing path '$($info.lexical_path)'." }
    $stream = [IO.FileStream]::new(
        $info.lexical_path,
        [IO.FileMode]::CreateNew,
        [IO.FileAccess]::Write,
        [IO.FileShare]::None,
        4096,
        [IO.FileOptions]::WriteThrough
    )
    try {
        $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Content)
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    } finally {
        $stream.Dispose()
    }
    $null = Get-Sprint6ASecurePathInfo -Path $info.lexical_path -RequireLeaf
}

function New-Sprint6AExclusiveEmptyFile {
    param([Parameter(Mandatory)][string]$Path)
    Write-Sprint6AFileExclusive -Path $Path -Content ''
}

function New-Sprint6AAcceptancePublicationLock {
    param([Parameter(Mandatory)][string]$EvidencePath)

    $target = Get-Sprint6ASecurePathInfo -Path $EvidencePath -AllowMissingLeaf
    $lockName = '.sprint-6a-acceptance-' + (Get-Sprint6ASha256Text -Text $target.physical_path).Substring(0, 24) + '.lock'
    $lockPath = Join-Path (Split-Path -Parent $target.lexical_path) $lockName
    $lockInfo = Get-Sprint6ASecurePathInfo -Path $lockPath -AllowMissingLeaf
    if ($lockInfo.exists) {
        throw "A concurrent or stale acceptance evidence publication lock exists for '$($target.lexical_path)'."
    }
    try {
        [IO.FileStream]::new(
            $lockInfo.lexical_path,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::ReadWrite,
            [IO.FileShare]::None,
            1,
            ([IO.FileOptions]::DeleteOnClose -bor [IO.FileOptions]::WriteThrough)
        )
    } catch {
        throw "Could not exclusively reserve acceptance evidence publication for '$($target.lexical_path)': $($_.Exception.Message)"
    }
}

function Publish-Sprint6AAcceptanceEvidence {
    param(
        [Parameter(Mandatory)][string]$EvidencePath,
        [Parameter(Mandatory)][string]$DeploymentEvidencePath,
        [Parameter(Mandatory)][string]$RunnerFilePath,
        [Parameter(Mandatory)][object]$Evidence,
        [switch]$Overwrite,
        [ValidateSet('None', 'AfterEvidencePublish', 'FinalValidationFailure')][string]$FailurePoint = 'None'
    )

    $target = Assert-Sprint6AAcceptanceEvidenceTargetAvailable `
        -EvidencePath $EvidencePath `
        -DeploymentEvidencePath $DeploymentEvidencePath `
        -Overwrite:$Overwrite
    Assert-Sprint6AAcceptanceEvidenceDocument `
        -Evidence $Evidence `
        -DeploymentEvidencePath $target.deployment_evidence_path `
        -RunnerFilePath $RunnerFilePath

    $lock = $null
    $temporaryDirectory = $null
    $stagedEvidence = $null
    $stagedDigest = $null
    $backupEvidence = $null
    $backupDigest = $null
    $publishedEvidence = $false
    $publishedDigest = $false
    $backedUpEvidence = $false
    $backedUpDigest = $false
    try {
        $lock = New-Sprint6AAcceptancePublicationLock -EvidencePath $target.evidence_path
        $target = Assert-Sprint6AAcceptanceEvidenceTargetAvailable `
            -EvidencePath $target.evidence_path `
            -DeploymentEvidencePath $target.deployment_evidence_path `
            -Overwrite:$Overwrite

        $parent = Split-Path -Parent $target.evidence_path
        $leaf = Split-Path -Leaf $target.evidence_path
        $temporaryDirectory = Join-Path $parent (".$leaf.publish-" + [guid]::NewGuid().ToString('N'))
        [IO.Directory]::CreateDirectory($temporaryDirectory) | Out-Null
        $null = Get-Sprint6ASecurePathInfo -Path $temporaryDirectory -RequireContainer
        $stagedEvidence = Join-Path $temporaryDirectory 'evidence.json'
        $stagedDigest = Join-Path $temporaryDirectory 'evidence.json.sha256'
        $backupEvidence = Join-Path $temporaryDirectory 'previous-evidence.json'
        $backupDigest = Join-Path $temporaryDirectory 'previous-evidence.json.sha256'

        $json = $Evidence | ConvertTo-Json -Depth 30
        Write-Sprint6AFileExclusive -Path $stagedEvidence -Content "$json`n"
        $digest = (Get-FileHash -LiteralPath $stagedEvidence -Algorithm SHA256).Hash.ToLowerInvariant()
        Write-Sprint6AFileExclusive -Path $stagedDigest -Content "$digest`n"
        Assert-Sprint6AAcceptanceEvidenceDigest `
            -EvidencePath $stagedEvidence `
            -DigestPath $stagedDigest `
            -DeploymentEvidencePath $target.deployment_evidence_path `
            -RunnerFilePath $RunnerFilePath | Out-Null

        # This is a rollback-safe two-file transaction, not a reader-atomic pair.
        Assert-Sprint6AAcceptanceEvidenceDocument `
            -Evidence $Evidence `
            -DeploymentEvidencePath $target.deployment_evidence_path `
            -RunnerFilePath $RunnerFilePath
        $target = Assert-Sprint6AAcceptanceEvidenceTargetAvailable `
            -EvidencePath $target.evidence_path `
            -DeploymentEvidencePath $target.deployment_evidence_path `
            -Overwrite:$Overwrite
        if (Test-Path -LiteralPath $target.evidence_path) {
            [IO.File]::Move($target.evidence_path, $backupEvidence, $false)
            $backedUpEvidence = $true
        }
        if (Test-Path -LiteralPath $target.digest_path) {
            [IO.File]::Move($target.digest_path, $backupDigest, $false)
            $backedUpDigest = $true
        }
        [IO.File]::Move($stagedEvidence, $target.evidence_path, $false)
        $publishedEvidence = $true
        if ($FailurePoint -eq 'AfterEvidencePublish') {
            throw 'Injected acceptance evidence failure after publishing the artifact and before publishing its digest.'
        }
        [IO.File]::Move($stagedDigest, $target.digest_path, $false)
        $publishedDigest = $true
        if ($FailurePoint -eq 'FinalValidationFailure') {
            throw 'Injected final acceptance validation failure after publishing both files.'
        }
        $verifiedDigest = Assert-Sprint6AAcceptanceEvidenceDigest `
            -EvidencePath $target.evidence_path `
            -DeploymentEvidencePath $target.deployment_evidence_path `
            -RunnerFilePath $RunnerFilePath
        [pscustomobject][ordered]@{
            evidence_path = $target.evidence_path
            digest_path = $target.digest_path
            sha256 = $verifiedDigest
        }
    } catch {
        $publishError = $_
        $rollbackErrors = [Collections.Generic.List[string]]::new()
        foreach ($publishedPath in @(
            @{ Published = $publishedDigest; Path = $target.digest_path },
            @{ Published = $publishedEvidence; Path = $target.evidence_path }
        )) {
            if ($publishedPath.Published -and (Test-Path -LiteralPath $publishedPath.Path)) {
                try {
                    $null = Get-Sprint6ASecurePathInfo -Path $publishedPath.Path -RequireLeaf
                    Remove-Item -LiteralPath $publishedPath.Path -Force
                } catch { $rollbackErrors.Add("remove '$($publishedPath.Path)': $($_.Exception.Message)") }
            }
        }
        foreach ($backup in @(
            @{ BackedUp = $backedUpEvidence; Path = $backupEvidence; Destination = $target.evidence_path },
            @{ BackedUp = $backedUpDigest; Path = $backupDigest; Destination = $target.digest_path }
        )) {
            if ($backup.BackedUp -and (Test-Path -LiteralPath $backup.Path)) {
                try { [IO.File]::Move($backup.Path, $backup.Destination, $false) }
                catch { $rollbackErrors.Add("restore '$($backup.Destination)': $($_.Exception.Message)") }
            }
        }
        if ($rollbackErrors.Count -gt 0) {
            throw "Acceptance evidence publish failed: $($publishError.Exception.Message). Rollback also failed: $($rollbackErrors -join '; ')"
        }
        throw $publishError
    } finally {
        if (-not [string]::IsNullOrWhiteSpace($temporaryDirectory) -and (Test-Path -LiteralPath $temporaryDirectory)) {
            $null = Get-Sprint6ASecurePathInfo -Path $temporaryDirectory -RequireContainer
            Remove-Item -LiteralPath $temporaryDirectory -Recurse -Force
        }
        if ($null -ne $lock) { $lock.Dispose() }
    }
}

function Register-Sprint6ASensitivePath {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[string]]$Paths,
        [Parameter(Mandatory)][string]$Path
    )
    $fullPath = [IO.Path]::GetFullPath($Path)
    $null = Get-Sprint6ASecurePathInfo -Path $fullPath -AllowMissingLeaf
    if (@($Paths | Where-Object { [string]::Equals($_, $fullPath, [StringComparison]::OrdinalIgnoreCase) }).Count -gt 0) {
        throw "Sensitive path '$fullPath' was registered more than once."
    }
    $Paths.Add($fullPath)
    $fullPath
}

function Register-Sprint6ACurrentRunSession {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Sessions,
        [Parameter(Mandatory)][ValidateSet('bearer', 'browser')][string]$Source,
        [string]$Token,
        [string]$CookiePath
    )
    if ($Source -eq 'bearer' -and [string]::IsNullOrWhiteSpace($Token)) {
        throw 'A current-run bearer session requires its exact token.'
    }
    if ($Source -eq 'browser' -and [string]::IsNullOrWhiteSpace($CookiePath)) {
        throw 'A current-run browser session requires its exact cookie path.'
    }
    $normalizedToken = $null
    if (-not [string]::IsNullOrWhiteSpace($Token)) {
        $parsed = [guid]::Empty
        if (-not [guid]::TryParse($Token, [ref]$parsed) -or $parsed -eq [guid]::Empty) {
            throw 'A current-run session returned a non-UUID or zero token.'
        }
        $normalizedToken = $parsed.ToString('D').ToLowerInvariant()
        if (@($Sessions | Where-Object { $_.token -ceq $normalizedToken }).Count -gt 0) {
            throw 'A current-run session token was registered more than once.'
        }
    }
    $Sessions.Add([pscustomobject][ordered]@{
        source = $Source
        token = $normalizedToken
        cookie_path = if ([string]::IsNullOrWhiteSpace($CookiePath)) { $null } else { [IO.Path]::GetFullPath($CookiePath) }
        revoked = $false
    })
}

function Get-Sprint6ASessionTokensFromCookieJar {
    param([Parameter(Mandatory)][string]$CookiePath)

    $cookieInfo = Get-Sprint6ASecurePathInfo -Path $CookiePath -RequireLeaf
    $tokens = [Collections.Generic.List[string]]::new()
    foreach ($line in @(Get-Content -LiteralPath $cookieInfo.lexical_path -ErrorAction Stop)) {
        if ([string]::IsNullOrWhiteSpace($line) -or
            ($line.StartsWith('#') -and -not $line.StartsWith('#HttpOnly_', [StringComparison]::Ordinal))) {
            continue
        }
        $fields = @($line -split "`t")
        if ($fields.Count -lt 7) { continue }
        $candidate = [string]$fields[6]
        $parsed = [guid]::Empty
        if ([guid]::TryParse($candidate, [ref]$parsed) -and
            $parsed -ne [guid]::Empty -and
            $candidate -ceq $parsed.ToString('D').ToLowerInvariant()) {
            $normalized = $parsed.ToString('D').ToLowerInvariant()
            if ($tokens -cnotcontains $normalized) { $tokens.Add($normalized) }
        }
    }
    @($tokens)
}

function Complete-Sprint6ABrowserLoginObservation {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Sessions,
        [Parameter(Mandatory)][string]$CookiePath,
        [AllowNull()][object]$Response,
        [Parameter(Mandatory)][int]$CurlExitCode,
        [Parameter(Mandatory)][string]$Context
    )

    # The server may commit and set a session cookie before a transfer truncates.
    # Register every exact UUID cookie before interpreting curl status or body bytes.
    $cookieTokens = @(Get-Sprint6ASessionTokensFromCookieJar -CookiePath $CookiePath)
    foreach ($cookieToken in $cookieTokens) {
        if (@($Sessions | Where-Object { $_.token -ceq $cookieToken }).Count -eq 0) {
            Register-Sprint6ACurrentRunSession `
                -Sessions $Sessions `
                -Source browser `
                -Token $cookieToken `
                -CookiePath $CookiePath
        }
    }

    if ($CurlExitCode -ne 0) {
        throw "$Context curl login failed with exit code $CurlExitCode after committed-cookie inspection."
    }
    $responseText = [string](@($Response) -join [Environment]::NewLine)
    if ([string]::IsNullOrWhiteSpace($responseText)) {
        throw "$Context login response was empty after committed-cookie inspection."
    }
    try { $login = $responseText | ConvertFrom-Json -NoEnumerate }
    catch { throw "$Context login response was malformed after committed-cookie inspection: $($_.Exception.Message)" }
    if ($null -eq $login.PSObject.Properties['token'] -or $login.token -isnot [string]) {
        throw "$Context login response did not contain an exact string token after committed-cookie inspection."
    }
    $responseToken = [string]$login.token
    $parsedResponseToken = [guid]::Empty
    if (-not [guid]::TryParse($responseToken, [ref]$parsedResponseToken) -or
        $parsedResponseToken -eq [guid]::Empty -or
        $responseToken -cne $parsedResponseToken.ToString('D').ToLowerInvariant()) {
        throw "$Context login response token was not a nonzero canonical UUID after committed-cookie inspection."
    }
    if (@($Sessions | Where-Object { $_.token -ceq $responseToken }).Count -eq 0) {
        $source = if ($cookieTokens.Count -eq 0) { 'browser' } else { 'bearer' }
        Register-Sprint6ACurrentRunSession `
            -Sessions $Sessions `
            -Source $source `
            -Token $responseToken `
            -CookiePath $(if ($source -eq 'browser') { $CookiePath } else { $null })
    }
    if ($cookieTokens.Count -gt 1) {
        throw "$Context login produced multiple exact session cookies; all were registered for fail-closed cleanup."
    }
    [IO.Path]::GetFullPath($CookiePath)
}

function Get-Sprint6AProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)][string[]]$Names)
    $snapshot = [ordered]@{}
    foreach ($name in $Names) {
        $snapshot[$name] = [pscustomobject][ordered]@{
            exists = Test-Path -LiteralPath "Env:$name"
            value = [Environment]::GetEnvironmentVariable($name, [EnvironmentVariableTarget]::Process)
        }
    }
    $snapshot
}

function Restore-Sprint6AProcessEnvironmentSnapshot {
    param([Parameter(Mandatory)][Collections.IDictionary]$Snapshot)
    foreach ($name in $Snapshot.Keys) {
        $entry = $Snapshot[$name]
        if ([bool]$entry.exists) {
            [Environment]::SetEnvironmentVariable([string]$name, [string]$entry.value, [EnvironmentVariableTarget]::Process)
        } else {
            Remove-Item -LiteralPath "Env:$name" -Force -ErrorAction SilentlyContinue
        }
    }
}

function Assert-Sprint6AProcessEnvironmentRestored {
    param([Parameter(Mandatory)][Collections.IDictionary]$Snapshot)
    foreach ($name in $Snapshot.Keys) {
        $entry = $Snapshot[$name]
        $exists = Test-Path -LiteralPath "Env:$name"
        $value = [Environment]::GetEnvironmentVariable([string]$name, [EnvironmentVariableTarget]::Process)
        if ($exists -ne [bool]$entry.exists -or ($exists -and $value -cne [string]$entry.value)) {
            throw "Process environment variable '$name' was not restored exactly."
        }
    }
}

function Complete-Sprint6AAcceptanceRunCleanup {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Sessions,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[string]]$SensitivePaths,
        [Parameter(Mandatory)][scriptblock]$LogoutAction,
        [Collections.IDictionary]$EnvironmentSnapshot,
        [switch]$FinalAttempt
    )

    $errors = [Collections.Generic.List[string]]::new()
    foreach ($session in @($Sessions | Where-Object { -not $_.revoked })) {
        try {
            $response = & $LogoutAction $session
            if ($response -isnot [pscustomobject] -or
                $null -eq $response.PSObject.Properties['signed_out'] -or
                $response.signed_out -isnot [bool] -or
                -not $response.signed_out) {
                throw 'DELETE /api/auth/logout did not return exact signed_out=true.'
            }
            $session.revoked = $true
            $session.token = $null
        } catch {
            $errors.Add("$($session.source) session logout failed: $($_.Exception.Message)")
        }
    }

    $pendingBrowserPaths = @($Sessions | Where-Object { -not $_.revoked -and $_.source -eq 'browser' } | ForEach-Object cookie_path)
    foreach ($path in $SensitivePaths) {
        $retainForRetry = -not $FinalAttempt -and @($pendingBrowserPaths | Where-Object {
            [string]::Equals($_, $path, [StringComparison]::OrdinalIgnoreCase)
        }).Count -gt 0
        if ($retainForRetry) { continue }
        try {
            if (Test-Path -LiteralPath $path) {
                $null = Get-Sprint6ASecurePathInfo -Path $path -RequireLeaf
                Remove-Item -LiteralPath $path -Force
            }
            if (Test-Path -LiteralPath $path) { throw 'file still exists after removal' }
        } catch { $errors.Add("sensitive file cleanup failed without reading '$path': $($_.Exception.Message)") }
    }
    if ($null -ne $EnvironmentSnapshot) {
        try {
            Restore-Sprint6AProcessEnvironmentSnapshot -Snapshot $EnvironmentSnapshot
            Assert-Sprint6AProcessEnvironmentRestored -Snapshot $EnvironmentSnapshot
        } catch { $errors.Add("process environment restoration failed: $($_.Exception.Message)") }
    }
    if ($errors.Count -gt 0) {
        throw "Sprint 6A acceptance cleanup failed; evidence publication is forbidden: $($errors -join '; ')"
    }
    foreach ($session in $Sessions) { $session.cookie_path = $null }
}

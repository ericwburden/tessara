Set-StrictMode -Version Latest

$script:Sprint7ARepositoryRoot = Split-Path -Parent $PSScriptRoot
$script:Sprint7AFixtureContractPath = Join-Path $script:Sprint7ARepositoryRoot "deploy/sprint-7a/uat-fixture-contract.json"
$script:Sprint7AFixture = [ordered]@{
    installation_id = "01980000-0000-7000-8000-00000000007a"
    organization_id = "01980000-0002-7000-8000-000000000002"
    dataset_id = "01980000-0002-7000-8000-000000000003"
    metric_component_id = "01980000-0002-7000-8000-000000000004"
    metric_version_id = "01980000-0001-7000-8000-000000000001"
    table_component_id = "01980000-0002-7000-8000-000000000005"
    table_version_id = "01980000-0001-7000-8000-000000000002"
    dashboard_id = "01980000-0003-7000-8000-000000000001"
    metric_placement_id = "01980000-0003-7000-8000-000000000002"
    table_placement_id = "01980000-0003-7000-8000-000000000003"
    chart_component_id = "01980000-0002-7000-8000-000000000006"
    chart_version_id = "01980000-0001-7000-8000-000000000003"
    chart_placement_id = "01980000-0003-7000-8000-000000000004"
    blocked_placement_id = "01980000-0003-7000-8000-000000000005"
    blocked_dashboard_id = "01980000-0003-7000-8000-000000000006"
    blocked_organization_id = "01980000-0002-7000-8000-000000000007"
    blocked_dataset_id = "01980000-0002-7000-8000-000000000008"
    blocked_dataset_revision_id = "01980000-0002-7000-8000-000000000009"
    blocked_component_id = "01980000-0002-7000-8000-00000000000b"
    blocked_component_version_id = "01980000-0001-7000-8000-000000000004"
}

function Get-Sprint7AFixtureContract {
    if (-not (Test-Path -LiteralPath $script:Sprint7AFixtureContractPath -PathType Leaf)) {
        throw "Sprint 7A UAT fixture contract is missing: $script:Sprint7AFixtureContractPath"
    }
    Get-Content -LiteralPath $script:Sprint7AFixtureContractPath -Raw | ConvertFrom-Json
}

function Assert-Sprint7AFixtureContract {
    $contract = Get-Sprint7AFixtureContract
    if ($contract.schema_version -ne 1 -or $contract.contract -cne "tessara.sprint-7a.uat-fixtures") {
        throw "Sprint 7A UAT fixture contract identity is invalid."
    }
    $requiredActors = @("administrator", "scoped_operator", "mixed_scope_operator", "no_analytics_actor", "undeclared_service", "wrong_service_instance")
    $requiredDatasets = @("four_tier", "blocked")
    $requiredRows = @("public", "internal", "restricted", "confidential_blocked")
    $requiredComponents = @("table", "chart", "stat", "blocked")
    $requiredDashboards = @("mixed", "blocked")
    $requiredPlacements = @("table", "chart", "stat", "blocked")
    $requiredFreshness = @("authorization_revision", "organization_revision", "dataset_authority_revision", "component_authority_revision", "dashboard_authority_revision")
    foreach ($entry in @(
        @{ values = $requiredActors; object = $contract.actors; label = "actor" },
        @{ values = $requiredDatasets; object = $contract.datasets; label = "Dataset" },
        @{ values = $requiredRows; object = $contract.dataset_rows; label = "row tier" },
        @{ values = $requiredComponents; object = $contract.component_versions; label = "ComponentVersion" },
        @{ values = $requiredDashboards; object = $contract.dashboards; label = "Dashboard" },
        @{ values = $requiredPlacements; object = $contract.placements; label = "placement" }
    )) {
        foreach ($name in $entry.values) {
            if ([string]::IsNullOrWhiteSpace([string]$entry.object.$name)) {
                throw "Sprint 7A UAT fixture contract is missing $($entry.label) '$name'."
            }
        }
    }
    foreach ($name in $requiredFreshness) {
        if (@($contract.freshness_specimens) -cnotcontains $name) {
            throw "Sprint 7A UAT fixture contract is missing freshness specimen '$name'."
        }
    }
    if ([string]::IsNullOrWhiteSpace([string]$contract.identifier_specimens.known_blocked) -or
        [string]::IsNullOrWhiteSpace([string]$contract.identifier_specimens.random) -or
        $contract.identifier_specimens.known_blocked -ceq $contract.identifier_specimens.random) {
        throw "Sprint 7A UAT fixture contract must contain distinct known-blocked and random identifier specimens."
    }
    $contract
}

function Get-Sprint7ASha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        ([BitConverter]::ToString(
            $algorithm.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text))
        ) -replace "-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Get-Sprint7AFileSha256 {
    param([Parameter(Mandatory)][string]$Path)
    $stream = [IO.File]::OpenRead([IO.Path]::GetFullPath($Path))
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        ([BitConverter]::ToString($algorithm.ComputeHash($stream)) -replace "-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
        $stream.Dispose()
    }
}

function Invoke-Sprint7ARequest {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Path,
        [ValidateSet("GET", "POST", "PUT", "DELETE")][string]$Method = "GET",
        [string]$Token,
        [object]$Body
    )
    $headers = @{}
    if (-not [string]::IsNullOrWhiteSpace($Token)) {
        $headers.Authorization = "Bearer $Token"
    }
    $parameters = @{
        Uri = "$($BaseUrl.TrimEnd('/'))$Path"
        Method = $Method
        Headers = $headers
        UseBasicParsing = $true
    }
    $supportsSkipHttpErrorCheck = (Get-Command Invoke-WebRequest).Parameters.ContainsKey("SkipHttpErrorCheck")
    if ($supportsSkipHttpErrorCheck) {
        $parameters.SkipHttpErrorCheck = $true
    }
    if ($null -ne $Body) {
        $parameters.ContentType = "application/json"
        $parameters.Body = $Body | ConvertTo-Json -Depth 20 -Compress
    }
    $status = 0
    $contentType = ""
    $content = ""
    try {
        $response = Invoke-WebRequest @parameters
        $status = [int]$response.StatusCode
        $contentType = [string]$response.Headers["Content-Type"]
        $content = [string]$response.Content
    } catch {
        if ($supportsSkipHttpErrorCheck -or $null -eq $_.Exception.Response) {
            throw
        }
        $response = $_.Exception.Response
        $status = [int]$response.StatusCode
        $contentType = [string]$response.ContentType
        $stream = $response.GetResponseStream()
        if ($null -ne $stream) {
            $reader = [IO.StreamReader]::new($stream)
            try {
                $content = $reader.ReadToEnd()
            } finally {
                $reader.Dispose()
                $stream.Dispose()
            }
        }
        if ([string]::IsNullOrEmpty($content) -and -not [string]::IsNullOrEmpty([string]$_.ErrorDetails.Message)) {
            $content = [string]$_.ErrorDetails.Message
        }
    }
    [pscustomobject]@{
        status = $status
        content_type = $contentType
        body = $content
        body_sha256 = Get-Sprint7ASha256 -Text $content
        body_utf8_length = [Text.Encoding]::UTF8.GetByteCount($content)
    }
}

function Get-Sprint7AToken {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Email,
        [Parameter(Mandatory)][string]$Password
    )
    $response = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/login" -Method POST -Body @{
        email = $Email
        password = $Password
    }
    if ($response.status -ne 200) {
        throw "Login failed for '$Email' with HTTP $($response.status)."
    }
    $document = $response.body | ConvertFrom-Json
    if ([string]::IsNullOrWhiteSpace([string]$document.token)) {
        throw "Login response for '$Email' omitted the bearer token."
    }
    [string]$document.token
}

function Assert-Sprint7A {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Code,
        [Parameter(Mandatory)][string]$Detail,
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[object]]$Checks
    )
    $Checks.Add([pscustomobject][ordered]@{ code = $Code; passed = $Condition; detail = $Detail })
    if (-not $Condition) {
        throw "Sprint 7A acceptance failed: $Code - $Detail"
    }
}

function Test-Sprint7AEvidencePair {
    param(
        [Parameter(Mandatory)][string]$ArtifactPath,
        [Parameter(Mandatory)][string]$SidecarPath
    )

    if (-not (Test-Path -LiteralPath $ArtifactPath -PathType Leaf) -or
        -not (Test-Path -LiteralPath $SidecarPath -PathType Leaf)) {
        return $false
    }
    try {
        $expected = (Get-Content -LiteralPath $SidecarPath -Raw).Trim()
        $expected -match '^[0-9a-f]{64}$' -and
            (Get-Sprint7AFileSha256 -Path $ArtifactPath) -ceq $expected
    } catch { $false }
}

function Get-Sprint7AEvidencePublicationTransients {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    $directory = Split-Path -Parent $fullPath
    if (-not (Test-Path -LiteralPath $directory -PathType Container)) {
        return @()
    }
    $leaf = [regex]::Escape([IO.Path]::GetFileName($fullPath))
    $publisherTemporaryPattern = "^\.$leaf\.[0-9a-f]{32}\.tmp(?:\.sha256)?$"
    $partialPairTemporaryPattern = "^\.$leaf\.[0-9a-f]{32}\.(?:json|sha256)\.tmp$"
    $journalTemporaryPattern = "^\.$leaf\.publish-journal\.json\.[0-9a-f]{32}\.tmp$"
    @(
        Get-ChildItem -LiteralPath $directory -Force -File | Where-Object {
            $_.Name -cmatch $publisherTemporaryPattern -or
                $_.Name -cmatch $partialPairTemporaryPattern -or
                $_.Name -cmatch $journalTemporaryPattern
        } | ForEach-Object { [IO.Path]::GetFullPath($_.FullName) }
    )
}

function Remove-Sprint7AEvidencePublicationTransients {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string[]]$AdditionalPaths = @()
    )

    $ownedPaths = @((Get-Sprint7AEvidencePublicationTransients -Path $Path)) + @($AdditionalPaths)
    foreach ($ownedPath in @($ownedPaths | Where-Object {
        -not [string]::IsNullOrWhiteSpace([string]$_)
    } | Select-Object -Unique)) {
        Remove-Item -LiteralPath ([string]$ownedPath) -Force -ErrorAction SilentlyContinue
    }
}

function Set-Sprint7AEvidencePublicationPair {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$ArtifactPath,
        [Parameter(Mandatory)][string]$SidecarPath
    )

    $fullPath = [IO.Path]::GetFullPath($Path)
    $fullSidecarPath = "$fullPath.sha256"
    $recoveryArtifactPath = [IO.Path]::GetFullPath($ArtifactPath)
    $recoverySidecarPath = [IO.Path]::GetFullPath($SidecarPath)
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $recoveryArtifactPath -SidecarPath $recoverySidecarPath)) {
        throw "Evidence publication recovery source is not authenticated for '$fullPath'."
    }
    if (-not [string]::Equals($recoveryArtifactPath, $fullPath, [StringComparison]::OrdinalIgnoreCase)) {
        Remove-Item -LiteralPath $fullPath -Force -ErrorAction SilentlyContinue
        Move-Item -LiteralPath $recoveryArtifactPath -Destination $fullPath
    }
    if (-not [string]::Equals($recoverySidecarPath, $fullSidecarPath, [StringComparison]::OrdinalIgnoreCase)) {
        Remove-Item -LiteralPath $fullSidecarPath -Force -ErrorAction SilentlyContinue
        Move-Item -LiteralPath $recoverySidecarPath -Destination $fullSidecarPath
    }
    if (-not (Test-Sprint7AEvidencePair -ArtifactPath $fullPath -SidecarPath $fullSidecarPath)) {
        throw "Evidence publication recovery produced an unauthenticated pair for '$fullPath'."
    }
}

function Repair-Sprint7AEvidencePublication {
    param([Parameter(Mandatory)][string]$Path)

    $fullPath = [IO.Path]::GetFullPath($Path)
    $sidecar = "$fullPath.sha256"
    $journalPath = "$fullPath.publish-journal.json"
    $rollbackPath = "$fullPath.rollback"
    $rollbackSidecarPath = "$sidecar.rollback"
    $transients = @(Get-Sprint7AEvidencePublicationTransients -Path $fullPath)
    $leaf = [regex]::Escape([IO.Path]::GetFileName($fullPath))
    $publisherTemporaryPattern = "^\.$leaf\.[0-9a-f]{32}\.tmp$"
    $publisherTemporarySidecarPattern = "^\.$leaf\.[0-9a-f]{32}\.tmp\.sha256$"
    $temporaryArtifacts = @($transients | Where-Object {
        [IO.Path]::GetFileName([string]$_) -cmatch $publisherTemporaryPattern
    })
    $temporarySidecars = @($transients | Where-Object {
        [IO.Path]::GetFileName([string]$_) -cmatch $publisherTemporarySidecarPattern
    })
    $rollbackCandidates = @(
        [pscustomobject]@{ artifact = $rollbackPath; sidecar = $rollbackSidecarPath },
        [pscustomobject]@{ artifact = $rollbackPath; sidecar = $sidecar },
        [pscustomobject]@{ artifact = $fullPath; sidecar = $rollbackSidecarPath }
    )
    $completionCandidates = [Collections.Generic.List[object]]::new()
    foreach ($temporaryArtifact in $temporaryArtifacts) {
        $temporaryArtifactSidecar = "$temporaryArtifact.sha256"
        $completionCandidates.Add([pscustomobject]@{ artifact = $temporaryArtifact; sidecar = $temporaryArtifactSidecar })
        $completionCandidates.Add([pscustomobject]@{ artifact = $fullPath; sidecar = $temporaryArtifactSidecar })
        $completionCandidates.Add([pscustomobject]@{ artifact = $temporaryArtifact; sidecar = $sidecar })
    }
    foreach ($temporaryPublicationSidecar in $temporarySidecars) {
        $completionCandidates.Add([pscustomobject]@{ artifact = $fullPath; sidecar = $temporaryPublicationSidecar })
    }
    $fixedControls = @($rollbackPath, $rollbackSidecarPath, $journalPath)
    if (Test-Sprint7AEvidencePair -ArtifactPath $fullPath -SidecarPath $sidecar) {
        Remove-Sprint7AEvidencePublicationTransients -Path $fullPath -AdditionalPaths $fixedControls
        return
    }

    $journalExists = Test-Path -LiteralPath $journalPath -PathType Leaf
    $journal = $null
    $journalValid = $false
    $temporary = $null
    $temporarySidecar = $null
    if ($journalExists) {
        try {
            $journal = Get-Content -LiteralPath $journalPath -Raw | ConvertFrom-Json
            $temporary = [IO.Path]::GetFullPath([string]$journal.temporary)
            $temporarySidecar = [IO.Path]::GetFullPath([string]$journal.temporary_sidecar)
            $journalValid = [int]$journal.schema_version -eq 1 -and
                [string]::Equals([string]$journal.path, $fullPath, [StringComparison]::OrdinalIgnoreCase) -and
                [string]::Equals([string]$journal.sidecar, $sidecar, [StringComparison]::OrdinalIgnoreCase) -and
                [IO.Path]::GetFileName($temporary) -cmatch $publisherTemporaryPattern -and
                [string]::Equals((Split-Path -Parent $temporary), (Split-Path -Parent $fullPath), [StringComparison]::OrdinalIgnoreCase) -and
                [string]::Equals($temporarySidecar, "$temporary.sha256", [StringComparison]::OrdinalIgnoreCase) -and
                $journal.PSObject.Properties.Name -contains "had_prior" -and
                $journal.had_prior -is [bool] -and
                [string]$journal.intended_sha256 -cmatch '^[0-9a-f]{64}$'
        } catch {
            $journalValid = $false
        }
    }

    if (-not $journalExists) {
        $rollbackRecovery = @($rollbackCandidates | Where-Object {
            Test-Sprint7AEvidencePair -ArtifactPath ([string]$_.artifact) -SidecarPath ([string]$_.sidecar)
        } | Select-Object -First 1)
        if ($rollbackRecovery.Count -eq 1) {
            Set-Sprint7AEvidencePublicationPair `
                -Path $fullPath `
                -ArtifactPath ([string]$rollbackRecovery[0].artifact) `
                -SidecarPath ([string]$rollbackRecovery[0].sidecar)
        }
        Remove-Sprint7AEvidencePublicationTransients `
            -Path $fullPath `
            -AdditionalPaths @($rollbackPath, $rollbackSidecarPath)
        return
    }

    $recovery = @()
    if ($journalValid) {
        $journalCompletionCandidates = @(
            [pscustomobject]@{ artifact = $temporary; sidecar = $temporarySidecar; intended = $true },
            [pscustomobject]@{ artifact = $fullPath; sidecar = $temporarySidecar; intended = $true },
            [pscustomobject]@{ artifact = $temporary; sidecar = $sidecar; intended = $true }
        )
        $journalRollbackCandidates = @($rollbackCandidates | ForEach-Object {
            [pscustomobject]@{ artifact = $_.artifact; sidecar = $_.sidecar; intended = $false }
        })
        $recoveryCandidates = if ([bool]$journal.had_prior) {
            @($journalRollbackCandidates) + @($journalCompletionCandidates)
        } else {
            @($journalCompletionCandidates) + @($journalRollbackCandidates)
        }
        $recovery = @($recoveryCandidates | Where-Object {
            (Test-Sprint7AEvidencePair -ArtifactPath ([string]$_.artifact) -SidecarPath ([string]$_.sidecar)) -and
                (-not [bool]$_.intended -or
                    (Get-Sprint7AFileSha256 -Path ([string]$_.artifact)) -ceq [string]$journal.intended_sha256)
        } | Select-Object -First 1)
    } else {
        $recovery = @($rollbackCandidates | Where-Object {
            Test-Sprint7AEvidencePair -ArtifactPath ([string]$_.artifact) -SidecarPath ([string]$_.sidecar)
        } | Select-Object -First 1)
        if ($recovery.Count -eq 0) {
            $authenticatedCompletions = @($completionCandidates | Where-Object {
                Test-Sprint7AEvidencePair -ArtifactPath ([string]$_.artifact) -SidecarPath ([string]$_.sidecar)
            })
            $completionDigests = @($authenticatedCompletions | ForEach-Object {
                Get-Sprint7AFileSha256 -Path ([string]$_.artifact)
            } | Select-Object -Unique)
            if ($completionDigests.Count -eq 1) {
                $recovery = @($authenticatedCompletions | Select-Object -First 1)
            }
        }
    }

    if ($recovery.Count -eq 1) {
        Set-Sprint7AEvidencePublicationPair `
            -Path $fullPath `
            -ArtifactPath ([string]$recovery[0].artifact) `
            -SidecarPath ([string]$recovery[0].sidecar)
    } else {
        # A committed but unauthenticated publication is fail-closed to a clean retry boundary.
        Remove-Item -LiteralPath $fullPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $sidecar -Force -ErrorAction SilentlyContinue
    }
    Remove-Sprint7AEvidencePublicationTransients -Path $fullPath -AdditionalPaths $fixedControls
}

function Publish-Sprint7AEvidence {
    param(
        [Parameter(Mandatory)][object]$Document,
        [Parameter(Mandatory)][string]$OutputPath,
        [switch]$Overwrite
    )
    $fullPath = if ([IO.Path]::IsPathRooted($OutputPath)) {
        [IO.Path]::GetFullPath($OutputPath)
    } else {
        [IO.Path]::GetFullPath((Join-Path $script:Sprint7ARepositoryRoot $OutputPath))
    }
    $sidecar = "$fullPath.sha256"
    Repair-Sprint7AEvidencePublication -Path $fullPath
    $artifactExists = Test-Path -LiteralPath $fullPath -PathType Leaf
    $sidecarExists = Test-Path -LiteralPath $sidecar -PathType Leaf
    if (($artifactExists -or $sidecarExists) -and -not $Overwrite) {
        throw "Retained evidence exists; use -Overwrite only for an intentional replacement: $fullPath"
    }
    $directory = Split-Path -Parent $fullPath
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    $temporary = Join-Path $directory ".$([IO.Path]::GetFileName($fullPath)).$([guid]::NewGuid().ToString('N')).tmp"
    $temporarySidecar = "$temporary.sha256"
    $journalPath = "$fullPath.publish-journal.json"
    $rollbackPath = "$fullPath.rollback"
    $rollbackSidecarPath = "$sidecar.rollback"
    $journalTemporary = Join-Path $directory ".$([IO.Path]::GetFileName($fullPath)).publish-journal.json.$([guid]::NewGuid().ToString('N')).tmp"
    try {
        $json = ($Document | ConvertTo-Json -Depth 30) + "`n"
        [IO.File]::WriteAllText($temporary, $json, [Text.UTF8Encoding]::new($false))
        $digest = Get-Sprint7AFileSha256 -Path $temporary
        [IO.File]::WriteAllText($temporarySidecar, "$digest`n", [Text.UTF8Encoding]::new($false))
        if ($Overwrite -and (-not $artifactExists -or -not $sidecarExists -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $fullPath -SidecarPath $sidecar))) {
            throw "Intentional evidence replacement requires one authenticated prior JSON/sidecar pair: $fullPath"
        }
        $journal = [pscustomobject][ordered]@{
            schema_version = 1
            path = $fullPath
            sidecar = $sidecar
            temporary = $temporary
            temporary_sidecar = $temporarySidecar
            had_prior = [bool]$artifactExists
            intended_sha256 = $digest
        }
        [IO.File]::WriteAllText(
            $journalTemporary,
            (($journal | ConvertTo-Json -Depth 10) + "`n"),
            [Text.UTF8Encoding]::new($false)
        )
        Move-Item -LiteralPath $journalTemporary -Destination $journalPath
        if ($Overwrite) {
            [IO.File]::Replace($temporary, $fullPath, $rollbackPath, $true)
            [IO.File]::Replace($temporarySidecar, $sidecar, $rollbackSidecarPath, $true)
        } else {
            Move-Item -LiteralPath $temporary -Destination $fullPath
            Move-Item -LiteralPath $temporarySidecar -Destination $sidecar
        }
        if ((Get-Sprint7AFileSha256 -Path $fullPath) -cne $digest) {
            throw "Published evidence digest changed for '$fullPath'."
        }
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $fullPath -SidecarPath $sidecar)) {
            throw "Published evidence pair is not authenticated for '$fullPath'."
        }
        Remove-Sprint7AEvidencePublicationTransients `
            -Path $fullPath `
            -AdditionalPaths @($rollbackPath, $rollbackSidecarPath, $journalPath)
        [pscustomobject]@{ path = $fullPath; sha256 = $digest }
    } catch {
        $publicationError = $_
        try {
            Repair-Sprint7AEvidencePublication -Path $fullPath
        } catch {
            throw "Evidence publication failed and recovery could not authenticate a pair for '$fullPath': $($publicationError.Exception.Message); recovery: $($_.Exception.Message)"
        }
        throw $publicationError
    } finally {
        if (-not (Test-Path -LiteralPath $journalPath -PathType Leaf)) {
            Remove-Sprint7AEvidencePublicationTransients -Path $fullPath
        }
    }
}

function Test-Sprint7AAcceptanceContract {
    $null = Assert-Sprint7AFixtureContract
    if ((Get-Sprint7ASha256 -Text "tessara") -cne "05d9b610d3ebf2405566edefc77a6c676d43ba447b08fc3c6573972a8b7d3359") {
        throw "Sprint 7A SHA-256 helper is not runtime-compatible."
    }
    $root = Join-Path ([IO.Path]::GetTempPath()) "tessara-sprint-7a-acceptance-$([guid]::NewGuid().ToString('N'))"
    try {
        $path = Join-Path $root "evidence.json"
        $published = Publish-Sprint7AEvidence -Document ([ordered]@{ schema_version = 1; passed = $true }) -OutputPath $path
        if (-not (Test-Path -LiteralPath $published.path) -or -not (Test-Path -LiteralPath "$($published.path).sha256")) {
            throw "Sprint 7A evidence publication self-test did not create the exact JSON/sidecar pair."
        }
        $parsed = Get-Content -LiteralPath $published.path -Raw | ConvertFrom-Json
        if ($parsed.schema_version -ne 1 -or -not $parsed.passed) {
            throw "Sprint 7A evidence publication self-test did not retain the exact document."
        }
        $replacement = Publish-Sprint7AEvidence `
            -Document ([ordered]@{ schema_version = 1; passed = $false; revision = 2 }) `
            -OutputPath $path `
            -Overwrite
        $replacementParsed = Get-Content -LiteralPath $replacement.path -Raw | ConvertFrom-Json
        if ($replacementParsed.revision -ne 2 -or $replacementParsed.passed -ne $false -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $path -SidecarPath "$path.sha256")) {
            throw "Sprint 7A recoverable evidence replacement self-test failed."
        }
        $interruptedPath = Join-Path $root "interrupted.json"
        Publish-Sprint7AEvidence -Document ([ordered]@{ generation = "old" }) -OutputPath $interruptedPath | Out-Null
        $temporary = Join-Path $root ".interrupted.json.$([guid]::NewGuid().ToString('N')).tmp"
        $temporarySidecar = "$temporary.sha256"
        [IO.File]::WriteAllText($temporary, (([ordered]@{ generation = "new" } | ConvertTo-Json -Depth 30) + "`n"), [Text.UTF8Encoding]::new($false))
        $temporaryDigest = Get-Sprint7AFileSha256 -Path $temporary
        [IO.File]::WriteAllText($temporarySidecar, "$temporaryDigest`n", [Text.UTF8Encoding]::new($false))
        $journalPath = "$interruptedPath.publish-journal.json"
        [IO.File]::WriteAllText(
            $journalPath,
            (([ordered]@{
                schema_version = 1; path = $interruptedPath; sidecar = "$interruptedPath.sha256"
                temporary = $temporary; temporary_sidecar = $temporarySidecar
                had_prior = $true; intended_sha256 = $temporaryDigest
            } | ConvertTo-Json -Depth 10) + "`n"),
            [Text.UTF8Encoding]::new($false)
        )
        [IO.File]::Replace($temporary, $interruptedPath, "$interruptedPath.rollback", $true)
        Repair-Sprint7AEvidencePublication -Path $interruptedPath
        $recovered = Get-Content -LiteralPath $interruptedPath -Raw | ConvertFrom-Json
        if ([string]$recovered.generation -cne "old" -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $interruptedPath -SidecarPath "$interruptedPath.sha256") -or
            (Test-Path -LiteralPath $journalPath)) {
            throw "Sprint 7A interrupted evidence replacement did not restore its authenticated prior pair."
        }

        $preJournalPath = Join-Path $root "pre-journal.json"
        $preJournalTemporary = Join-Path $root ".pre-journal.json.$([guid]::NewGuid().ToString('N')).tmp"
        $preJournalTemporarySidecar = "$preJournalTemporary.sha256"
        [IO.File]::WriteAllText($preJournalTemporary, "{`"generation`":`"uncommitted`"}`n", [Text.UTF8Encoding]::new($false))
        $preJournalDigest = Get-Sprint7AFileSha256 -Path $preJournalTemporary
        [IO.File]::WriteAllText($preJournalTemporarySidecar, "$preJournalDigest`n", [Text.UTF8Encoding]::new($false))
        $orphanJournalTemporary = Join-Path $root ".pre-journal.json.publish-journal.json.$([guid]::NewGuid().ToString('N')).tmp"
        [IO.File]::WriteAllText($orphanJournalTemporary, "{", [Text.UTF8Encoding]::new($false))
        Repair-Sprint7AEvidencePublication -Path $preJournalPath
        if ((Test-Path -LiteralPath $preJournalPath) -or
            (Test-Path -LiteralPath $preJournalTemporary) -or
            (Test-Path -LiteralPath $preJournalTemporarySidecar) -or
            (Test-Path -LiteralPath $orphanJournalTemporary)) {
            throw "Sprint 7A pre-journal publication recovery did not discard only its uncommitted transients."
        }

        $malformedPath = Join-Path $root "malformed.json"
        $malformedTemporary = Join-Path $root ".malformed.json.$([guid]::NewGuid().ToString('N')).tmp"
        $malformedTemporarySidecar = "$malformedTemporary.sha256"
        [IO.File]::WriteAllText($malformedTemporary, "{`"generation`":`"recoverable`"}`n", [Text.UTF8Encoding]::new($false))
        $malformedDigest = Get-Sprint7AFileSha256 -Path $malformedTemporary
        [IO.File]::WriteAllText($malformedTemporarySidecar, "$malformedDigest`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText("$malformedPath.publish-journal.json", "{", [Text.UTF8Encoding]::new($false))
        Repair-Sprint7AEvidencePublication -Path $malformedPath
        $malformedRecovered = Get-Content -LiteralPath $malformedPath -Raw | ConvertFrom-Json
        if ([string]$malformedRecovered.generation -cne "recoverable" -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $malformedPath -SidecarPath "$malformedPath.sha256") -or
            (Test-Path -LiteralPath "$malformedPath.publish-journal.json")) {
            throw "Sprint 7A malformed-journal recovery did not complete its sole authenticated pair."
        }

        $malformedMovedPath = Join-Path $root "malformed-moved.json"
        $malformedMovedTemporary = Join-Path $root ".malformed-moved.json.$([guid]::NewGuid().ToString('N')).tmp"
        $malformedMovedTemporarySidecar = "$malformedMovedTemporary.sha256"
        [IO.File]::WriteAllText($malformedMovedTemporary, "{`"generation`":`"moved`"}`n", [Text.UTF8Encoding]::new($false))
        $malformedMovedDigest = Get-Sprint7AFileSha256 -Path $malformedMovedTemporary
        [IO.File]::WriteAllText($malformedMovedTemporarySidecar, "$malformedMovedDigest`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText("$malformedMovedPath.publish-journal.json", "{", [Text.UTF8Encoding]::new($false))
        Move-Item -LiteralPath $malformedMovedTemporary -Destination $malformedMovedPath
        Repair-Sprint7AEvidencePublication -Path $malformedMovedPath
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $malformedMovedPath -SidecarPath "$malformedMovedPath.sha256") -or
            [string](Get-Content -LiteralPath $malformedMovedPath -Raw | ConvertFrom-Json).generation -cne "moved") {
            throw "Sprint 7A malformed-journal recovery did not authenticate a partially moved publication."
        }

        $partialPath = Join-Path $root "partial-journal.json"
        $partialTemporary = Join-Path $root ".partial-journal.json.$([guid]::NewGuid().ToString('N')).tmp"
        $partialTemporarySidecar = "$partialTemporary.sha256"
        [IO.File]::WriteAllText($partialTemporary, "{`"generation`":`"partial`"}`n", [Text.UTF8Encoding]::new($false))
        $partialDigest = Get-Sprint7AFileSha256 -Path $partialTemporary
        [IO.File]::WriteAllText($partialTemporarySidecar, "$partialDigest`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText(
            "$partialPath.publish-journal.json",
            (([ordered]@{
                schema_version = 1; path = $partialPath; sidecar = "$partialPath.sha256"
                temporary = $partialTemporary; had_prior = $false; intended_sha256 = $partialDigest
            } | ConvertTo-Json -Depth 10) + "`n"),
            [Text.UTF8Encoding]::new($false)
        )
        Repair-Sprint7AEvidencePublication -Path $partialPath
        if (-not (Test-Sprint7AEvidencePair -ArtifactPath $partialPath -SidecarPath "$partialPath.sha256") -or
            [string](Get-Content -LiteralPath $partialPath -Raw | ConvertFrom-Json).generation -cne "partial") {
            throw "Sprint 7A partial-journal recovery did not complete its sole authenticated pair."
        }

        $malformedOverwritePath = Join-Path $root "malformed-overwrite.json"
        Publish-Sprint7AEvidence -Document ([ordered]@{ generation = "prior" }) -OutputPath $malformedOverwritePath | Out-Null
        $malformedOverwriteTemporary = Join-Path $root ".malformed-overwrite.json.$([guid]::NewGuid().ToString('N')).tmp"
        $malformedOverwriteTemporarySidecar = "$malformedOverwriteTemporary.sha256"
        [IO.File]::WriteAllText($malformedOverwriteTemporary, "{`"generation`":`"replacement`"}`n", [Text.UTF8Encoding]::new($false))
        $malformedOverwriteDigest = Get-Sprint7AFileSha256 -Path $malformedOverwriteTemporary
        [IO.File]::WriteAllText($malformedOverwriteTemporarySidecar, "$malformedOverwriteDigest`n", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText("$malformedOverwritePath.publish-journal.json", "{`"schema_version`":1", [Text.UTF8Encoding]::new($false))
        [IO.File]::Replace(
            $malformedOverwriteTemporary,
            $malformedOverwritePath,
            "$malformedOverwritePath.rollback",
            $true
        )
        Repair-Sprint7AEvidencePublication -Path $malformedOverwritePath
        $malformedOverwriteRecovered = Get-Content -LiteralPath $malformedOverwritePath -Raw | ConvertFrom-Json
        if ([string]$malformedOverwriteRecovered.generation -cne "prior" -or
            -not (Test-Sprint7AEvidencePair -ArtifactPath $malformedOverwritePath -SidecarPath "$malformedOverwritePath.sha256")) {
            throw "Sprint 7A malformed overwrite journal did not restore its authenticated prior pair."
        }

        $journalOnlyPath = Join-Path $root "journal-only.json"
        [IO.File]::WriteAllText("$journalOnlyPath.publish-journal.json", "{", [Text.UTF8Encoding]::new($false))
        Repair-Sprint7AEvidencePublication -Path $journalOnlyPath
        if ((Test-Path -LiteralPath $journalOnlyPath) -or
            (Test-Path -LiteralPath "$journalOnlyPath.sha256") -or
            (Test-Path -LiteralPath "$journalOnlyPath.publish-journal.json")) {
            throw "Sprint 7A journal-only recovery did not return to a clean retry boundary."
        }
    } finally {
        Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue
    }
}

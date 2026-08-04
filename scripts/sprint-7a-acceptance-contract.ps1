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
    $artifactExists = Test-Path -LiteralPath $fullPath -PathType Leaf
    $sidecarExists = Test-Path -LiteralPath $sidecar -PathType Leaf
    if (($artifactExists -or $sidecarExists) -and -not $Overwrite) {
        throw "Retained evidence exists; use -Overwrite only for an intentional replacement: $fullPath"
    }
    $directory = Split-Path -Parent $fullPath
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    $temporary = Join-Path $directory ".$([IO.Path]::GetFileName($fullPath)).$([guid]::NewGuid().ToString('N')).tmp"
    $temporarySidecar = "$temporary.sha256"
    try {
        $json = ($Document | ConvertTo-Json -Depth 30) + "`n"
        [IO.File]::WriteAllText($temporary, $json, [Text.UTF8Encoding]::new($false))
        $digest = Get-Sprint7AFileSha256 -Path $temporary
        [IO.File]::WriteAllText($temporarySidecar, "$digest`n", [Text.UTF8Encoding]::new($false))
        if ($Overwrite) {
            Remove-Item -LiteralPath $fullPath -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath $sidecar -Force -ErrorAction SilentlyContinue
        }
        Move-Item -LiteralPath $temporary -Destination $fullPath
        Move-Item -LiteralPath $temporarySidecar -Destination $sidecar
        if ((Get-Sprint7AFileSha256 -Path $fullPath) -cne $digest) {
            throw "Published evidence digest changed for '$fullPath'."
        }
        [pscustomobject]@{ path = $fullPath; sha256 = $digest }
    } finally {
        Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $temporarySidecar -Force -ErrorAction SilentlyContinue
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
    } finally {
        Remove-Item -LiteralPath $root -Recurse -Force -ErrorAction SilentlyContinue
    }
}

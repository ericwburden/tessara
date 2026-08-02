Set-StrictMode -Version Latest

$script:Sprint7ARepositoryRoot = Split-Path -Parent $PSScriptRoot
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
}

function Get-Sprint7ASha256 {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Text)
    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        [Convert]::ToHexString(
            $algorithm.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text))
        ).ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
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
        SkipHttpErrorCheck = $true
        UseBasicParsing = $true
    }
    if ($null -ne $Body) {
        $parameters.ContentType = "application/json"
        $parameters.Body = $Body | ConvertTo-Json -Depth 20 -Compress
    }
    $response = Invoke-WebRequest @parameters
    [pscustomobject]@{
        status = [int]$response.StatusCode
        content_type = [string]$response.Headers["Content-Type"]
        body = [string]$response.Content
        body_sha256 = Get-Sprint7ASha256 -Text ([string]$response.Content)
        body_utf8_length = [Text.Encoding]::UTF8.GetByteCount([string]$response.Content)
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
        $digest = (Get-FileHash -LiteralPath $temporary -Algorithm SHA256).Hash.ToLowerInvariant()
        [IO.File]::WriteAllText($temporarySidecar, "$digest`n", [Text.UTF8Encoding]::new($false))
        if ($Overwrite) {
            Remove-Item -LiteralPath $fullPath -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath $sidecar -Force -ErrorAction SilentlyContinue
        }
        Move-Item -LiteralPath $temporary -Destination $fullPath
        Move-Item -LiteralPath $temporarySidecar -Destination $sidecar
        if ((Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant() -cne $digest) {
            throw "Published evidence digest changed for '$fullPath'."
        }
        [pscustomobject]@{ path = $fullPath; sha256 = $digest }
    } finally {
        Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $temporarySidecar -Force -ErrorAction SilentlyContinue
    }
}

function Test-Sprint7AAcceptanceContract {
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

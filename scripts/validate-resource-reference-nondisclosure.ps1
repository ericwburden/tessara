<#
.SYNOPSIS
Validates Sprint 6A resource-reference non-disclosure shape and timing.

.DESCRIPTION
Runs against an already-started, loopback-only Tessara release build backed by
a populated disposable database. The default actors come from the deterministic
demo seed: respondent produces `unauthorized`, while the scoped operator
produces `not_evaluated` for a Form reference.

For each restricted access state, the script warms both a known and a random
identifier, then records at least 200 samples for each identifier in balanced
AB/BA order through one persistent client limited to one connection at a time.
It fails if status or response bytes differ, if any restricted dimension is
disclosed, or if either the median or nearest-rank p95 delta is greater than
max(2 ms, 20% of the faster result).

Passing evidence is fully validated beneath a unique temporary path, then its
JSON and SHA-256 sidecar are moved sequentially within one rollback-safe
publication transaction. This does not claim two-file reader atomicity.
Existing retained evidence is never replaced unless -Overwrite is explicit.

.EXAMPLE
.\scripts\validate-resource-reference-nondisclosure.ps1 `
  -BaseUrl 'http://127.0.0.1:8080' `
  -DeploymentEvidencePath 'artifacts/sprint-6a-ui/deployment-fresh.json' `
  -ExpectedDataState fresh

.EXAMPLE
.\scripts\validate-resource-reference-nondisclosure.ps1 -SelfTest
#>

[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8080",

    [ValidateRange(200, 100000)]
    [int]$SamplesPerIdentifier = 200,

    [ValidateRange(1, 10000)]
    [int]$WarmupPairsPerState = 25,

    [ValidateRange(1, 300)]
    [int]$RequestTimeoutSeconds = 30,

    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$UnauthorizedEmail = "respondent@tessara.local",
    [string]$UnauthorizedPassword = "tessara-dev-respondent",
    [string]$NotEvaluatedEmail = "operator@tessara.local",
    [string]$NotEvaluatedPassword = "tessara-dev-operator",
    [string]$KnownFormId,
    [string]$OutputPath,
    [string]$DeploymentEvidencePath,
    [ValidateSet("fresh")][string]$ExpectedDataState,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$resourceType = "tessara.transition.form"
$resolvePath = "/api/platform/resource-references/resolve"
$minimumDeltaMilliseconds = 2.0
$relativeTolerance = 0.20
$deploymentEvidenceCommon = Join-Path $PSScriptRoot "sprint-6a-deployment-evidence-common.ps1"

if (-not (Test-Path -LiteralPath $deploymentEvidenceCommon -PathType Leaf)) {
    throw "Could not find Sprint 6A deployment evidence validator at $deploymentEvidenceCommon"
}
. $deploymentEvidenceCommon

function Invoke-HttpRequest {
    param(
        [Parameter(Mandatory)]
        [System.Net.Http.HttpClient]$Client,

        [Parameter(Mandatory)]
        [string]$Method,

        [Parameter(Mandatory)]
        [string]$Uri,

        [string]$Token,
        [string]$Body,
        [switch]$Measure
    )

    $request = [System.Net.Http.HttpRequestMessage]::new(
        [System.Net.Http.HttpMethod]::new($Method),
        $Uri
    )
    if (-not [string]::IsNullOrWhiteSpace($Token)) {
        $request.Headers.Authorization = [System.Net.Http.Headers.AuthenticationHeaderValue]::new(
            "Bearer",
            $Token
        )
    }
    if ($null -ne $Body) {
        $request.Content = [System.Net.Http.StringContent]::new(
            $Body,
            [System.Text.Encoding]::UTF8,
            "application/json"
        )
    }

    $stopwatch = $null
    $response = $null
    try {
        if ($Measure) {
            $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        }
        $response = $Client.SendAsync(
            $request,
            [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead
        ).GetAwaiter().GetResult()
        $responseBody = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        if ($null -ne $stopwatch) {
            $stopwatch.Stop()
        }

        [pscustomobject]@{
            StatusCode          = [int]$response.StatusCode
            Body                = $responseBody
            ContentType         = if ($null -eq $response.Content.Headers.ContentType) {
                "<none>"
            } else {
                [string]$response.Content.Headers.ContentType
            }
            Utf8Length          = [Text.Encoding]::UTF8.GetByteCount($responseBody)
            BodySha256          = Get-Sha256 -Text $responseBody
            ElapsedMilliseconds = if ($null -ne $stopwatch) {
                [double]$stopwatch.Elapsed.TotalMilliseconds
            } else {
                $null
            }
        }
    } finally {
        if ($null -ne $response) {
            $response.Dispose()
        }
        $request.Dispose()
    }
}

function Get-HttpResponseDiagnostic {
    param(
        [Parameter(Mandatory)]
        $Response,

        [Parameter(Mandatory)]
        [string]$Label
    )

    $contentType = if ($Response.PSObject.Properties.Name -contains "ContentType") {
        [string]$Response.ContentType
    } else {
        "<unknown>"
    }
    $utf8Length = if ($Response.PSObject.Properties.Name -contains "Utf8Length") {
        [long]$Response.Utf8Length
    } else {
        [long][Text.Encoding]::UTF8.GetByteCount([string]$Response.Body)
    }
    $bodySha256 = if ($Response.PSObject.Properties.Name -contains "BodySha256") {
        [string]$Response.BodySha256
    } else {
        Get-Sha256 -Text ([string]$Response.Body)
    }
    "label='$Label'; status=$([int]$Response.StatusCode); content_type='$contentType'; utf8_length=$utf8Length; sha256=$bodySha256"
}

function ConvertFrom-RequiredJson {
    param(
        [Parameter(Mandatory)]
        $Response,

        [Parameter(Mandatory)]
        [string]$Context
    )

    try {
        return $Response.Body | ConvertFrom-Json
    } catch {
        throw "$Context did not return valid JSON. $(Get-HttpResponseDiagnostic -Response $Response -Label $Context)"
    }
}

function ConvertFrom-ValidatedJsonText {
    param(
        [Parameter(Mandatory)]
        [string]$Text,

        [Parameter(Mandatory)]
        [string]$Context
    )

    try {
        return $Text | ConvertFrom-Json
    } catch {
        $length = [Text.Encoding]::UTF8.GetByteCount($Text)
        $digest = Get-Sha256 -Text $Text
        throw "$Context was not valid JSON; utf8_length=$length; sha256=$digest."
    }
}

function Assert-Status {
    param(
        [Parameter(Mandatory)]
        $Response,

        [Parameter(Mandatory)]
        [int]$Expected,

        [Parameter(Mandatory)]
        [string]$Context
    )

    if ($Response.StatusCode -ne $Expected) {
        throw "$Context returned an unexpected HTTP status; expected=$Expected; $(Get-HttpResponseDiagnostic -Response $Response -Label $Context)"
    }
}

function Invoke-JsonApi {
    param(
        [Parameter(Mandatory)]
        [System.Net.Http.HttpClient]$Client,

        [Parameter(Mandatory)]
        [string]$Method,

        [Parameter(Mandatory)]
        [string]$Uri,

        [string]$Token,
        [object]$Body,
        [int]$ExpectedStatus = 200
    )

    $serializedBody = if ($null -ne $Body) {
        $Body | ConvertTo-Json -Depth 20 -Compress
    } else {
        $null
    }
    $response = Invoke-HttpRequest `
        -Client $Client `
        -Method $Method `
        -Uri $Uri `
        -Token $Token `
        -Body $serializedBody
    Assert-Status -Response $response -Expected $ExpectedStatus -Context "$Method $Uri"
    ConvertFrom-RequiredJson -Response $response -Context "$Method $Uri"
}

function Get-LoginToken {
    param(
        [Parameter(Mandatory)]
        [System.Net.Http.HttpClient]$Client,

        [Parameter(Mandatory)]
        [string]$RootUrl,

        [Parameter(Mandatory)]
        [string]$Email,

        [Parameter(Mandatory)]
        [string]$Password
    )

    $login = Invoke-JsonApi `
        -Client $Client `
        -Method "POST" `
        -Uri "$RootUrl/api/auth/login" `
        -Body @{ email = $Email; password = $Password }
    if ([string]::IsNullOrWhiteSpace([string]$login.token)) {
        throw "Login response for $Email did not include a token."
    }
    [string]$login.token
}

function New-ResolveBody {
    param(
        [Parameter(Mandatory)]
        [string]$InstallationId,

        [Parameter(Mandatory)]
        [string]$ResourceId
    )

    @{
        schema_version = 1
        reference      = @{
            installation_id = $InstallationId
            owner             = @{
                kind            = "core_installation"
                installation_id = $InstallationId
            }
            resource_type     = $resourceType
            resource_id       = $ResourceId
        }
    } | ConvertTo-Json -Depth 10 -Compress
}

function Assert-PropertySet {
    param(
        [Parameter(Mandatory)]
        $Value,

        [Parameter(Mandatory)]
        [string[]]$Expected,

        [Parameter(Mandatory)]
        [string]$Context
    )

    $actualNames = @($Value.PSObject.Properties.Name | Sort-Object)
    $expectedNames = @($Expected | Sort-Object)
    if ($actualNames.Count -ne $expectedNames.Count `
        -or [string]::Join("|", $actualNames) -cne [string]::Join("|", $expectedNames)) {
        throw "$Context property set differed. Expected [$($expectedNames -join ', ')], got [$($actualNames -join ', ')]."
    }
}

function Assert-ExactJsonType {
    param(
        [Parameter(Mandatory)]
        [AllowNull()]
        $Value,

        [Parameter(Mandatory)]
        [type]$ExpectedType,

        [Parameter(Mandatory)]
        [string]$Context
    )

    if ($null -eq $Value -or $Value.GetType() -ne $ExpectedType) {
        $actualType = if ($null -eq $Value) { "null" } else { $Value.GetType().FullName }
        throw "$Context must have exact JSON/CLR type '$($ExpectedType.FullName)', got '$actualType'."
    }
}

function Assert-JsonInteger {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Context
    )

    if ($Value -isnot [int] -and $Value -isnot [long]) {
        $actualType = if ($null -eq $Value) { 'null' } else { $Value.GetType().FullName }
        throw "$Context must be a JSON integer, got '$actualType'."
    }
}

function Assert-JsonNumber {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Context
    )

    if ($Value -isnot [int] -and $Value -isnot [long] -and
        $Value -isnot [double] -and $Value -isnot [decimal]) {
        $actualType = if ($null -eq $Value) { 'null' } else { $Value.GetType().FullName }
        throw "$Context must be a JSON number, got '$actualType'."
    }
}

function Assert-CanonicalLowercaseUuid {
    param(
        [Parameter(Mandatory)]
        $Value,

        [Parameter(Mandatory)]
        [string]$Context
    )

    Assert-ExactJsonType -Value $Value -ExpectedType ([string]) -Context $Context
    $parsed = [guid]::Empty
    if (-not [guid]::TryParseExact([string]$Value, "D", [ref]$parsed) `
        -or [string]$Value -cne $parsed.ToString("D")) {
        throw "$Context must be a canonical lowercase hyphenated UUID string."
    }
}

function Assert-NoReparsePointInPathChain {
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [Parameter(Mandatory)]
        [string]$Context
    )

    $current = [IO.Path]::GetFullPath($Path)
    while ($null -ne $current) {
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force -ErrorAction Stop
            $isReparsePoint = ([IO.FileAttributes]$item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
            $hasLinkType = $item.PSObject.Properties.Name -contains "LinkType" `
                -and -not [string]::IsNullOrWhiteSpace([string]$item.LinkType)
            if ($isReparsePoint -or $hasLinkType) {
                throw "$Context rejects reparse points, junctions, and symbolic links in the existing path chain at '$current'."
            }
        }
        $parent = [IO.Directory]::GetParent($current)
        $current = if ($null -eq $parent) { $null } else { $parent.FullName }
    }
}

function Assert-NondisclosurePathSetSafety {
    param(
        [Parameter(Mandatory)]
        [string[]]$Paths,

        [Parameter(Mandatory)]
        [string]$Context
    )

    $distinctPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($path in $Paths) {
        if ([string]::IsNullOrWhiteSpace($path)) {
            throw "$Context contains an empty path."
        }
        $fullPath = [IO.Path]::GetFullPath($path)
        if (-not $distinctPaths.Add($fullPath)) {
            throw "$Context paths must be pairwise distinct; collision at '$fullPath'."
        }
        Assert-NoReparsePointInPathChain -Path $fullPath -Context $Context
    }
}

function Remove-PathVerified {
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [switch]$Recurse,

        [Parameter(Mandatory)]
        [string]$Context
    )

    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Force -Recurse:$Recurse -ErrorAction Stop
    }
    if (Test-Path -LiteralPath $Path) {
        throw "$Context cleanup did not remove '$Path'."
    }
}

function Assert-RestrictedEnvelope {
    param(
        [Parameter(Mandatory)]
        $Response,

        [Parameter(Mandatory)]
        [ValidateSet("unauthorized", "not_evaluated")]
        [string]$ExpectedAccessState,

        [Parameter(Mandatory)]
        [string]$Context
    )

    Assert-Status -Response $Response -Expected 200 -Context $Context
    try {
        $envelope = ConvertFrom-RequiredJson -Response $Response -Context $Context
        Assert-PropertySet `
            -Value $envelope `
            -Expected @(
                "schema_version",
                "access_state",
                "owner_state",
                "resource_identity_state",
                "resource_lifecycle_state",
                "compatibility_state",
                "availability_state"
            ) `
            -Context $Context

        Assert-PropertySet -Value $envelope.owner_state -Expected @("kind") -Context "$Context owner_state"
        Assert-PropertySet `
            -Value $envelope.resource_lifecycle_state `
            -Expected @("kind") `
            -Context "$Context resource_lifecycle_state"

        if ($envelope.schema_version -ne 1 `
            -or $envelope.access_state -cne $ExpectedAccessState `
            -or $envelope.owner_state.kind -cne "undisclosed" `
            -or $envelope.resource_identity_state -cne "undisclosed" `
            -or $envelope.resource_lifecycle_state.kind -cne "undisclosed" `
            -or $envelope.compatibility_state -cne "undisclosed" `
            -or $envelope.availability_state -cne "undisclosed") {
            throw "restricted envelope mismatch"
        }
    } catch {
        throw "$Context was not the exact schema-v1 restricted envelope. $(Get-HttpResponseDiagnostic -Response $Response -Label $Context)"
    }
}

function Assert-AuthorizedEnvelope {
    param(
        [Parameter(Mandatory)]
        $Response,

        [Parameter(Mandatory)]
        [ValidateSet("resolved", "unknown_resource")]
        [string]$ExpectedIdentity,

        [Parameter(Mandatory)]
        [string]$Context
    )

    Assert-Status -Response $Response -Expected 200 -Context $Context
    $envelope = ConvertFrom-RequiredJson -Response $Response -Context $Context
    if ($envelope.schema_version -ne 1 `
        -or $envelope.access_state -cne "authorized" `
        -or $envelope.resource_identity_state -cne $ExpectedIdentity) {
        throw "$Context did not prove the expected authorized resource identity. $(Get-HttpResponseDiagnostic -Response $Response -Label $Context)"
    }
}

function Assert-ExactRestrictedResponse {
    param(
        [Parameter(Mandatory)]
        $Response,

        [Parameter(Mandatory)]
        [string]$ExpectedBody,

        [Parameter(Mandatory)]
        [string]$Context
    )

    Assert-Status -Response $Response -Expected 200 -Context $Context
    if ($Response.Body -cne $ExpectedBody) {
        $expectedLength = [Text.Encoding]::UTF8.GetByteCount($ExpectedBody)
        $expectedDigest = Get-Sha256 -Text $ExpectedBody
        throw "$Context changed the restricted response bytes; expected_utf8_length=$expectedLength; expected_sha256=$expectedDigest; $(Get-HttpResponseDiagnostic -Response $Response -Label $Context)"
    }
}

function Get-Median {
    param(
        [Parameter(Mandatory)]
        [double[]]$Samples
    )

    if ($Samples.Count -eq 0) {
        throw "Median requires at least one sample."
    }
    [double[]]$sorted = @($Samples | Sort-Object)
    $middle = [int][Math]::Floor($sorted.Count / 2.0)
    if (($sorted.Count % 2) -eq 1) {
        return [double]$sorted[$middle]
    }
    ([double]$sorted[$middle - 1] + [double]$sorted[$middle]) / 2.0
}

function Get-NearestRankPercentile {
    param(
        [Parameter(Mandatory)]
        [double[]]$Samples,

        [Parameter(Mandatory)]
        [ValidateRange(0.0, 1.0)]
        [double]$Percentile
    )

    if ($Samples.Count -eq 0) {
        throw "Percentile requires at least one sample."
    }
    [double[]]$sorted = @($Samples | Sort-Object)
    $rank = [int][Math]::Ceiling($Percentile * $sorted.Count)
    $index = [Math]::Max(0, $rank - 1)
    [double]$sorted[$index]
}

function New-MetricComparison {
    param(
        [Parameter(Mandatory)]
        [double]$KnownMilliseconds,

        [Parameter(Mandatory)]
        [double]$RandomMilliseconds
    )

    $faster = [Math]::Min($KnownMilliseconds, $RandomMilliseconds)
    $delta = [Math]::Abs($KnownMilliseconds - $RandomMilliseconds)
    $tolerance = [Math]::Max(
        $minimumDeltaMilliseconds,
        $relativeTolerance * $faster
    )
    $relativeDeltaPercent = if ($faster -gt 0.0) {
        100.0 * $delta / $faster
    } else {
        $null
    }

    [pscustomobject]@{
        known_ms              = [Math]::Round($KnownMilliseconds, 6)
        random_ms             = [Math]::Round($RandomMilliseconds, 6)
        delta_ms              = [Math]::Round($delta, 6)
        relative_delta_percent = if ($null -eq $relativeDeltaPercent) {
            $null
        } else {
            [Math]::Round($relativeDeltaPercent, 6)
        }
        tolerance_ms          = [Math]::Round($tolerance, 6)
        passed                = $delta -le $tolerance
    }
}

function Get-Sha256 {
    param(
        [Parameter(Mandatory)]
        [string]$Text
    )

    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $algorithm.ComputeHash($bytes)
        ([System.BitConverter]::ToString($hash)).Replace("-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Get-FileSha256 {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-NondisclosureMetric {
    param(
        [Parameter(Mandatory)]
        $Metric,

        [Parameter(Mandatory)]
        [string]$Context
    )

    Assert-ExactJsonType `
        -Value $Metric `
        -ExpectedType ([Management.Automation.PSCustomObject]) `
        -Context $Context
    Assert-PropertySet `
        -Value $Metric `
        -Expected @(
            "known_ms",
            "random_ms",
            "delta_ms",
            "relative_delta_percent",
            "tolerance_ms",
            "passed"
        ) `
        -Context $Context

    Assert-ExactJsonType -Value $Metric.passed -ExpectedType ([bool]) -Context "$Context passed"

    foreach ($name in @("known_ms", "random_ms", "delta_ms", "tolerance_ms")) {
        Assert-JsonNumber -Value $Metric.$name -Context "$Context $name"
        $value = [double]$Metric.$name
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value) -or $value -lt 0.0) {
            throw "$Context $name must be a finite non-negative number."
        }
    }
    if ($null -ne $Metric.relative_delta_percent) {
        Assert-JsonNumber `
            -Value $Metric.relative_delta_percent `
            -Context "$Context relative_delta_percent"
        $relativeDelta = [double]$Metric.relative_delta_percent
        if ([double]::IsNaN($relativeDelta) -or [double]::IsInfinity($relativeDelta) -or $relativeDelta -lt 0.0) {
            throw "$Context relative_delta_percent must be null or a finite non-negative number."
        }
    }

    $known = [double]$Metric.known_ms
    $random = [double]$Metric.random_ms
    $expectedDelta = [Math]::Round([Math]::Abs($known - $random), 6)
    $faster = [Math]::Min($known, $random)
    $expectedTolerance = [Math]::Round(
        [Math]::Max($minimumDeltaMilliseconds, $relativeTolerance * $faster),
        6
    )
    $expectedRelativeDelta = if ($faster -gt 0.0) {
        [Math]::Round(100.0 * [Math]::Abs($known - $random) / $faster, 6)
    } else {
        $null
    }
    if ([Math]::Abs(([double]$Metric.delta_ms) - $expectedDelta) -gt 0.000002 `
        -or [Math]::Abs(([double]$Metric.tolerance_ms) - $expectedTolerance) -gt 0.000002) {
        throw "$Context retained inconsistent timing delta/tolerance claims."
    }
    if ($null -eq $expectedRelativeDelta) {
        if ($null -ne $Metric.relative_delta_percent) {
            throw "$Context relative_delta_percent must be null when the faster statistic is zero."
        }
    } else {
        $deltaLower = [Math]::Max(0.0, [Math]::Abs($known - $random) - 0.000001)
        $deltaUpper = [Math]::Abs($known - $random) + 0.000001
        $fasterLower = [Math]::Max(0.0, $faster - 0.0000005)
        $fasterUpper = $faster + 0.0000005
        $relativeLower = 100.0 * $deltaLower / $fasterUpper
        $relativeUpper = if ($fasterLower -gt 0.0) {
            100.0 * $deltaUpper / $fasterLower
        } else {
            [double]::PositiveInfinity
        }
        $retainedRelative = if ($null -eq $Metric.relative_delta_percent) {
            [double]::NaN
        } else {
            [double]$Metric.relative_delta_percent
        }
        if ([double]::IsNaN($retainedRelative) `
            -or $retainedRelative -lt ($relativeLower - 0.000001) `
            -or $retainedRelative -gt ($relativeUpper + 0.000001)) {
            throw "$Context retained an inconsistent relative timing delta claim."
        }
    }

    if ([bool]$Metric.passed -and ([double]$Metric.delta_ms) -gt (([double]$Metric.tolerance_ms) + 0.000002)) {
        throw "$Context passed did not match its retained timing values."
    }
}

function Assert-NondisclosureEvidenceDocument {
    param(
        [Parameter(Mandatory)]
        $Document,

        [Parameter(Mandatory)]
        [string]$ExpectedBaseUrl,

        [Parameter(Mandatory)]
        [string]$ExpectedDeploymentEvidencePath,

        [Parameter(Mandatory)]
        [string]$ExpectedDeploymentEvidenceSha256,

        [Parameter(Mandatory)]
        [ValidateSet("fresh")]
        [string]$ExpectedDeploymentDataState,

        [Parameter(Mandatory)]
        [string]$ExpectedReleaseImageId,

        [Parameter(Mandatory)]
        [string]$ExpectedSourceCommit,

        [Parameter(Mandatory)]
        [string]$ExpectedDatabaseName,

        [Parameter(Mandatory)]
        [string]$ExpectedUnauthorizedActor,

        [Parameter(Mandatory)]
        [string]$ExpectedNotEvaluatedActor,

        [Parameter(Mandatory)]
        [int]$ExpectedWarmupPairsPerState,

        [Parameter(Mandatory)]
        [int]$ExpectedSamplesPerIdentifier,

        [Parameter(Mandatory)]
        [string]$ExpectedInstallationId,

        [Parameter(Mandatory)]
        [string]$ExpectedKnownResourceId,

        [Parameter(Mandatory)]
        [string]$ExpectedRandomResourceId,

        [Parameter(Mandatory)]
        [hashtable]$ExpectedRestrictedBodySha256ByAccessState
    )

    Assert-ExactJsonType `
        -Value $Document `
        -ExpectedType ([Management.Automation.PSCustomObject]) `
        -Context "nondisclosure evidence"
    Assert-PropertySet `
        -Value $Document `
        -Expected @("schema_version", "evidence_kind", "generated_at", "passed", "environment", "fixture", "methodology", "states") `
        -Context "nondisclosure evidence"
    Assert-JsonInteger -Value $Document.schema_version -Context "nondisclosure evidence schema_version"
    Assert-ExactJsonType -Value $Document.evidence_kind -ExpectedType ([string]) -Context "nondisclosure evidence evidence_kind"
    Assert-ExactJsonType -Value $Document.generated_at -ExpectedType ([string]) -Context "nondisclosure evidence generated_at"
    Assert-ExactJsonType -Value $Document.passed -ExpectedType ([bool]) -Context "nondisclosure evidence passed"
    if ($Document.schema_version -ne 1 `
        -or $Document.evidence_kind -cne "tessara.sprint-6a.resource-reference-nondisclosure" `
        -or -not $Document.passed) {
        throw "Nondisclosure evidence must be a passing schema-v1 report."
    }
    $generatedAt = [DateTimeOffset]::MinValue
    $timestampFormat = "yyyy-MM-dd'T'HH:mm:ss.fffffffzzz"
    if (-not [DateTimeOffset]::TryParseExact(
        $Document.generated_at,
        $timestampFormat,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::None,
        [ref]$generatedAt
    ) `
        -or $generatedAt.Offset -ne [TimeSpan]::Zero `
        -or $generatedAt.ToString($timestampFormat, [Globalization.CultureInfo]::InvariantCulture) -cne $Document.generated_at) {
        throw "Nondisclosure evidence generated_at must be an exact round-trip UTC timestamp with +00:00 offset."
    }

    Assert-ExactJsonType `
        -Value $Document.environment `
        -ExpectedType ([Management.Automation.PSCustomObject]) `
        -Context "nondisclosure evidence environment"
    Assert-PropertySet `
        -Value $Document.environment `
        -Expected @(
            "base_url",
            "deployment_evidence_path",
            "deployment_evidence_sha256",
            "deployment_data_state",
            "release_image_id",
            "source_commit",
            "database_name",
            "powershell_version",
            "dotnet_version",
            "os"
        ) `
        -Context "nondisclosure evidence environment"
    $environment = $Document.environment
    foreach ($name in @(
        "base_url",
        "deployment_evidence_path",
        "deployment_evidence_sha256",
        "deployment_data_state",
        "release_image_id",
        "source_commit",
        "database_name",
        "powershell_version",
        "dotnet_version",
        "os"
    )) {
        Assert-ExactJsonType -Value $environment.$name -ExpectedType ([string]) -Context "nondisclosure evidence environment.$name"
    }
    if ($environment.base_url -cne $ExpectedBaseUrl `
        -or $environment.deployment_evidence_path -cne [IO.Path]::GetFullPath($ExpectedDeploymentEvidencePath) `
        -or $environment.deployment_evidence_sha256 -cne $ExpectedDeploymentEvidenceSha256 `
        -or $environment.deployment_data_state -cne $ExpectedDeploymentDataState `
        -or $environment.release_image_id -cne $ExpectedReleaseImageId `
        -or $environment.source_commit -cne $ExpectedSourceCommit `
        -or $environment.database_name -cne $ExpectedDatabaseName) {
        throw "Nondisclosure evidence environment claims do not match the validated deployment."
    }
    if ($environment.deployment_evidence_sha256 -cnotmatch '^[0-9a-f]{64}$' `
        -or $environment.release_image_id -cnotmatch '^sha256:[0-9a-f]{64}$' `
        -or $environment.source_commit -cnotmatch '^[0-9a-f]{40,64}$' `
        -or [string]::IsNullOrWhiteSpace($environment.database_name) `
        -or [string]::IsNullOrWhiteSpace($environment.powershell_version) `
        -or [string]::IsNullOrWhiteSpace($environment.dotnet_version) `
        -or [string]::IsNullOrWhiteSpace($environment.os)) {
        throw "Nondisclosure evidence environment contains malformed identity/runtime claims."
    }

    Assert-ExactJsonType `
        -Value $Document.fixture `
        -ExpectedType ([Management.Automation.PSCustomObject]) `
        -Context "nondisclosure evidence fixture"
    Assert-PropertySet `
        -Value $Document.fixture `
        -Expected @(
            "installation_id",
            "resource_type",
            "known_resource_id",
            "random_resource_id",
            "unauthorized_actor",
            "not_evaluated_actor"
        ) `
        -Context "nondisclosure evidence fixture"
    $fixture = $Document.fixture
    foreach ($name in @(
        "installation_id",
        "resource_type",
        "known_resource_id",
        "random_resource_id",
        "unauthorized_actor",
        "not_evaluated_actor"
    )) {
        Assert-ExactJsonType -Value $fixture.$name -ExpectedType ([string]) -Context "nondisclosure evidence fixture.$name"
    }
    Assert-CanonicalLowercaseUuid -Value $fixture.installation_id -Context "nondisclosure evidence fixture.installation_id"
    Assert-CanonicalLowercaseUuid -Value $fixture.known_resource_id -Context "nondisclosure evidence fixture.known_resource_id"
    Assert-CanonicalLowercaseUuid -Value $fixture.random_resource_id -Context "nondisclosure evidence fixture.random_resource_id"
    Assert-CanonicalLowercaseUuid -Value $ExpectedInstallationId -Context "expected live installation id"
    Assert-CanonicalLowercaseUuid -Value $ExpectedKnownResourceId -Context "expected live known resource id"
    Assert-CanonicalLowercaseUuid -Value $ExpectedRandomResourceId -Context "expected live random resource id"
    if ($fixture.known_resource_id -ceq $fixture.random_resource_id `
        -or $fixture.installation_id -cne $ExpectedInstallationId `
        -or $fixture.known_resource_id -cne $ExpectedKnownResourceId `
        -or $fixture.random_resource_id -cne $ExpectedRandomResourceId `
        -or $fixture.resource_type -cne $resourceType `
        -or $fixture.unauthorized_actor -cne $ExpectedUnauthorizedActor `
        -or $fixture.not_evaluated_actor -cne $ExpectedNotEvaluatedActor) {
        throw "Nondisclosure evidence fixture claims are invalid."
    }

    Assert-ExactJsonType `
        -Value $Document.methodology `
        -ExpectedType ([Management.Automation.PSCustomObject]) `
        -Context "nondisclosure evidence methodology"
    Assert-PropertySet `
        -Value $Document.methodology `
        -Expected @(
            "endpoint",
            "transport",
            "measurement_boundary",
            "request_order",
            "warmup_pairs_per_state",
            "samples_per_identifier_per_state",
            "percentile",
            "relative_baseline",
            "pass_tolerance",
            "retries"
        ) `
        -Context "nondisclosure evidence methodology"
    $methodology = $Document.methodology
    foreach ($name in @("endpoint", "transport", "measurement_boundary", "request_order", "percentile", "relative_baseline", "pass_tolerance")) {
        Assert-ExactJsonType -Value $methodology.$name -ExpectedType ([string]) -Context "nondisclosure evidence methodology.$name"
    }
    foreach ($name in @("warmup_pairs_per_state", "samples_per_identifier_per_state", "retries")) {
        Assert-JsonInteger -Value $methodology.$name -Context "nondisclosure evidence methodology.$name"
    }
    if ($methodology.endpoint -cne $resolvePath `
        -or $methodology.transport -cne "one persistent loopback HttpClient; one connection at a time; proxy and cookies disabled" `
        -or $methodology.measurement_boundary -cne "SendAsync through complete response-body read; request construction and assertions excluded" `
        -or $methodology.request_order -cne "balanced AB/BA known-random pairs" `
        -or $methodology.warmup_pairs_per_state -ne $ExpectedWarmupPairsPerState `
        -or $methodology.samples_per_identifier_per_state -ne $ExpectedSamplesPerIdentifier `
        -or $methodology.percentile -cne "nearest-rank p95" `
        -or $methodology.relative_baseline -cne "faster (lower) known/random statistic" `
        -or $methodology.pass_tolerance -cne "delta <= max(2 ms, 20% of faster statistic), independently for median and p95" `
        -or $methodology.retries -ne 0) {
        throw "Nondisclosure evidence methodology differs from the fixed Sprint 6A protocol."
    }

    Assert-ExactJsonType -Value $Document.states -ExpectedType ([object[]]) -Context "nondisclosure evidence states"
    $expectedDigestKeys = @($ExpectedRestrictedBodySha256ByAccessState.Keys | Sort-Object)
    if ($expectedDigestKeys.Count -ne 2 `
        -or [string]::Join("|", $expectedDigestKeys) -cne "not_evaluated|unauthorized" `
        -or @($ExpectedRestrictedBodySha256ByAccessState.Values | Where-Object { [string]$_ -cnotmatch '^[0-9a-f]{64}$' }).Count -ne 0) {
        throw "Expected live restricted-body digests must contain exactly unauthorized and not_evaluated lowercase SHA-256 values."
    }

    $states = @($Document.states)
    if ($states.Count -ne 2) {
        throw "Nondisclosure evidence must contain exactly unauthorized and not_evaluated results."
    }
    $seenStates = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($state in $states) {
        Assert-ExactJsonType `
            -Value $state `
            -ExpectedType ([Management.Automation.PSCustomObject]) `
            -Context "nondisclosure evidence state"
        Assert-PropertySet `
            -Value $state `
            -Expected @(
                "access_state",
                "response_status",
                "exact_known_random_body_match",
                "restricted_envelope",
                "restricted_body_sha256",
                "known_sample_count",
                "random_sample_count",
                "median",
                "p95",
                "passed"
            ) `
            -Context "nondisclosure evidence state"
        Assert-ExactJsonType -Value $state.access_state -ExpectedType ([string]) -Context "nondisclosure evidence state access_state"
        Assert-JsonInteger -Value $state.response_status -Context "nondisclosure evidence state response_status"
        Assert-ExactJsonType -Value $state.exact_known_random_body_match -ExpectedType ([bool]) -Context "nondisclosure evidence state exact_known_random_body_match"
        Assert-ExactJsonType -Value $state.restricted_body_sha256 -ExpectedType ([string]) -Context "nondisclosure evidence state restricted_body_sha256"
        Assert-JsonInteger -Value $state.known_sample_count -Context "nondisclosure evidence state known_sample_count"
        Assert-JsonInteger -Value $state.random_sample_count -Context "nondisclosure evidence state random_sample_count"
        Assert-ExactJsonType -Value $state.passed -ExpectedType ([bool]) -Context "nondisclosure evidence state passed"
        $accessState = $state.access_state
        if ($accessState -notin @("unauthorized", "not_evaluated") -or -not $seenStates.Add($accessState)) {
            throw "Nondisclosure evidence contains a missing, repeated, or unknown restricted state."
        }
        if ($state.response_status -ne 200 `
            -or -not $state.exact_known_random_body_match `
            -or $state.restricted_body_sha256 -cnotmatch '^[0-9a-f]{64}$' `
            -or -not $ExpectedRestrictedBodySha256ByAccessState.ContainsKey($accessState) `
            -or [string]$ExpectedRestrictedBodySha256ByAccessState[$accessState] -cne $state.restricted_body_sha256 `
            -or $state.known_sample_count -ne $ExpectedSamplesPerIdentifier `
            -or $state.random_sample_count -ne $ExpectedSamplesPerIdentifier) {
            throw "Nondisclosure evidence state '$accessState' has incomplete response/sample claims."
        }
        $envelope = $state.restricted_envelope
        Assert-ExactJsonType `
            -Value $envelope `
            -ExpectedType ([Management.Automation.PSCustomObject]) `
            -Context "nondisclosure evidence state '$accessState' envelope"
        Assert-PropertySet `
            -Value $envelope `
            -Expected @(
                "schema_version",
                "access_state",
                "owner_state",
                "resource_identity_state",
                "resource_lifecycle_state",
                "compatibility_state",
                "availability_state"
            ) `
            -Context "nondisclosure evidence state '$accessState' envelope"
        Assert-PropertySet -Value $envelope.owner_state -Expected @("kind") -Context "nondisclosure evidence owner_state"
        Assert-PropertySet -Value $envelope.resource_lifecycle_state -Expected @("kind") -Context "nondisclosure evidence lifecycle_state"
        Assert-JsonInteger -Value $envelope.schema_version -Context "restricted envelope schema_version"
        Assert-ExactJsonType -Value $envelope.access_state -ExpectedType ([string]) -Context "restricted envelope access_state"
        Assert-ExactJsonType -Value $envelope.owner_state -ExpectedType ([Management.Automation.PSCustomObject]) -Context "restricted envelope owner_state"
        Assert-ExactJsonType -Value $envelope.owner_state.kind -ExpectedType ([string]) -Context "restricted envelope owner_state.kind"
        Assert-ExactJsonType -Value $envelope.resource_identity_state -ExpectedType ([string]) -Context "restricted envelope resource_identity_state"
        Assert-ExactJsonType -Value $envelope.resource_lifecycle_state -ExpectedType ([Management.Automation.PSCustomObject]) -Context "restricted envelope resource_lifecycle_state"
        Assert-ExactJsonType -Value $envelope.resource_lifecycle_state.kind -ExpectedType ([string]) -Context "restricted envelope resource_lifecycle_state.kind"
        Assert-ExactJsonType -Value $envelope.compatibility_state -ExpectedType ([string]) -Context "restricted envelope compatibility_state"
        Assert-ExactJsonType -Value $envelope.availability_state -ExpectedType ([string]) -Context "restricted envelope availability_state"
        if ($envelope.schema_version -ne 1 `
            -or $envelope.access_state -cne $accessState `
            -or $envelope.owner_state.kind -cne "undisclosed" `
            -or $envelope.resource_identity_state -cne "undisclosed" `
            -or $envelope.resource_lifecycle_state.kind -cne "undisclosed" `
            -or $envelope.compatibility_state -cne "undisclosed" `
            -or $envelope.availability_state -cne "undisclosed") {
            throw "Nondisclosure evidence state '$accessState' did not retain the exact restricted envelope."
        }
        $retainedEnvelopeJson = $envelope | ConvertTo-Json -Depth 10 -Compress
        if ((Get-Sha256 -Text $retainedEnvelopeJson) -cne [string]$state.restricted_body_sha256) {
            throw "Nondisclosure evidence state '$accessState' restricted-body digest does not match its exact retained envelope bytes."
        }
        Assert-NondisclosureMetric -Metric $state.median -Context "nondisclosure evidence state '$accessState' median"
        Assert-NondisclosureMetric -Metric $state.p95 -Context "nondisclosure evidence state '$accessState' p95"
        if (-not [bool]$state.median.passed -or -not [bool]$state.p95.passed -or -not [bool]$state.passed) {
            throw "Nondisclosure evidence state '$accessState' is not a passing result."
        }
    }
    if (-not $seenStates.Contains("unauthorized") -or -not $seenStates.Contains("not_evaluated")) {
        throw "Nondisclosure evidence is missing a required restricted state."
    }
}

function Assert-NondisclosureEvidenceHashPair {
    param(
        [Parameter(Mandatory)]
        [string]$EvidencePath,

        [Parameter(Mandatory)]
        [string]$DigestPath
    )

    if (-not (Test-Path -LiteralPath $EvidencePath -PathType Leaf) `
        -or -not (Test-Path -LiteralPath $DigestPath -PathType Leaf)) {
        throw "Nondisclosure evidence JSON and SHA-256 sidecar must both exist."
    }
    [byte[]]$digestBytes = [IO.File]::ReadAllBytes($DigestPath)
    if ($digestBytes.Length -ne 65 -or $digestBytes[64] -ne 10) {
        throw "Nondisclosure evidence SHA-256 sidecar must contain exactly 64 lowercase hexadecimal ASCII bytes followed by one LF byte."
    }
    $retainedDigest = [Text.Encoding]::ASCII.GetString($digestBytes, 0, 64)
    $actualDigest = Get-FileSha256 -Path $EvidencePath
    if ($retainedDigest -cnotmatch '^[0-9a-f]{64}$' -or $retainedDigest -cne $actualDigest) {
        throw "Nondisclosure evidence SHA-256 sidecar does not match the JSON bytes."
    }
}

function Publish-NondisclosureEvidencePair {
    param(
        [Parameter(Mandatory)][string]$TemporaryEvidencePath,
        [Parameter(Mandatory)][string]$TemporaryDigestPath,
        [Parameter(Mandatory)][string]$TemporaryRootPath,
        [Parameter(Mandatory)][string]$FinalEvidencePath,
        [Parameter(Mandatory)][string]$FinalDigestPath,
        [Parameter(Mandatory)][string[]]$ProtectedInputPaths,
        [switch]$AllowOverwrite,
        [ValidateSet("None", "AfterFirstFinalMove", "FormerOuterHash", "BackupCleanup")]
        [string]$InjectFailurePoint = "None"
    )

    foreach ($temporaryPath in @($TemporaryEvidencePath, $TemporaryDigestPath)) {
        if (-not (Test-Path -LiteralPath $temporaryPath -PathType Leaf)) {
            throw "Validated nondisclosure artifact '$temporaryPath' is missing before publication."
        }
    }
    Assert-NondisclosureEvidenceHashPair `
        -EvidencePath $TemporaryEvidencePath `
        -DigestPath $TemporaryDigestPath

    $states = @(
        [pscustomobject]@{
            TemporaryPath = [IO.Path]::GetFullPath($TemporaryEvidencePath)
            FinalPath = [IO.Path]::GetFullPath($FinalEvidencePath)
            BackupPath = "$([IO.Path]::GetFullPath($FinalEvidencePath)).backup-$([guid]::NewGuid().ToString('N'))"
            FinalExisted = Test-Path -LiteralPath $FinalEvidencePath -PathType Leaf
            OriginalBytes = if (Test-Path -LiteralPath $FinalEvidencePath -PathType Leaf) {
                [IO.File]::ReadAllBytes($FinalEvidencePath)
            } else {
                $null
            }
            BackedUp = $false
            Published = $false
        },
        [pscustomobject]@{
            TemporaryPath = [IO.Path]::GetFullPath($TemporaryDigestPath)
            FinalPath = [IO.Path]::GetFullPath($FinalDigestPath)
            BackupPath = "$([IO.Path]::GetFullPath($FinalDigestPath)).backup-$([guid]::NewGuid().ToString('N'))"
            FinalExisted = Test-Path -LiteralPath $FinalDigestPath -PathType Leaf
            OriginalBytes = if (Test-Path -LiteralPath $FinalDigestPath -PathType Leaf) {
                [IO.File]::ReadAllBytes($FinalDigestPath)
            } else {
                $null
            }
            BackedUp = $false
            Published = $false
        }
    )
    if (@($states | Where-Object FinalExisted).Count -gt 0 -and -not $AllowOverwrite) {
        throw "Retained nondisclosure evidence already exists. Refusing to replace the JSON/sidecar set without -Overwrite."
    }

    $transactionPaths = @(
        $TemporaryRootPath,
        $TemporaryEvidencePath,
        $TemporaryDigestPath,
        $FinalEvidencePath,
        $FinalDigestPath,
        $states[0].BackupPath,
        $states[1].BackupPath
    ) + @($ProtectedInputPaths)
    # This is the final path-chain/alias check immediately before the first
    # publication mutation. No output/input/temp/backup path may traverse a
    # symlink, junction, or other reparse point.
    Assert-NondisclosurePathSetSafety `
        -Paths $transactionPaths `
        -Context "nondisclosure publication"

    $publishedCount = 0
    $preparedResult = $null
    $rollbackTemporaryPaths = [Collections.Generic.List[string]]::new()
    try {
        foreach ($state in $states) {
            if ($state.FinalExisted) {
                [IO.File]::Move($state.FinalPath, $state.BackupPath)
                $state.BackedUp = $true
            }
        }
        foreach ($state in $states) {
            [IO.File]::Move($state.TemporaryPath, $state.FinalPath)
            $state.Published = $true
            $publishedCount += 1
            if ($InjectFailurePoint -ceq "AfterFirstFinalMove" -and $publishedCount -eq 1) {
                throw "Injected nondisclosure evidence partial-publication failure after artifact $publishedCount."
            }
        }
        Assert-NondisclosureEvidenceHashPair `
            -EvidencePath $FinalEvidencePath `
            -DigestPath $FinalDigestPath
        $finalSha256 = Get-FileSha256 -Path $FinalEvidencePath
        $preparedResult = [pscustomobject]@{
            EvidencePath = [IO.Path]::GetFullPath($FinalEvidencePath)
            DigestPath = [IO.Path]::GetFullPath($FinalDigestPath)
            Sha256 = $finalSha256
        }
        if ($InjectFailurePoint -ceq "FormerOuterHash") {
            throw "Injected failure at the former post-publication outer hash/result point."
        }
        $cleanedBackupCount = 0
        foreach ($state in $states) {
            Remove-PathVerified `
                -Path $state.BackupPath `
                -Context "nondisclosure committed-backup"
            $cleanedBackupCount += 1
            if ($InjectFailurePoint -ceq "BackupCleanup" -and $cleanedBackupCount -eq 1) {
                throw "Injected nondisclosure evidence failure after the first backup cleanup."
            }
        }
        Remove-PathVerified `
            -Path $TemporaryRootPath `
            -Recurse `
            -Context "nondisclosure committed-temporary-root"
    } catch {
        $publicationFailure = $_.Exception.Message
        $restoreFailures = [Collections.Generic.List[string]]::new()
        for ($index = $states.Count - 1; $index -ge 0; $index--) {
            $state = $states[$index]
            if ($state.Published -and (Test-Path -LiteralPath $state.FinalPath)) {
                try {
                    Remove-PathVerified `
                        -Path $state.FinalPath `
                        -Context "nondisclosure rollback replacement"
                } catch {
                    $restoreFailures.Add("could not remove new '$($state.FinalPath)': $($_.Exception.Message)")
                }
            }
        }
        for ($index = $states.Count - 1; $index -ge 0; $index--) {
            $state = $states[$index]
            if ($state.FinalExisted) {
                try {
                    if (Test-Path -LiteralPath $state.FinalPath) {
                        Remove-PathVerified `
                            -Path $state.FinalPath `
                            -Context "nondisclosure rollback conflicting final"
                    }
                    if ($state.BackedUp -and (Test-Path -LiteralPath $state.BackupPath -PathType Leaf)) {
                        [IO.File]::Move($state.BackupPath, $state.FinalPath)
                        $state.BackedUp = $false
                    } else {
                        $restoreTemporaryPath = "$($state.FinalPath).restore-$([guid]::NewGuid().ToString('N'))"
                        $rollbackTemporaryPaths.Add($restoreTemporaryPath)
                        Assert-NondisclosurePathSetSafety `
                            -Paths (@($restoreTemporaryPath) + @($ProtectedInputPaths)) `
                            -Context "nondisclosure rollback restoration"
                        [IO.File]::WriteAllBytes($restoreTemporaryPath, [byte[]]$state.OriginalBytes)
                        [IO.File]::Move($restoreTemporaryPath, $state.FinalPath)
                    }
                    if (-not [Linq.Enumerable]::SequenceEqual(
                        [byte[]]$state.OriginalBytes,
                        [IO.File]::ReadAllBytes($state.FinalPath)
                    )) {
                        throw "restored bytes differ from the byte-for-byte prior artifact"
                    }
                } catch {
                    $restoreFailures.Add("could not restore prior '$($state.FinalPath)': $($_.Exception.Message)")
                }
            } elseif (Test-Path -LiteralPath $state.FinalPath) {
                $restoreFailures.Add("new final '$($state.FinalPath)' remained after rollback")
            }
        }
        foreach ($state in $states) {
            try {
                Remove-PathVerified `
                    -Path $state.BackupPath `
                    -Context "nondisclosure rollback-backup"
            } catch {
                $restoreFailures.Add($_.Exception.Message)
            }
        }
        foreach ($rollbackTemporaryPath in $rollbackTemporaryPaths) {
            try {
                Remove-PathVerified `
                    -Path $rollbackTemporaryPath `
                    -Context "nondisclosure rollback-restoration-temporary"
            } catch {
                $restoreFailures.Add($_.Exception.Message)
            }
        }
        try {
            Remove-PathVerified `
                -Path $TemporaryRootPath `
                -Recurse `
                -Context "nondisclosure rollback-temporary-root"
        } catch {
            $restoreFailures.Add($_.Exception.Message)
        }
        if ($restoreFailures.Count -gt 0) {
            throw "Nondisclosure evidence publication failed ('$publicationFailure') and rollback was incomplete; recovery artifacts were retained where possible. $($restoreFailures -join '; ')"
        }
        throw "Nondisclosure evidence publication failed and the prior artifact set was restored byte-for-byte: $publicationFailure"
    }

    # Commit point: all final assertions, hashes, result construction, backup
    # cleanup, temporary cleanup, and absence checks completed. Nothing fallible
    # follows except returning the already-prepared in-memory result.
    return $preparedResult
}

function Write-ValidatedNondisclosureEvidenceSet {
    param(
        [Parameter(Mandatory)]
        $Report,

        [Parameter(Mandatory)]
        [string]$FinalEvidencePath,

        [Parameter(Mandatory)]
        [string]$ExpectedBaseUrl,

        [Parameter(Mandatory)]
        [string]$ExpectedDeploymentEvidencePath,

        [Parameter(Mandatory)]
        [string]$ExpectedDeploymentEvidenceSha256,

        [Parameter(Mandatory)]
        [ValidateSet("fresh")]
        [string]$ExpectedDeploymentDataState,

        [Parameter(Mandatory)]
        [string]$ExpectedReleaseImageId,

        [Parameter(Mandatory)]
        [string]$ExpectedSourceCommit,

        [Parameter(Mandatory)]
        [string]$ExpectedDatabaseName,

        [Parameter(Mandatory)]
        [string]$ExpectedUnauthorizedActor,

        [Parameter(Mandatory)]
        [string]$ExpectedNotEvaluatedActor,

        [Parameter(Mandatory)]
        [int]$ExpectedWarmupPairsPerState,

        [Parameter(Mandatory)]
        [int]$ExpectedSamplesPerIdentifier,

        [Parameter(Mandatory)]
        [string]$ExpectedInstallationId,

        [Parameter(Mandatory)]
        [string]$ExpectedKnownResourceId,

        [Parameter(Mandatory)]
        [string]$ExpectedRandomResourceId,

        [Parameter(Mandatory)]
        [hashtable]$ExpectedRestrictedBodySha256ByAccessState,

        [switch]$AllowOverwrite,

        [ValidateSet("None", "AfterFirstFinalMove", "FormerOuterHash", "BackupCleanup")]
        [string]$InjectFailurePoint = "None"
    )

    $finalPath = [IO.Path]::GetFullPath($FinalEvidencePath)
    $finalDigestPath = "$finalPath.sha256"
    $protectedInputPaths = @(
        [IO.Path]::GetFullPath($ExpectedDeploymentEvidencePath),
        "$([IO.Path]::GetFullPath($ExpectedDeploymentEvidencePath)).sha256"
    )
    $basePathSet = @($finalPath, $finalDigestPath) + $protectedInputPaths
    Assert-NondisclosurePathSetSafety `
        -Paths $basePathSet `
        -Context "nondisclosure evidence input/output"
    foreach ($candidatePath in @($finalPath, $finalDigestPath)) {
        if ((Test-Path -LiteralPath $candidatePath) `
            -and -not (Test-Path -LiteralPath $candidatePath -PathType Leaf)) {
            throw "Nondisclosure evidence output '$candidatePath' exists but is not a file."
        }
    }
    if (((Test-Path -LiteralPath $finalPath) -or (Test-Path -LiteralPath $finalDigestPath)) `
        -and -not $AllowOverwrite) {
        throw "Retained nondisclosure evidence already exists. Refusing to replace '$finalPath' or its sidecar without -Overwrite."
    }

    $outputDirectory = [IO.Path]::GetDirectoryName($finalPath)
    [IO.Directory]::CreateDirectory($outputDirectory) | Out-Null
    Assert-NondisclosurePathSetSafety `
        -Paths $basePathSet `
        -Context "nondisclosure evidence input/output after output-directory creation"
    $temporaryDirectory = Join-Path `
        $outputDirectory `
        ".$([IO.Path]::GetFileName($finalPath)).nondisclosure-$([guid]::NewGuid().ToString('N'))"
    Assert-NondisclosurePathSetSafety `
        -Paths (@($temporaryDirectory) + $basePathSet) `
        -Context "nondisclosure evidence temporary path"
    [IO.Directory]::CreateDirectory($temporaryDirectory) | Out-Null
    $temporaryEvidencePath = Join-Path $temporaryDirectory "evidence.json"
    $temporaryDigestPath = "$temporaryEvidencePath.sha256"
    $publicationCommitted = $false
    $preparedResult = $null
    $primaryFailure = $null
    try {
        $reportJson = $Report | ConvertTo-Json -Depth 20
        Assert-NondisclosurePathSetSafety `
            -Paths (@($temporaryDirectory, $temporaryEvidencePath, $temporaryDigestPath) + $basePathSet) `
            -Context "nondisclosure evidence temporary path after creation"
        [IO.File]::WriteAllText(
            $temporaryEvidencePath,
            $reportJson + [Environment]::NewLine,
            [Text.UTF8Encoding]::new($false)
        )

        try {
            $roundTrippedReport = ConvertFrom-Sprint6ADeploymentEvidenceJson `
                -Json (Get-Content -LiteralPath $temporaryEvidencePath -Raw)
        } catch {
            throw "Temporary nondisclosure evidence is not complete valid JSON: $($_.Exception.Message)"
        }
        Assert-NondisclosureEvidenceDocument `
            -Document $roundTrippedReport `
            -ExpectedBaseUrl $ExpectedBaseUrl `
            -ExpectedDeploymentEvidencePath $ExpectedDeploymentEvidencePath `
            -ExpectedDeploymentEvidenceSha256 $ExpectedDeploymentEvidenceSha256 `
            -ExpectedDeploymentDataState $ExpectedDeploymentDataState `
            -ExpectedReleaseImageId $ExpectedReleaseImageId `
            -ExpectedSourceCommit $ExpectedSourceCommit `
            -ExpectedDatabaseName $ExpectedDatabaseName `
            -ExpectedUnauthorizedActor $ExpectedUnauthorizedActor `
            -ExpectedNotEvaluatedActor $ExpectedNotEvaluatedActor `
            -ExpectedWarmupPairsPerState $ExpectedWarmupPairsPerState `
            -ExpectedSamplesPerIdentifier $ExpectedSamplesPerIdentifier `
            -ExpectedInstallationId $ExpectedInstallationId `
            -ExpectedKnownResourceId $ExpectedKnownResourceId `
            -ExpectedRandomResourceId $ExpectedRandomResourceId `
            -ExpectedRestrictedBodySha256ByAccessState $ExpectedRestrictedBodySha256ByAccessState

        $digest = Get-FileSha256 -Path $temporaryEvidencePath
        Assert-NondisclosurePathSetSafety `
            -Paths (@($temporaryDirectory, $temporaryEvidencePath, $temporaryDigestPath) + $basePathSet) `
            -Context "nondisclosure evidence temporary sidecar path before write"
        [IO.File]::WriteAllText(
            $temporaryDigestPath,
            $digest + "`n",
            [Text.UTF8Encoding]::new($false)
        )
        Assert-NondisclosureEvidenceHashPair `
            -EvidencePath $temporaryEvidencePath `
            -DigestPath $temporaryDigestPath

        $preparedResult = Publish-NondisclosureEvidencePair `
            -TemporaryEvidencePath $temporaryEvidencePath `
            -TemporaryDigestPath $temporaryDigestPath `
            -TemporaryRootPath $temporaryDirectory `
            -FinalEvidencePath $finalPath `
            -FinalDigestPath $finalDigestPath `
            -ProtectedInputPaths $protectedInputPaths `
            -AllowOverwrite:$AllowOverwrite `
            -InjectFailurePoint $InjectFailurePoint
        $publicationCommitted = $true
    } catch {
        $primaryFailure = $_.Exception
    }

    if (-not $publicationCommitted) {
        try {
            Remove-PathVerified `
                -Path $temporaryDirectory `
                -Recurse `
                -Context "nondisclosure pre-publication temporary-root"
        } catch {
            $cleanupFailure = $_.Exception.Message
            throw "Nondisclosure evidence failed ('$($primaryFailure.Message)') and temporary cleanup also failed: $cleanupFailure"
        }
        throw $primaryFailure
    }

    # Publication returned only after its commit point and already constructed
    # this result. Do not perform another filesystem assertion/hash here.
    return $preparedResult
}

function New-NondisclosureSelfTestReport {
    param(
        [Parameter(Mandatory)][string]$DeploymentEvidencePath,
        [Parameter(Mandatory)][string]$GeneratedAt,
        [Parameter(Mandatory)][string]$RandomResourceId
    )

    $restrictedEnvelope = {
        param([Parameter(Mandatory)][string]$AccessState)
        [pscustomobject][ordered]@{
            schema_version = 1
            access_state = $AccessState
            owner_state = [pscustomobject][ordered]@{ kind = "undisclosed" }
            resource_identity_state = "undisclosed"
            resource_lifecycle_state = [pscustomobject][ordered]@{ kind = "undisclosed" }
            compatibility_state = "undisclosed"
            availability_state = "undisclosed"
        }
    }
    $metric = {
        [pscustomobject][ordered]@{
            known_ms = 1.0
            random_ms = 1.1
            delta_ms = 0.1
            relative_delta_percent = 10.0
            tolerance_ms = 2.0
            passed = $true
        }
    }
    $states = @("unauthorized", "not_evaluated") | ForEach-Object {
        $envelope = & $restrictedEnvelope $_
        [pscustomobject][ordered]@{
            access_state = $_
            response_status = 200
            exact_known_random_body_match = $true
            restricted_envelope = $envelope
            restricted_body_sha256 = Get-Sha256 -Text ($envelope | ConvertTo-Json -Depth 10 -Compress)
            known_sample_count = 200
            random_sample_count = 200
            median = & $metric
            p95 = & $metric
            passed = $true
        }
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        evidence_kind = "tessara.sprint-6a.resource-reference-nondisclosure"
        generated_at = $GeneratedAt
        passed = $true
        environment = [pscustomobject][ordered]@{
            base_url = "http://127.0.0.1:8080"
            deployment_evidence_path = [IO.Path]::GetFullPath($DeploymentEvidencePath)
            deployment_evidence_sha256 = "b" * 64
            deployment_data_state = "fresh"
            release_image_id = "sha256:$('c' * 64)"
            source_commit = "d" * 40
            database_name = "tessara_sprint6a_self_test"
            powershell_version = $PSVersionTable.PSVersion.ToString()
            dotnet_version = [Environment]::Version.ToString()
            os = [Runtime.InteropServices.RuntimeInformation]::OSDescription
        }
        fixture = [pscustomobject][ordered]@{
            installation_id = "11111111-1111-4111-8111-111111111111"
            resource_type = $resourceType
            known_resource_id = "22222222-2222-4222-8222-222222222222"
            random_resource_id = $RandomResourceId
            unauthorized_actor = "respondent@tessara.local"
            not_evaluated_actor = "operator@tessara.local"
        }
        methodology = [pscustomobject][ordered]@{
            endpoint = $resolvePath
            transport = "one persistent loopback HttpClient; one connection at a time; proxy and cookies disabled"
            measurement_boundary = "SendAsync through complete response-body read; request construction and assertions excluded"
            request_order = "balanced AB/BA known-random pairs"
            warmup_pairs_per_state = 25
            samples_per_identifier_per_state = 200
            percentile = "nearest-rank p95"
            relative_baseline = "faster (lower) known/random statistic"
            pass_tolerance = "delta <= max(2 ms, 20% of faster statistic), independently for median and p95"
            retries = 0
        }
        states = $states
    }
}

function Invoke-ResolveRequest {
    param(
        [Parameter(Mandatory)]
        [System.Net.Http.HttpClient]$Client,

        [Parameter(Mandatory)]
        [string]$RootUrl,

        [Parameter(Mandatory)]
        [string]$Token,

        [Parameter(Mandatory)]
        [string]$Body,

        [switch]$Measure
    )

    Invoke-HttpRequest `
        -Client $Client `
        -Method "POST" `
        -Uri "$RootUrl$resolvePath" `
        -Token $Token `
        -Body $Body `
        -Measure:$Measure
}

function Invoke-StateWarmup {
    param(
        [Parameter(Mandatory)]
        [System.Net.Http.HttpClient]$Client,

        [Parameter(Mandatory)]
        [string]$RootUrl,

        [Parameter(Mandatory)]
        [hashtable]$State
    )

    for ($pair = 0; $pair -lt $WarmupPairsPerState; $pair++) {
        $requests = if (($pair % 2) -eq 0) {
            @(
                @{ Label = "known"; Body = $State.KnownBody },
                @{ Label = "random"; Body = $State.RandomBody }
            )
        } else {
            @(
                @{ Label = "random"; Body = $State.RandomBody },
                @{ Label = "known"; Body = $State.KnownBody }
            )
        }

        foreach ($request in $requests) {
            $response = Invoke-ResolveRequest `
                -Client $Client `
                -RootUrl $RootUrl `
                -Token $State.Token `
                -Body $request.Body
            Assert-ExactRestrictedResponse `
                -Response $response `
                -ExpectedBody $State.BaselineBody `
                -Context "$($State.Name) warm-up $($request.Label) pair $pair"
        }
    }
}

function Invoke-StateMeasurement {
    param(
        [Parameter(Mandatory)]
        [System.Net.Http.HttpClient]$Client,

        [Parameter(Mandatory)]
        [string]$RootUrl,

        [Parameter(Mandatory)]
        [hashtable]$State
    )

    for ($pair = 0; $pair -lt $SamplesPerIdentifier; $pair++) {
        $requests = if (($pair % 2) -eq 0) {
            @(
                @{ Label = "known"; Body = $State.KnownBody },
                @{ Label = "random"; Body = $State.RandomBody }
            )
        } else {
            @(
                @{ Label = "random"; Body = $State.RandomBody },
                @{ Label = "known"; Body = $State.KnownBody }
            )
        }

        foreach ($request in $requests) {
            $response = Invoke-ResolveRequest `
                -Client $Client `
                -RootUrl $RootUrl `
                -Token $State.Token `
                -Body $request.Body `
                -Measure
            Assert-ExactRestrictedResponse `
                -Response $response `
                -ExpectedBody $State.BaselineBody `
                -Context "$($State.Name) measured $($request.Label) pair $pair"
            if ($request.Label -eq "known") {
                $State.KnownSamples.Add([double]$response.ElapsedMilliseconds)
            } else {
                $State.RandomSamples.Add([double]$response.ElapsedMilliseconds)
            }
        }
    }
}

function New-StateResult {
    param(
        [Parameter(Mandatory)]
        [hashtable]$State
    )

    [double[]]$knownSamples = $State.KnownSamples.ToArray()
    [double[]]$randomSamples = $State.RandomSamples.ToArray()
    $knownMedian = Get-Median -Samples $knownSamples
    $randomMedian = Get-Median -Samples $randomSamples
    $knownP95 = Get-NearestRankPercentile -Samples $knownSamples -Percentile 0.95
    $randomP95 = Get-NearestRankPercentile -Samples $randomSamples -Percentile 0.95
    $median = New-MetricComparison `
        -KnownMilliseconds $knownMedian `
        -RandomMilliseconds $randomMedian
    $p95 = New-MetricComparison `
        -KnownMilliseconds $knownP95 `
        -RandomMilliseconds $randomP95

    [pscustomobject]@{
        access_state                 = $State.ExpectedAccessState
        response_status              = 200
        exact_known_random_body_match = $true
        restricted_envelope          = ConvertFrom-ValidatedJsonText `
            -Text $State.BaselineBody `
            -Context "$($State.Name) retained restricted envelope"
        restricted_body_sha256       = Get-Sha256 -Text $State.BaselineBody
        known_sample_count           = $knownSamples.Count
        random_sample_count          = $randomSamples.Count
        median                       = $median
        p95                          = $p95
        passed                       = [bool]($median.passed -and $p95.passed)
    }
}

if ($SelfTest) {
    $selfTestRoot = Join-Path `
        ([IO.Path]::GetTempPath()) `
        "tessara-nondisclosure-evidence-$([guid]::NewGuid().ToString('N'))"
    [IO.Directory]::CreateDirectory($selfTestRoot) | Out-Null
    $selfTestFailure = $null
    $aliasPath = $null
    try {
        $finalEvidencePath = Join-Path $selfTestRoot "resource-reference-nondisclosure-fresh.json"
        $finalDigestPath = "$finalEvidencePath.sha256"
        $deploymentPath = Join-Path $selfTestRoot "deployment-fresh.json"
        $randomResourceId = "33333333-3333-4333-8333-33333333333a"
        $firstReport = New-NondisclosureSelfTestReport `
            -DeploymentEvidencePath $deploymentPath `
            -GeneratedAt "2026-07-14T12:00:00.0000000+00:00" `
            -RandomResourceId $randomResourceId
        $expectedRestrictedDigests = @{}
        foreach ($state in $firstReport.states) {
            $expectedRestrictedDigests[[string]$state.access_state] = [string]$state.restricted_body_sha256
        }
        $commonArguments = @{
            FinalEvidencePath = $finalEvidencePath
            ExpectedBaseUrl = "http://127.0.0.1:8080"
            ExpectedDeploymentEvidencePath = $deploymentPath
            ExpectedDeploymentEvidenceSha256 = "b" * 64
            ExpectedDeploymentDataState = "fresh"
            ExpectedReleaseImageId = "sha256:$('c' * 64)"
            ExpectedSourceCommit = "d" * 40
            ExpectedDatabaseName = "tessara_sprint6a_self_test"
            ExpectedUnauthorizedActor = "respondent@tessara.local"
            ExpectedNotEvaluatedActor = "operator@tessara.local"
            ExpectedWarmupPairsPerState = 25
            ExpectedSamplesPerIdentifier = 200
            ExpectedInstallationId = "11111111-1111-4111-8111-111111111111"
            ExpectedKnownResourceId = "22222222-2222-4222-8222-222222222222"
            ExpectedRandomResourceId = $randomResourceId
            ExpectedRestrictedBodySha256ByAccessState = $expectedRestrictedDigests
        }

        Write-ValidatedNondisclosureEvidenceSet -Report $firstReport @commonArguments | Out-Null
        Assert-NondisclosureEvidenceHashPair `
            -EvidencePath $finalEvidencePath `
            -DigestPath $finalDigestPath
        $firstEvidenceBytes = [IO.File]::ReadAllBytes($finalEvidencePath)
        $firstDigestBytes = [IO.File]::ReadAllBytes($finalDigestPath)

        $secondReport = New-NondisclosureSelfTestReport `
            -DeploymentEvidencePath $deploymentPath `
            -GeneratedAt "2026-07-14T12:01:00.0000000+00:00" `
            -RandomResourceId $randomResourceId
        $overwriteRejected = $false
        try {
            Write-ValidatedNondisclosureEvidenceSet -Report $secondReport @commonArguments | Out-Null
        } catch {
            $overwriteRejected = $true
        }
        if (-not $overwriteRejected `
            -or -not [Linq.Enumerable]::SequenceEqual($firstEvidenceBytes, [IO.File]::ReadAllBytes($finalEvidencePath)) `
            -or -not [Linq.Enumerable]::SequenceEqual($firstDigestBytes, [IO.File]::ReadAllBytes($finalDigestPath))) {
            throw "Self-test failed: overwrite refusal did not preserve the retained nondisclosure artifact set."
        }

        Write-ValidatedNondisclosureEvidenceSet `
            -Report $secondReport `
            @commonArguments `
            -AllowOverwrite | Out-Null
        Assert-NondisclosureEvidenceHashPair `
            -EvidencePath $finalEvidencePath `
            -DigestPath $finalDigestPath
        $secondEvidenceBytes = [IO.File]::ReadAllBytes($finalEvidencePath)
        $secondDigestBytes = [IO.File]::ReadAllBytes($finalDigestPath)
        if ([Linq.Enumerable]::SequenceEqual($firstEvidenceBytes, $secondEvidenceBytes)) {
            throw "Self-test failed: explicitly authorized evidence publication did not replace the prior JSON."
        }

        $thirdReport = New-NondisclosureSelfTestReport `
            -DeploymentEvidencePath $deploymentPath `
            -GeneratedAt "2026-07-14T12:02:00.0000000+00:00" `
            -RandomResourceId $randomResourceId
        $publicationFailed = $false
        try {
            Write-ValidatedNondisclosureEvidenceSet `
                -Report $thirdReport `
                @commonArguments `
                -AllowOverwrite `
                -InjectFailurePoint AfterFirstFinalMove | Out-Null
        } catch {
            $publicationFailed = $true
        }
        if (-not $publicationFailed `
            -or -not [Linq.Enumerable]::SequenceEqual($secondEvidenceBytes, [IO.File]::ReadAllBytes($finalEvidencePath)) `
            -or -not [Linq.Enumerable]::SequenceEqual($secondDigestBytes, [IO.File]::ReadAllBytes($finalDigestPath))) {
            throw "Self-test failed: partial-publication rollback did not preserve the complete prior artifact set."
        }
        Assert-NondisclosureEvidenceHashPair `
            -EvidencePath $finalEvidencePath `
            -DigestPath $finalDigestPath

        $formerOuterHashFailed = $false
        try {
            Write-ValidatedNondisclosureEvidenceSet `
                -Report $thirdReport `
                @commonArguments `
                -AllowOverwrite `
                -InjectFailurePoint FormerOuterHash | Out-Null
        } catch {
            $formerOuterHashFailed = $true
        }
        if (-not $formerOuterHashFailed `
            -or -not [Linq.Enumerable]::SequenceEqual($secondEvidenceBytes, [IO.File]::ReadAllBytes($finalEvidencePath)) `
            -or -not [Linq.Enumerable]::SequenceEqual($secondDigestBytes, [IO.File]::ReadAllBytes($finalDigestPath))) {
            throw "Self-test failed: failure at the former outer hash/result point did not restore the prior pair byte-for-byte."
        }

        $cleanupFailed = $false
        try {
            Write-ValidatedNondisclosureEvidenceSet `
                -Report $thirdReport `
                @commonArguments `
                -AllowOverwrite `
                -InjectFailurePoint BackupCleanup | Out-Null
        } catch {
            $cleanupFailed = $true
        }
        if (-not $cleanupFailed `
            -or -not [Linq.Enumerable]::SequenceEqual($secondEvidenceBytes, [IO.File]::ReadAllBytes($finalEvidencePath)) `
            -or -not [Linq.Enumerable]::SequenceEqual($secondDigestBytes, [IO.File]::ReadAllBytes($finalDigestPath))) {
            throw "Self-test failed: injected cleanup failure was not surfaced with prior-pair restoration."
        }

        $negativeBaseReport = New-NondisclosureSelfTestReport `
            -DeploymentEvidencePath $deploymentPath `
            -GeneratedAt "2026-07-14T12:03:00.0000000+00:00" `
            -RandomResourceId $randomResourceId
        $negativeCases = @(
            [pscustomobject]@{ Name = "claim"; Mutate = { param($value) $value.states[0].exact_known_random_body_match = $false } },
            [pscustomobject]@{ Name = "schema-type"; Mutate = { param($value) $value.schema_version = "1" } },
            [pscustomobject]@{ Name = "metric-type"; Mutate = { param($value) $value.states[0].median.known_ms = "1.0" } },
            [pscustomobject]@{ Name = "noncanonical-time"; Mutate = { param($value) $value.generated_at = "2026-07-14T12:03:00Z" } },
            [pscustomobject]@{ Name = "uppercase-uuid"; Mutate = { param($value) $value.fixture.random_resource_id = $value.fixture.random_resource_id.ToUpperInvariant() } },
            [pscustomobject]@{ Name = "live-id-mismatch"; Mutate = { param($value) $value.fixture.known_resource_id = "77777777-7777-4777-8777-777777777777" } },
            [pscustomobject]@{ Name = "live-digest-mismatch"; Mutate = { param($value) $value.states[0].restricted_body_sha256 = "f" * 64 } }
        )
        foreach ($negativeCase in $negativeCases) {
            $invalidReport = ConvertFrom-Sprint6ADeploymentEvidenceJson `
                -Json ($negativeBaseReport | ConvertTo-Json -Depth 20)
            & $negativeCase.Mutate $invalidReport
            $invalidOutputPath = Join-Path $selfTestRoot "invalid-$($negativeCase.Name).json"
            $invalidArguments = @{} + $commonArguments
            $invalidArguments.FinalEvidencePath = $invalidOutputPath
            $invalidRejected = $false
            try {
                Write-ValidatedNondisclosureEvidenceSet -Report $invalidReport @invalidArguments | Out-Null
            } catch {
                $invalidRejected = $true
            }
            if (-not $invalidRejected `
                -or (Test-Path -LiteralPath $invalidOutputPath) `
                -or (Test-Path -LiteralPath "$invalidOutputPath.sha256")) {
                throw "Self-test failed: invalid '$($negativeCase.Name)' evidence reached a retained output path."
            }
        }

        $secretBody = "SECRET-LOGIN-OR-RESTRICTED-BODY<"
        $diagnosticResponse = [pscustomobject]@{
            StatusCode = 403
            Body = $secretBody
            ContentType = "application/json; charset=utf-8"
            Utf8Length = [Text.Encoding]::UTF8.GetByteCount($secretBody)
            BodySha256 = Get-Sha256 -Text $secretBody
        }
        $diagnosticMessage = $null
        try {
            ConvertFrom-RequiredJson -Response $diagnosticResponse -Context "self-test restricted response" | Out-Null
        } catch {
            $diagnosticMessage = $_.Exception.Message
        }
        if ([string]::IsNullOrWhiteSpace($diagnosticMessage) `
            -or $diagnosticMessage.Contains($secretBody) `
            -or $diagnosticMessage -notmatch "label='self-test restricted response'" `
            -or $diagnosticMessage -notmatch "status=403" `
            -or $diagnosticMessage -notmatch "content_type='application/json; charset=utf-8'" `
            -or $diagnosticMessage -notmatch "utf8_length=" `
            -or $diagnosticMessage -notmatch "sha256=[0-9a-f]{64}") {
            throw "Self-test failed: HTTP diagnostics disclosed a raw body or omitted bounded metadata."
        }

        $restrictedSecret = "RESTRICTED-PROPERTY-SECRET"
        $restrictedBody = "{`"$restrictedSecret`":true}"
        $restrictedDiagnosticResponse = [pscustomobject]@{
            StatusCode = 200
            Body = $restrictedBody
            ContentType = "application/json"
            Utf8Length = [Text.Encoding]::UTF8.GetByteCount($restrictedBody)
            BodySha256 = Get-Sha256 -Text $restrictedBody
        }
        $restrictedDiagnosticMessage = $null
        try {
            Assert-RestrictedEnvelope `
                -Response $restrictedDiagnosticResponse `
                -ExpectedAccessState unauthorized `
                -Context "self-test restricted envelope"
        } catch {
            $restrictedDiagnosticMessage = $_.Exception.Message
        }
        if ([string]::IsNullOrWhiteSpace($restrictedDiagnosticMessage) `
            -or $restrictedDiagnosticMessage.Contains($restrictedSecret) `
            -or $restrictedDiagnosticMessage -notmatch "label='self-test restricted envelope'" `
            -or $restrictedDiagnosticMessage -notmatch "status=200" `
            -or $restrictedDiagnosticMessage -notmatch "utf8_length=" `
            -or $restrictedDiagnosticMessage -notmatch "sha256=[0-9a-f]{64}") {
            throw "Self-test failed: restricted-envelope diagnostics disclosed raw JSON or omitted bounded metadata."
        }

        $sidecarFixture = Join-Path $selfTestRoot "sidecar-fixture.json"
        $sidecarFixtureDigest = "$sidecarFixture.sha256"
        [IO.File]::WriteAllText($sidecarFixture, "fixture", [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText(
            $sidecarFixtureDigest,
            (Get-FileSha256 -Path $sidecarFixture) + "`nJUNK",
            [Text.UTF8Encoding]::new($false)
        )
        $sidecarJunkRejected = $false
        try {
            Assert-NondisclosureEvidenceHashPair `
                -EvidencePath $sidecarFixture `
                -DigestPath $sidecarFixtureDigest
        } catch {
            $sidecarJunkRejected = $true
        }
        if (-not $sidecarJunkRejected) {
            throw "Self-test failed: a SHA-256 sidecar with surrounding junk was accepted."
        }
        Remove-PathVerified -Path $sidecarFixture -Context "nondisclosure self-test sidecar fixture"
        Remove-PathVerified -Path $sidecarFixtureDigest -Context "nondisclosure self-test sidecar digest"

        $collisionArguments = @{} + $commonArguments
        $collisionArguments.FinalEvidencePath = $deploymentPath
        $collisionRejected = $false
        try {
            Write-ValidatedNondisclosureEvidenceSet -Report $negativeBaseReport @collisionArguments | Out-Null
        } catch {
            $collisionRejected = $true
        }
        if (-not $collisionRejected -or (Test-Path -LiteralPath $deploymentPath)) {
            throw "Self-test failed: a lexical deployment-input/output collision was not rejected before mutation."
        }

        $realDirectory = Join-Path $selfTestRoot "real-output"
        [IO.Directory]::CreateDirectory($realDirectory) | Out-Null
        $aliasPath = Join-Path $selfTestRoot "aliased-output"
        try {
            if ([Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([Runtime.InteropServices.OSPlatform]::Windows)) {
                New-Item -ItemType Junction -Path $aliasPath -Target $realDirectory -ErrorAction Stop | Out-Null
            } else {
                New-Item -ItemType SymbolicLink -Path $aliasPath -Target $realDirectory -ErrorAction Stop | Out-Null
            }
        } catch {
            throw "Self-test failed: the required reparse-point alias fixture could not be created: $($_.Exception.Message)"
        }

        $aliasArguments = @{} + $commonArguments
        $aliasArguments.FinalEvidencePath = Join-Path $aliasPath "aliased.json"
        $aliasRejected = $false
        try {
            Write-ValidatedNondisclosureEvidenceSet -Report $negativeBaseReport @aliasArguments | Out-Null
        } catch {
            $aliasRejected = $true
        }
        if (-not $aliasRejected `
            -or (Test-Path -LiteralPath (Join-Path $realDirectory "aliased.json")) `
            -or (Test-Path -LiteralPath (Join-Path $realDirectory "aliased.json.sha256"))) {
            throw "Self-test failed: a reparse-point output ancestor was not rejected before mutation."
        }
        Remove-Item -LiteralPath $aliasPath -Force -ErrorAction Stop
        if (Test-Path -LiteralPath $aliasPath) {
            throw "Self-test failed: reparse-point fixture cleanup did not remove the alias."
        }
        $aliasPath = $null

        $temporaryResidue = @(
            Get-ChildItem -LiteralPath $selfTestRoot -Force -Recurse | Where-Object {
                $_.Name -match '\.nondisclosure-' `
                    -or $_.Name -match '\.backup-' `
                    -or $_.Name -match '\.restore-'
            }
        )
        if ($temporaryResidue.Count -ne 0) {
            throw "Self-test failed: temporary or backup publication artifacts were not cleaned: $($temporaryResidue.FullName -join ', ')"
        }
    } catch {
        $selfTestFailure = $_.Exception
    }
    if ($null -ne $aliasPath -and (Test-Path -LiteralPath $aliasPath)) {
        try {
            Remove-Item -LiteralPath $aliasPath -Force -ErrorAction Stop
            if (Test-Path -LiteralPath $aliasPath) {
                throw "alias remained after cleanup"
            }
        } catch {
            throw "Nondisclosure self-test failed ('$($selfTestFailure.Message)') and alias cleanup failed: $($_.Exception.Message)"
        }
    }
    try {
        Remove-PathVerified `
            -Path $selfTestRoot `
            -Recurse `
            -Context "nondisclosure self-test root"
    } catch {
        throw "Nondisclosure self-test failed ('$($selfTestFailure.Message)') and root cleanup failed: $($_.Exception.Message)"
    }
    if ($null -ne $selfTestFailure) {
        throw $selfTestFailure
    }
    Write-Host "Sprint 6A nondisclosure evidence schema/hash/publication self-test passed." -ForegroundColor Green
    exit 0
}

if ([string]::IsNullOrWhiteSpace($DeploymentEvidencePath)) {
    throw "-DeploymentEvidencePath is required for retained nondisclosure evidence."
}
if ([string]::IsNullOrWhiteSpace($ExpectedDataState)) {
    throw "-ExpectedDataState fresh is required for retained nondisclosure evidence."
}

$baseUri = [Uri]$BaseUrl
if (-not $baseUri.IsAbsoluteUri `
    -or $baseUri.Scheme -notin @("http", "https") `
    -or -not $baseUri.IsLoopback) {
    throw "Timing conformance requires an absolute loopback http(s) BaseUrl, such as http://127.0.0.1:8080."
}
$baseUrl = $BaseUrl.TrimEnd('/')
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $timestamp = [DateTimeOffset]::UtcNow.ToString("yyyyMMddTHHmmssZ")
    $OutputPath = Join-Path $repoRoot "tmp/sprint-6a-resource-reference-nondisclosure-$timestamp.json"
} elseif (-not [System.IO.Path]::IsPathRooted($OutputPath)) {
    $OutputPath = Join-Path $repoRoot $OutputPath
}
$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)
$OutputDigestPath = "$OutputPath.sha256"
foreach ($candidatePath in @($OutputPath, $OutputDigestPath)) {
    if ((Test-Path -LiteralPath $candidatePath) `
        -and -not (Test-Path -LiteralPath $candidatePath -PathType Leaf)) {
        throw "Nondisclosure evidence output '$candidatePath' exists but is not a file."
    }
}
if (((Test-Path -LiteralPath $OutputPath -PathType Leaf) `
    -or (Test-Path -LiteralPath $OutputDigestPath -PathType Leaf)) -and -not $Overwrite) {
    throw "Retained nondisclosure evidence already exists. Refusing to replace '$OutputPath' or its sidecar without -Overwrite."
}
$resolvedDeploymentEvidencePath = Resolve-Sprint6ARepositoryPath `
    -RepositoryRoot $repoRoot `
    -Path $DeploymentEvidencePath
$protectedPaths = @(
    $OutputPath,
    $OutputDigestPath,
    $resolvedDeploymentEvidencePath,
    "$resolvedDeploymentEvidencePath.sha256"
)
Assert-NondisclosurePathSetSafety `
    -Paths $protectedPaths `
    -Context "nondisclosure evidence preflight input/output"
$deploymentEvidence = Assert-Sprint6ADeploymentEvidence `
    -RepositoryRoot $repoRoot `
    -EvidencePath $resolvedDeploymentEvidencePath `
    -BaseUrl $baseUrl `
    -ExpectedDataState $ExpectedDataState `
    -AdminEmail $AdminEmail `
    -AdminPassword $AdminPassword

$handler = [System.Net.Http.HttpClientHandler]::new()
$handler.UseProxy = $false
$handler.UseCookies = $false
$handler.MaxConnectionsPerServer = 1
$client = [System.Net.Http.HttpClient]::new($handler, $true)
$client.Timeout = [TimeSpan]::FromSeconds($RequestTimeoutSeconds)
$client.DefaultRequestHeaders.Accept.Add(
    [System.Net.Http.Headers.MediaTypeWithQualityHeaderValue]::new("application/json")
)

try {
    Write-Host "`n==> Sprint 6A resource-reference non-disclosure preflight" -ForegroundColor Cyan
    $health = Invoke-HttpRequest -Client $client -Method "GET" -Uri "$baseUrl/health"
    Assert-Status -Response $health -Expected 200 -Context "GET $baseUrl/health"

    $adminToken = Get-LoginToken `
        -Client $client `
        -RootUrl $baseUrl `
        -Email $AdminEmail `
        -Password $AdminPassword
    $unauthorizedToken = Get-LoginToken `
        -Client $client `
        -RootUrl $baseUrl `
        -Email $UnauthorizedEmail `
        -Password $UnauthorizedPassword
    $notEvaluatedToken = Get-LoginToken `
        -Client $client `
        -RootUrl $baseUrl `
        -Email $NotEvaluatedEmail `
        -Password $NotEvaluatedPassword

    $inventory = Invoke-JsonApi `
        -Client $client `
        -Method "GET" `
        -Uri "$baseUrl/api/admin/modules" `
        -Token $adminToken
    $installationId = [string]$inventory.installation.id
    if ([string]::IsNullOrWhiteSpace($installationId)) {
        throw "Module inventory did not expose an Application Installation id."
    }
    $installationGuid = [guid]::Empty
    if (-not [guid]::TryParseExact($installationId, "D", [ref]$installationGuid) `
        -or $installationId -cne $installationGuid.ToString("D")) {
        throw "Module inventory exposed a non-canonical Application Installation id."
    }
    $installationId = $installationGuid.ToString("D")

    if ([string]::IsNullOrWhiteSpace($KnownFormId)) {
        $forms = @(
            Invoke-JsonApi `
                -Client $client `
                -Method "GET" `
                -Uri "$baseUrl/api/forms" `
                -Token $adminToken
        )
        if ($forms.Count -eq 0) {
            throw "The disposable database is not populated with a Form. Seed demo data before running timing conformance."
        }
        $KnownFormId = [string]$forms[0].id
    }
    $knownGuid = [guid]::Empty
    if (-not [guid]::TryParse($KnownFormId, [ref]$knownGuid)) {
        throw "KnownFormId '$KnownFormId' is not a UUID."
    }
    $KnownFormId = $knownGuid.ToString("D").ToLowerInvariant()
    $knownBody = New-ResolveBody -InstallationId $installationId -ResourceId $KnownFormId

    $knownAdminResponse = Invoke-ResolveRequest `
        -Client $client `
        -RootUrl $baseUrl `
        -Token $adminToken `
        -Body $knownBody
    Assert-AuthorizedEnvelope `
        -Response $knownAdminResponse `
        -ExpectedIdentity "resolved" `
        -Context "admin known-resource preflight"

    $randomFormId = $null
    $randomBody = $null
    for ($attempt = 0; $attempt -lt 10; $attempt++) {
        $candidateId = [guid]::NewGuid().ToString("D").ToLowerInvariant()
        $candidateBody = New-ResolveBody `
            -InstallationId $installationId `
            -ResourceId $candidateId
        $candidateResponse = Invoke-ResolveRequest `
            -Client $client `
            -RootUrl $baseUrl `
            -Token $adminToken `
            -Body $candidateBody
        $candidateEnvelope = ConvertFrom-RequiredJson `
            -Response $candidateResponse `
            -Context "admin random-resource preflight"
        if ($candidateResponse.StatusCode -eq 200 `
            -and $candidateEnvelope.access_state -ceq "authorized" `
            -and $candidateEnvelope.resource_identity_state -ceq "unknown_resource") {
            $randomFormId = $candidateId
            $randomBody = $candidateBody
            break
        }
    }
    if ($null -eq $randomFormId) {
        throw "Could not produce an authorized unknown Form identifier after 10 attempts."
    }

    $states = @(
        @{
            Name                = "unauthorized"
            ExpectedAccessState = "unauthorized"
            Token               = $unauthorizedToken
            KnownBody           = $knownBody
            RandomBody          = $randomBody
            BaselineBody        = $null
            KnownSamples        = [System.Collections.Generic.List[double]]::new()
            RandomSamples       = [System.Collections.Generic.List[double]]::new()
        },
        @{
            Name                = "not_evaluated"
            ExpectedAccessState = "not_evaluated"
            Token               = $notEvaluatedToken
            KnownBody           = $knownBody
            RandomBody          = $randomBody
            BaselineBody        = $null
            KnownSamples        = [System.Collections.Generic.List[double]]::new()
            RandomSamples       = [System.Collections.Generic.List[double]]::new()
        }
    )

    foreach ($state in $states) {
        $knownRestricted = Invoke-ResolveRequest `
            -Client $client `
            -RootUrl $baseUrl `
            -Token $state.Token `
            -Body $state.KnownBody
        $randomRestricted = Invoke-ResolveRequest `
            -Client $client `
            -RootUrl $baseUrl `
            -Token $state.Token `
            -Body $state.RandomBody
        Assert-RestrictedEnvelope `
            -Response $knownRestricted `
            -ExpectedAccessState $state.ExpectedAccessState `
            -Context "$($state.Name) known-resource preflight"
        Assert-RestrictedEnvelope `
            -Response $randomRestricted `
            -ExpectedAccessState $state.ExpectedAccessState `
            -Context "$($state.Name) random-resource preflight"
        if ($knownRestricted.Body -cne $randomRestricted.Body) {
            throw "$($state.Name) known and random restricted bodies differed; $(Get-HttpResponseDiagnostic -Response $knownRestricted -Label "$($state.Name) known"); $(Get-HttpResponseDiagnostic -Response $randomRestricted -Label "$($state.Name) random")."
        }
        $state.BaselineBody = $knownRestricted.Body
    }

    Write-Host (
        "Preflight passed: known Form {0}; random Form {1}; restricted states unauthorized/not_evaluated." -f `
            $KnownFormId,
            $randomFormId
    ) -ForegroundColor Green
    Write-Host (
        "Warming {0} known/random pairs per state over one persistent loopback connection..." -f `
            $WarmupPairsPerState
    )
    foreach ($state in $states) {
        Invoke-StateWarmup -Client $client -RootUrl $baseUrl -State $state
    }

    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
    [GC]::Collect()

    Write-Host (
        "Measuring {0} known plus {0} random requests per restricted state..." -f `
            $SamplesPerIdentifier
    )
    foreach ($state in $states) {
        Invoke-StateMeasurement -Client $client -RootUrl $baseUrl -State $state
    }

    $stateResults = @($states | ForEach-Object { New-StateResult -State $_ })
    $liveRestrictedBodyDigests = @{}
    foreach ($state in $states) {
        $liveRestrictedBodyDigests[[string]$state.ExpectedAccessState] = Get-Sha256 -Text ([string]$state.BaselineBody)
    }
    $passed = @($stateResults | Where-Object { -not $_.passed }).Count -eq 0
    $deploymentEvidenceSha256 = Get-FileSha256 -Path $resolvedDeploymentEvidencePath
    $report = [pscustomobject]@{
        schema_version = 1
        evidence_kind  = "tessara.sprint-6a.resource-reference-nondisclosure"
        generated_at   = [DateTimeOffset]::UtcNow.ToString("o")
        passed         = $passed
        environment    = [pscustomobject]@{
            base_url                    = $baseUrl
            deployment_evidence_path    = $resolvedDeploymentEvidencePath
            deployment_evidence_sha256  = $deploymentEvidenceSha256
            deployment_data_state       = [string]$deploymentEvidence.snapshot.data.state
            release_image_id             = [string]$deploymentEvidence.snapshot.release_image.image_id
            source_commit                = [string]$deploymentEvidence.snapshot.source.commit
            database_name                = [string]$deploymentEvidence.snapshot.database_runtime.current_database
            powershell_version          = $PSVersionTable.PSVersion.ToString()
            dotnet_version              = [Environment]::Version.ToString()
            os                          = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
        }
        fixture        = [pscustomobject]@{
            installation_id = $installationId
            resource_type   = $resourceType
            known_resource_id = $KnownFormId
            random_resource_id = $randomFormId
            unauthorized_actor = $UnauthorizedEmail
            not_evaluated_actor = $NotEvaluatedEmail
        }
        methodology    = [pscustomobject]@{
            endpoint                       = $resolvePath
            transport                      = "one persistent loopback HttpClient; one connection at a time; proxy and cookies disabled"
            measurement_boundary           = "SendAsync through complete response-body read; request construction and assertions excluded"
            request_order                  = "balanced AB/BA known-random pairs"
            warmup_pairs_per_state          = $WarmupPairsPerState
            samples_per_identifier_per_state = $SamplesPerIdentifier
            percentile                     = "nearest-rank p95"
            relative_baseline              = "faster (lower) known/random statistic"
            pass_tolerance                  = "delta <= max(2 ms, 20% of faster statistic), independently for median and p95"
            retries                         = 0
        }
        states         = $stateResults
    }

    foreach ($result in $stateResults) {
        Write-Host "`n$($result.access_state)" -ForegroundColor Cyan
        Write-Host (
            "  median: known {0:N3} ms, random {1:N3} ms, delta {2:N3} ms, tolerance {3:N3} ms" -f `
                $result.median.known_ms,
                $result.median.random_ms,
                $result.median.delta_ms,
                $result.median.tolerance_ms
        )
        Write-Host (
            "  p95:    known {0:N3} ms, random {1:N3} ms, delta {2:N3} ms, tolerance {3:N3} ms" -f `
                $result.p95.known_ms,
                $result.p95.random_ms,
                $result.p95.delta_ms,
            $result.p95.tolerance_ms
        )
    }

    if (-not $passed) {
        throw "Resource-reference non-disclosure timing exceeded the fixed Sprint 6A tolerance. Investigate the release; do not widen the threshold."
    }

    Write-ValidatedNondisclosureEvidenceSet `
        -Report $report `
        -FinalEvidencePath $OutputPath `
        -ExpectedBaseUrl $baseUrl `
        -ExpectedDeploymentEvidencePath $resolvedDeploymentEvidencePath `
        -ExpectedDeploymentEvidenceSha256 $deploymentEvidenceSha256 `
        -ExpectedDeploymentDataState $ExpectedDataState `
        -ExpectedReleaseImageId ([string]$deploymentEvidence.snapshot.release_image.image_id) `
        -ExpectedSourceCommit ([string]$deploymentEvidence.snapshot.source.commit) `
        -ExpectedDatabaseName ([string]$deploymentEvidence.snapshot.database_runtime.current_database) `
        -ExpectedUnauthorizedActor $UnauthorizedEmail `
        -ExpectedNotEvaluatedActor $NotEvaluatedEmail `
        -ExpectedWarmupPairsPerState $WarmupPairsPerState `
        -ExpectedSamplesPerIdentifier $SamplesPerIdentifier `
        -ExpectedInstallationId $installationId `
        -ExpectedKnownResourceId $KnownFormId `
        -ExpectedRandomResourceId $randomFormId `
        -ExpectedRestrictedBodySha256ByAccessState $liveRestrictedBodyDigests `
        -AllowOverwrite:$Overwrite | Out-Null

    Write-Host "`nEvidence: $OutputPath"
    Write-Host "SHA-256 sidecar: $OutputDigestPath"
    Write-Host "Resource-reference non-disclosure shape and timing passed." -ForegroundColor Green
} finally {
    $client.Dispose()
}

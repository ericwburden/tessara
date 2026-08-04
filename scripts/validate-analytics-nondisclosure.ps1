[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8086",
    [string]$RestrictedEmail = "restricted-sprint7a@tessara.local",
    [string]$RestrictedPassword = "tessara-sprint-7a-restricted",
    [ValidateRange(200, 10000)][int]$SamplesPerIdentifier = 200,
    [string]$OutputPath,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

if ($SelfTest) {
    Test-Sprint7AAcceptanceContract
    if ($SamplesPerIdentifier -lt 200) { throw "The Sprint 7A nondisclosure profile requires at least 200 samples per identifier." }
    Write-Host "Sprint 7A analytics nondisclosure self-test passed."
    return
}
$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $RestrictedEmail -Password $RestrictedPassword
$random = [guid]::NewGuid().ToString("D")
$surfaces = @(
    [ordered]@{ name = "dataset"; known = "/api/datasets/$($script:Sprint7AFixture.dataset_id)"; random = "/api/datasets/$random" },
    [ordered]@{ name = "component"; known = "/api/components/$($script:Sprint7AFixture.metric_component_id)"; random = "/api/components/$random" },
    [ordered]@{ name = "dashboard"; known = "/api/dashboards/$($script:Sprint7AFixture.dashboard_id)"; random = "/api/dashboards/$random" }
)
$results = [Collections.Generic.List[object]]::new()
foreach ($surface in $surfaces) {
    $known = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $surface.known -Token $token
    $unknown = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $surface.random -Token $token
    if ($known.status -ne $unknown.status -or $known.content_type -cne $unknown.content_type -or $known.body -cne $unknown.body) {
        throw "$($surface.name) known/random restricted responses differ: known=$($known.status)/$($known.body_sha256), random=$($unknown.status)/$($unknown.body_sha256)."
    }
    $knownTimes = [Collections.Generic.List[double]]::new()
    $randomTimes = [Collections.Generic.List[double]]::new()
    foreach ($index in 0..($SamplesPerIdentifier - 1)) {
        foreach ($sample in $(if ($index % 2 -eq 0) { @("known", "random") } else { @("random", "known") })) {
            $path = if ($sample -eq "known") { $surface.known } else { $surface.random }
            $timer = [Diagnostics.Stopwatch]::StartNew()
            $null = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $path -Token $token
            $timer.Stop()
            if ($sample -eq "known") { $knownTimes.Add($timer.Elapsed.TotalMilliseconds) } else { $randomTimes.Add($timer.Elapsed.TotalMilliseconds) }
        }
    }
    $knownMedian = ($knownTimes | Sort-Object)[[int][Math]::Floor($knownTimes.Count / 2)]
    $randomMedian = ($randomTimes | Sort-Object)[[int][Math]::Floor($randomTimes.Count / 2)]
    $faster = [Math]::Max([Math]::Min($knownMedian, $randomMedian), 0.001)
    $allowed = [Math]::Max(2.0, $faster * 0.20)
    $delta = [Math]::Abs($knownMedian - $randomMedian)
    if ($delta -gt $allowed) {
        throw "$($surface.name) known/random median delta $delta ms exceeds $allowed ms."
    }
    $results.Add([ordered]@{
        surface = $surface.name
        response_status = $known.status
        content_type = $known.content_type
        restricted_body_sha256 = $known.body_sha256
        exact_body_match = $true
        sample_count_per_identifier = $SamplesPerIdentifier
        known_median_ms = $knownMedian
        random_median_ms = $randomMedian
        median_delta_ms = $delta
        allowed_delta_ms = $allowed
        passed = $true
    })
}

$result = [ordered]@{ schema_version = 1; evidence_kind = "tessara.sprint-7a.analytics-nondisclosure"; generated_at = [DateTimeOffset]::UtcNow.ToString("o"); base_url = $BaseUrl.TrimEnd('/'); actor = $RestrictedEmail; random_identifier = $random; surfaces = $results; passed = $true }
if ([string]::IsNullOrWhiteSpace($OutputPath)) { throw "-OutputPath is required for retained nondisclosure evidence." }
Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
$result | ConvertTo-Json -Depth 20

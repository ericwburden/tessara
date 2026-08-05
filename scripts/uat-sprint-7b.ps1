[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8086",
    [string]$SupervisorUrl = "http://127.0.0.1:8096",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$OutputPath,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7b-acceptance-contract.ps1")

if ($SelfTest) {
    Test-Sprint7BAcceptanceContract
    foreach ($runner in @("smoke-sprint-7b.ps1", "validate-resource-reference-nondisclosure.ps1")) {
        if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot $runner) -PathType Leaf)) {
            throw "Sprint 7B UAT dependency is missing: $runner"
        }
    }
    Write-Host "Sprint 7B UAT harness self-test passed."
    return
}
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    throw "-OutputPath is required for retained Sprint 7B scripted UAT evidence."
}

$checks = [Collections.Generic.List[object]]::new()
$smokePath = Join-Path (Split-Path -Parent ([IO.Path]::GetFullPath($OutputPath))) "smoke-prerequisite.json"
& (Join-Path $PSScriptRoot "smoke-sprint-7b.ps1") -BaseUrl $BaseUrl -SupervisorUrl $SupervisorUrl -AdminEmail $AdminEmail -AdminPassword $AdminPassword -OutputPath $smokePath -Overwrite:$Overwrite | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Sprint 7B smoke prerequisite failed." }
Assert-Sprint7A $true "smoke_prerequisite" "Source-exact Sprint 7B smoke passed" $checks

$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
try {
    $before = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/components/$($script:Sprint7BFixture.component_id)" -Token $token
    Assert-Sprint7A ($before.status -eq 200) "component_contract_read" "Provider lifecycle state is readable before refresh" $checks

    $refresh = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/dashboards/$($script:Sprint7BFixture.dashboard_id)/dependencies/refresh" -Method POST -Token $token
    Assert-Sprint7A ($refresh.status -eq 200) "dependency_refresh" "Authorized editor refresh crosses the deployed Dashboard-to-Core boundary" $checks
    $refreshDocument = $refresh.body | ConvertFrom-Json
    foreach ($property in @("dashboard_id", "health", "open_count", "deferred_count", "findings")) {
        Assert-Sprint7BJsonProperty -Object $refreshDocument -Name $property -Context "dependency refresh"
    }
    Assert-Sprint7A ($refreshDocument.dashboard_id -eq $script:Sprint7BFixture.dashboard_id) "dependency_refresh_identity" "Refresh response is tied to the expected Dashboard" $checks

    $after = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/components/$($script:Sprint7BFixture.component_id)" -Token $token
    Assert-Sprint7A ($after.status -eq 200 -and $after.body_sha256 -eq $before.body_sha256) "viewer_refresh_no_provider_write" "Dependency refresh does not mutate provider lifecycle state" $checks

    $editorDocument = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/dashboards/$($script:Sprint7BFixture.dashboard_id)/edit" -Token $token
    Assert-Sprint7A ($editorDocument.status -eq 200 -and $editorDocument.body.Contains("module-content") -and $editorDocument.body.Contains("Dependency health") -and -not $editorDocument.body.Contains("PROTOTYPE CONTROL") -and -not $editorDocument.body.Contains("One placement needs review before this Dashboard is healthy")) "dashboard_editor_contract" "Editor uses SDK shell, exposes dependency health, and omits rejected prototype/redundant controls" $checks
} finally {
    if (-not [string]::IsNullOrWhiteSpace($token)) {
        $logout = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token
        Assert-Sprint7A ($logout.status -in 200, 204) "session_cleanup" "Scripted UAT session was explicitly logged out" $checks
    }
}

$result = [ordered]@{
    schema_version = 1
    evidence_kind = "tessara.sprint-7b.scripted-uat"
    generated_at = [DateTimeOffset]::UtcNow.ToString("o")
    base_url = $BaseUrl.TrimEnd('/')
    fixture = $script:Sprint7BFixture
    smoke_evidence = $smokePath
    checks = $checks
    manual_scripts = 9
    passed = $true
}
Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
$result | ConvertTo-Json -Depth 20

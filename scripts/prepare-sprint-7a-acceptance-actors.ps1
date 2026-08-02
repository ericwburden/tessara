[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8086",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$ScopedEmail = "scoped-sprint7a@tessara.local",
    [string]$ScopedPassword = "tessara-sprint-7a-scoped",
    [string]$RestrictedEmail = "restricted-sprint7a@tessara.local",
    [string]$RestrictedPassword = "tessara-sprint-7a-restricted",
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

if ($SelfTest) {
    Test-Sprint7AAcceptanceContract
    if ($ScopedEmail -ceq $RestrictedEmail -or $ScopedPassword -ceq $RestrictedPassword) {
        throw "Scoped and restricted acceptance actors must have distinct credentials."
    }
    Write-Host "Sprint 7A acceptance actor preparation self-test passed."
    return
}

$token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
$rolesResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/roles" -Token $token
if ($rolesResponse.status -ne 200) { throw "Role catalog returned HTTP $($rolesResponse.status)." }
$roles = @($rolesResponse.body | ConvertFrom-Json)
$referenceRole = @($roles | Where-Object name -ceq "reference-operator")
if ($referenceRole.Count -ne 1) { throw "Expected exactly one reference-operator role, found $($referenceRole.Count)." }
$restrictedRole = @($roles | Where-Object name -ceq "respondent")
if ($restrictedRole.Count -ne 1) { throw "Expected exactly one respondent role, found $($restrictedRole.Count)." }

$usersResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users" -Token $token
if ($usersResponse.status -ne 200) { throw "User catalog returned HTTP $($usersResponse.status)." }
$users = @($usersResponse.body | ConvertFrom-Json)

function Set-AcceptanceActor {
    param(
        [Parameter(Mandatory)][string]$Email,
        [Parameter(Mandatory)][string]$DisplayName,
        [Parameter(Mandatory)][string]$Password,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$RoleIds,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$ScopeNodeIds
    )
    $existing = @($users | Where-Object email -ceq $Email)
    if ($existing.Count -gt 1) { throw "Acceptance actor '$Email' is not unique." }
    $payload = @{ email = $Email; display_name = $DisplayName; password = $Password; is_active = $true; role_ids = $RoleIds }
    $response = if ($existing.Count -eq 0) {
        Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users" -Method POST -Token $token -Body $payload
    } else {
        Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users/$($existing[0].id)" -Method PUT -Token $token -Body $payload
    }
    if ($response.status -ne 200) { throw "Acceptance actor '$Email' create/update returned HTTP $($response.status)." }
    $accountId = [string](($response.body | ConvertFrom-Json).id)
    $access = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users/$accountId/access" -Method PUT -Token $token -Body @{
        scope_node_ids = $ScopeNodeIds
        delegate_account_ids = @()
    }
    if ($access.status -ne 200) { throw "Acceptance actor '$Email' scope update returned HTTP $($access.status)." }
    [ordered]@{ email = $Email; account_id = $accountId; role_count = $RoleIds.Count; scope_count = $ScopeNodeIds.Count }
}

$scoped = Set-AcceptanceActor -Email $ScopedEmail -DisplayName "Sprint 7A Scoped Operator" -Password $ScopedPassword -RoleIds @([string]$referenceRole[0].id) -ScopeNodeIds @($script:Sprint7AFixture.organization_id)
$restricted = Set-AcceptanceActor -Email $RestrictedEmail -DisplayName "Sprint 7A Restricted Actor" -Password $RestrictedPassword -RoleIds @([string]$restrictedRole[0].id) -ScopeNodeIds @()
$logout = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token
if ($logout.status -notin 200, 204) { throw "Administrator fixture session cleanup returned HTTP $($logout.status)." }

[ordered]@{ schema_version = 1; prepared_at = [DateTimeOffset]::UtcNow.ToString("o"); actors = @($scoped, $restricted); secrets_retained = $false } | ConvertTo-Json -Depth 10

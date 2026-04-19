param(
    [string]$BaseUrl = "http://localhost:8080"
)
$BaseUrl = $BaseUrl.TrimEnd('/')

Write-Host "`n== Sprint UAT (1) Local deployment sanity ==" -ForegroundColor Cyan
Write-Host "Use after local deployment refresh:"
Write-Host "  .\scripts\local-launch.ps1"
Write-Host "  .\scripts\uat-sprint.ps1 -BaseUrl '$BaseUrl'"

function Assert-Contains {
    param(
        [string]$Content,
        [string[]]$Needles,
        [string]$Context
    )

    foreach ($needle in $Needles) {
        if ($Content -notlike "*$needle*") {
            throw "Sprint UAT failure in $Context. Missing marker: $needle"
        }
    }
}

function Invoke-Html {
    param(
        [string]$Uri,
        [string]$CookieJarPath = $null
    )

    $arguments = @("-sS", "-f")
    if ($null -ne $CookieJarPath) {
        $arguments += @("-b", $CookieJarPath)
    }
    $arguments += $Uri

    $content = & curl.exe @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "curl failed while fetching $Uri with exit code $LASTEXITCODE"
    }

    $content
}

function Assert-ProtectedShell {
    param(
        [string]$Content,
        [string[]]$Needles,
        [string]$Context
    )

    foreach ($needle in @(
        "top-app-bar",
        "global-search",
        "Loading Session",
        "Loading session state..."
    ) + $Needles) {
        if ($Content -notlike "*$needle*") {
            throw "Sprint UAT failure in $Context. Missing marker: $needle"
        }
    }
}

function Get-ApiToken {
    param(
        [string]$Email,
        [string]$Password
    )

    $loginBody = @{
        email    = $Email
        password = $Password
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/auth/login" -ContentType "application/json" -Body $loginBody
    if (-not $response.token) {
        throw "Sprint UAT failure: login response did not include a token for $Email."
    }
    return $response.token
}

function New-BrowserSession {
    param(
        [string]$Email,
        [string]$Password
    )

    $cookieJar = Join-Path ([System.IO.Path]::GetTempPath()) ("tessara-uat-" + [guid]::NewGuid().ToString() + ".txt")
    $payloadPath = Join-Path ([System.IO.Path]::GetTempPath()) ("tessara-uat-login-" + [guid]::NewGuid().ToString() + ".json")
    $loginBody = @{
        email    = $Email
        password = $Password
    } | ConvertTo-Json

    [System.IO.File]::WriteAllText($payloadPath, $loginBody, [System.Text.UTF8Encoding]::new($false))

    $response = & curl.exe -sS -f -c $cookieJar -H "Content-Type: application/json" --data-binary ("@" + $payloadPath) "$BaseUrl/api/auth/login"
    if ($LASTEXITCODE -ne 0) {
        throw "curl login failed for $Email with exit code $LASTEXITCODE"
    }
    if (-not $response) {
        throw "Login response for $Email was empty."
    }

    return $cookieJar
}

$adminToken = Get-ApiToken -Email "admin@tessara.local" -Password "tessara-dev-admin"
$headers = @{ Authorization = "Bearer $adminToken" }
$seedSummary = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/demo/seed" -Headers $headers -TimeoutSec 30
if ($seedSummary.seed_version -ne "uat-demo-v1") {
    throw "Sprint UAT failure: demo seed did not confirm expected uat-demo-v1."
}
$adminBrowserSession = New-BrowserSession -Email "admin@tessara.local" -Password "tessara-dev-admin"

$homeShell = Invoke-Html -Uri "$BaseUrl/app" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $homeShell -Needles @(
    "Tessara Home",
    "Product Areas"
) -Context "home shell"

$orgList = Invoke-Html -Uri "$BaseUrl/app/organization" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgList -Needles @("Organization") -Context "organization directory"

$nodes = Invoke-RestMethod -Uri "$BaseUrl/api/nodes" -Headers $headers -TimeoutSec 30
if (-not $nodes -or $nodes.Count -eq 0) {
    throw "Sprint UAT failure: seed dataset has no nodes."
}

$detailId = $nodes[0].id
$orgDetail = Invoke-Html -Uri "$BaseUrl/app/organization/$detailId" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgDetail -Needles @("Organization Detail") -Context "organization detail"

$orgCreate = Invoke-Html -Uri "$BaseUrl/app/organization/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgCreate -Needles @("Create Organization") -Context "organization create"

$orgEdit = Invoke-Html -Uri "$BaseUrl/app/organization/$detailId/edit" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgEdit -Needles @("Edit Organization") -Context "organization edit"

$formsList = Invoke-Html -Uri "$BaseUrl/app/forms" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formsList -Needles @("Forms") -Context "forms list"

$formCreate = Invoke-Html -Uri "$BaseUrl/app/forms/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formCreate -Needles @("Create Form") -Context "form create"

$formDetail = Invoke-Html -Uri "$BaseUrl/app/forms/$($seedSummary.form_id)" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formDetail -Needles @("Form Detail") -Context "form detail"

$formEdit = Invoke-Html -Uri "$BaseUrl/app/forms/$($seedSummary.form_id)/edit" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formEdit -Needles @("Edit Form") -Context "form edit"

$nodeTypesList = Invoke-Html -Uri "$BaseUrl/app/administration/node-types" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $nodeTypesList -Needles @("Organization Node Types") -Context "node-type directory"

$nodeTypesCreate = Invoke-Html -Uri "$BaseUrl/app/administration/node-types/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $nodeTypesCreate -Needles @("Create Organization Node Type") -Context "node-type create"

$nodeTypeCatalog = Invoke-RestMethod -Uri "$BaseUrl/api/node-types" -Headers $headers -TimeoutSec 30
if (-not $nodeTypeCatalog -or -not ($nodeTypeCatalog | Where-Object { $_.singular_label -and $_.plural_label })) {
    throw "Sprint UAT failure: readable node-type catalog did not include singular/plural labels."
}

$operatorToken = Get-ApiToken -Email "operator@tessara.local" -Password "tessara-dev-operator"
$operatorHeaders = @{ Authorization = "Bearer $operatorToken" }
$respondentToken = Get-ApiToken -Email "respondent@tessara.local" -Password "tessara-dev-respondent"
$respondentHeaders = @{ Authorization = "Bearer $respondentToken" }

foreach ($roleCheck in @(
    @{ Label = "operator"; Headers = $operatorHeaders },
    @{ Label = "respondent"; Headers = $respondentHeaders }
)) {
    try {
        Invoke-RestMethod -Uri "$BaseUrl/api/admin/node-types" -Headers $roleCheck.Headers -TimeoutSec 30 | Out-Null
        throw "Sprint UAT failure: $($roleCheck.Label) unexpectedly accessed /api/admin/node-types."
    } catch {
        if ($_.Exception.Message -notlike "*403*") {
            throw
        }
    }
}

Write-Host "`n== Sprint UAT checks passed for organization, forms, and seed flows. ==" -ForegroundColor Green
Write-Host "Next: if this was a sprint-completion run, keep the deployment open for UAT and log these pass markers."

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

$adminToken = Get-ApiToken -Email "admin@tessara.local" -Password "tessara-dev-admin"
$headers = @{ Authorization = "Bearer $adminToken" }
$seedSummary = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/demo/seed" -Headers $headers -TimeoutSec 30
if ($seedSummary.seed_version -ne "uat-demo-v1") {
    throw "Sprint UAT failure: demo seed did not confirm expected uat-demo-v1."
}

$homeShell = Invoke-RestMethod -Uri "$BaseUrl/app" -TimeoutSec 30
Assert-Contains -Content $homeShell -Needles @("Application Overview", "Role-Ready Home Modules", "/app/organization") -Context "home shell"

$orgList = Invoke-RestMethod -Uri "$BaseUrl/app/organization" -TimeoutSec 30
Assert-Contains -Content $orgList -Needles @(
    "Organization Directory",
    "Create Organization",
    "organization-directory-tree",
    "organization-skeleton-card"
) -Context "organization directory"

$nodes = Invoke-RestMethod -Uri "$BaseUrl/api/nodes" -Headers $headers -TimeoutSec 30
if (-not $nodes -or $nodes.Count -eq 0) {
    throw "Sprint UAT failure: seed dataset has no nodes."
}

$detailId = $nodes[0].id
$orgDetail = Invoke-RestMethod -Uri "$BaseUrl/app/organization/$detailId" -TimeoutSec 30
Assert-Contains -Content $orgDetail -Needles @(
    "Organization Detail",
    "Back to List",
    "organization-detail-status",
    "organization-detail-path",
    "organization-child-actions",
    "Related Forms",
    "Related Dashboards"
) -Context "organization detail"
if ($orgDetail -like "*Related Responses*") {
    throw "Sprint UAT failure in organization detail. Related Responses should not be rendered."
}

$orgCreate = Invoke-RestMethod -Uri "$BaseUrl/app/organization/new" -TimeoutSec 30
Assert-Contains -Content $orgCreate -Needles @(
    "Create Organization",
    "organization-form-status",
    "Submit",
    "Cancel",
    "organization-node-type-label",
    "organization-parent-node-label",
    "organization-metadata-title"
) -Context "organization create"

$orgEdit = Invoke-RestMethod -Uri "$BaseUrl/app/organization/$detailId/edit" -TimeoutSec 30
Assert-Contains -Content $orgEdit -Needles @(
    "Edit Organization",
    "organization-form-status",
    "Submit",
    "Cancel"
) -Context "organization edit"

$formsList = Invoke-RestMethod -Uri "$BaseUrl/app/forms" -TimeoutSec 30
Assert-Contains -Content $formsList -Needles @(
    "Forms",
    "Create Form",
    "form-list",
    "Lifecycle Summary"
) -Context "forms list"

$formCreate = Invoke-RestMethod -Uri "$BaseUrl/app/forms/new" -TimeoutSec 30
Assert-Contains -Content $formCreate -Needles @(
    "Create Form",
    "form-editor-status",
    "form-name",
    "form-slug",
    "form-scope-node-type",
    "Submit",
    "Cancel"
) -Context "form create"

$formDetail = Invoke-RestMethod -Uri "$BaseUrl/app/forms/$($seedSummary.form_id)" -TimeoutSec 30
Assert-Contains -Content $formDetail -Needles @(
    "Form Detail",
    "Form Summary",
    "Version Summary",
    "Section Preview",
    "Workflow Attachments"
) -Context "form detail"

$formEdit = Invoke-RestMethod -Uri "$BaseUrl/app/forms/$($seedSummary.form_id)/edit" -TimeoutSec 30
Assert-Contains -Content $formEdit -Needles @(
    "Edit Form",
    "Form Metadata",
    "Version Lifecycle",
    "form-version-create-form",
    "form-version-list",
    "Draft Version Workspace",
    "Publish Draft Version"
) -Context "form edit"

$nodeTypesList = Invoke-RestMethod -Uri "$BaseUrl/app/administration/node-types" -TimeoutSec 30
Assert-Contains -Content $nodeTypesList -Needles @(
    "Organization Node Types",
    "Create Organization Node Type",
    "node-type-list"
) -Context "node-type directory"

$nodeTypesCreate = Invoke-RestMethod -Uri "$BaseUrl/app/administration/node-types/new" -TimeoutSec 30
Assert-Contains -Content $nodeTypesCreate -Needles @(
    "Create Organization Node Type",
    "node-type-form",
    "node-type-parent-tags",
    "node-type-child-tags",
    "node-type-parent-options",
    "node-type-child-options",
    "node-type-metadata-fields-editor",
    "node-type-metadata-settings-modal"
) -Context "node-type create"

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

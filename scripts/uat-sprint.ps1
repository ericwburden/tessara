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
        "Search Tessara",
        "Primary navigation"
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
try {
    $seedSummary = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/demo/seed" -Headers $headers -TimeoutSec 30
} catch {
    $forms = Invoke-RestMethod -Uri "$BaseUrl/api/forms" -Headers $headers -TimeoutSec 30
    $datasets = Invoke-RestMethod -Uri "$BaseUrl/api/datasets" -Headers $headers -TimeoutSec 30
    $components = Invoke-RestMethod -Uri "$BaseUrl/api/components" -Headers $headers -TimeoutSec 30
    $sessionForm = $forms | Where-Object { $_.slug -eq "demo-session-log" } | Select-Object -First 1
    $sessionDataset = $datasets | Where-Object { $_.slug -eq "demo-session-log" } | Select-Object -First 1
    $sessionTableComponent = $components | Where-Object { $_.slug -eq "demo-session-log-table" } | Select-Object -First 1
    if (-not $sessionForm -or -not $sessionDataset -or -not $sessionTableComponent) {
        throw "Sprint UAT failure: demo seed failed and seeded Demo Session Log assets could not be found. Original error: $($_.Exception.Message)"
    }

    $seedSummary = [pscustomobject]@{
        seed_version         = "uat-demo-v1"
        form_id              = $sessionForm.id
        dataset_id           = $sessionDataset.id
        component_version_id = $sessionTableComponent.current_version_id
    }
}
if ($seedSummary.seed_version -ne "uat-demo-v1") {
    throw "Sprint UAT failure: demo seed did not confirm expected uat-demo-v1."
}
$adminBrowserSession = New-BrowserSession -Email "admin@tessara.local" -Password "tessara-dev-admin"

$homeShell = Invoke-Html -Uri "$BaseUrl/" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $homeShell -Needles @(
    "Home"
) -Context "home shell"

$orgList = Invoke-Html -Uri "$BaseUrl/organization" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgList -Needles @("Organization") -Context "organization directory"

$nodes = Invoke-RestMethod -Uri "$BaseUrl/api/nodes" -Headers $headers -TimeoutSec 30
if (-not $nodes -or $nodes.Count -eq 0) {
    throw "Sprint UAT failure: seed dataset has no nodes."
}

$detailId = $nodes[0].id
$orgDetail = Invoke-Html -Uri "$BaseUrl/organization/$detailId" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgDetail -Needles @("Organization Detail") -Context "organization detail"

$orgCreate = Invoke-Html -Uri "$BaseUrl/organization/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgCreate -Needles @("Create Organization") -Context "organization create"

$orgEdit = Invoke-Html -Uri "$BaseUrl/organization/$detailId/edit" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $orgEdit -Needles @("Edit Organization") -Context "organization edit"

$formsList = Invoke-Html -Uri "$BaseUrl/forms" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formsList -Needles @("Forms") -Context "forms list"

$formCreate = Invoke-Html -Uri "$BaseUrl/forms/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formCreate -Needles @("Create Form") -Context "form create"

$formDetail = Invoke-Html -Uri "$BaseUrl/forms/$($seedSummary.form_id)" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formDetail -Needles @("Form Detail") -Context "form detail"

$formEdit = Invoke-Html -Uri "$BaseUrl/forms/$($seedSummary.form_id)/edit" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $formEdit -Needles @("Edit Form") -Context "form edit"

$datasetsList = Invoke-Html -Uri "$BaseUrl/datasets" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $datasetsList -Needles @("Datasets") -Context "datasets list"

$datasetCreate = Invoke-Html -Uri "$BaseUrl/datasets/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $datasetCreate -Needles @("Create Dataset") -Context "dataset create"

$datasetDetail = Invoke-Html -Uri "$BaseUrl/datasets/$($seedSummary.dataset_id)" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $datasetDetail -Needles @("Dataset Detail") -Context "dataset detail"

$datasetEdit = Invoke-Html -Uri "$BaseUrl/datasets/$($seedSummary.dataset_id)/edit" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $datasetEdit -Needles @("Edit Dataset") -Context "dataset edit"

$datasetPreview = Invoke-RestMethod -Uri "$BaseUrl/api/datasets/$($seedSummary.dataset_id)/table" -Headers $headers -TimeoutSec 30
if (-not $datasetPreview.rows -or $datasetPreview.rows.Count -lt 1) {
    throw "Sprint UAT failure: dataset preview did not return seeded rows."
}

$components = Invoke-RestMethod -Uri "$BaseUrl/api/components" -Headers $headers -TimeoutSec 30
$seedComponent = $components | Where-Object { $_.current_version_id -eq $seedSummary.component_version_id } | Select-Object -First 1
if (-not $seedComponent) {
    throw "Sprint UAT failure: seeded component version $($seedSummary.component_version_id) did not appear in the component directory."
}
$componentTable = Invoke-RestMethod -Uri "$BaseUrl/api/components/$($seedComponent.slug)/table" -Headers $headers -TimeoutSec 30
if ($componentTable.materialization_state -ne "ready" -or -not $componentTable.rows -or $componentTable.rows.Count -lt 1) {
    throw "Sprint UAT failure: seeded component table did not return ready rows."
}

$seededVisualBar = Invoke-RestMethod -Uri "$BaseUrl/api/components/demo-session-log-bar/bar" -Headers $headers -TimeoutSec 30
$hasCompletedBarSeries = $seededVisualBar.points | Where-Object { $_.comparison -eq "Completed as planned" -and $_.color -eq "var(--semantic-primary)" }
$hasIncompleteBarSeries = $seededVisualBar.points | Where-Object { $_.comparison -eq "Did not complete as planned" -and $_.color -eq "var(--semantic-warning)" }
if ($seededVisualBar.materialization_state -ne "ready" -or $seededVisualBar.component_type -ne "bar" -or $seededVisualBar.legend_title -ne "Completion Status" -or -not $hasCompletedBarSeries -or -not $hasIncompleteBarSeries) {
    throw "Sprint UAT failure: seeded Demo Session Bar did not expose configured labels and colors."
}
$seededVisualLine = Invoke-RestMethod -Uri "$BaseUrl/api/components/demo-session-log-line/line" -Headers $headers -TimeoutSec 30
if ($seededVisualLine.materialization_state -ne "ready" -or $seededVisualLine.component_type -ne "line" -or -not $seededVisualLine.points -or $seededVisualLine.points.Count -lt 1) {
    throw "Sprint UAT failure: seeded Demo Session Line did not return ready points."
}
foreach ($seededSliceVisual in @(
    @{ Slug = "demo-session-completion-pie"; Kind = "pie" },
    @{ Slug = "demo-session-completion-donut"; Kind = "donut" }
)) {
    $seededSlices = Invoke-RestMethod -Uri "$BaseUrl/api/components/$($seededSliceVisual.Slug)/$($seededSliceVisual.Kind)" -Headers $headers -TimeoutSec 30
    if ($seededSlices.materialization_state -ne "ready" -or $seededSlices.component_type -ne $seededSliceVisual.Kind -or $seededSlices.legend_title -ne "Completion Status" -or -not ($seededSlices.slices | Where-Object { $_.category -eq "Did not complete as planned" -and $_.color -eq "var(--semantic-warning)" })) {
        throw "Sprint UAT failure: seeded Demo Session $($seededSliceVisual.Kind) did not expose configured legend, labels, and colors."
    }
}
$seededStatCard = Invoke-RestMethod -Uri "$BaseUrl/api/components/demo-session-total-participants-stat-card/stat-card" -Headers $headers -TimeoutSec 30
if ($seededStatCard.materialization_state -ne "ready" -or $seededStatCard.component_type -ne "stat_card" -or $seededStatCard.stat.label -ne "Total participants" -or $seededStatCard.stat.supporting_text -ne "Submitted Demo Session Log entries") {
    throw "Sprint UAT failure: seeded Demo Session StatCard did not expose configured display text."
}

$datasetDefinition = Invoke-RestMethod -Uri "$BaseUrl/api/datasets/$($seedSummary.dataset_id)" -Headers $headers -TimeoutSec 30
$visualField = $datasetDefinition.output_fields | Select-Object -First 1
if (-not $visualField -or -not $visualField.key) {
    throw "Sprint UAT failure: seeded dataset did not expose an output field for visual component coverage."
}
$visualDatasetMajor = $datasetDefinition.current_version_major
if (-not $visualDatasetMajor) {
    throw "Sprint UAT failure: seeded dataset did not expose a current major version for visual component coverage."
}
$visualSlug = "uat-visual-bar-$([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())"
$visualCreateBody = @{
    name        = "UAT Visual Bar"
    slug        = $visualSlug
    description = "Sprint 4B UAT visual component fixture."
    version     = @{
        dataset_id            = $seedSummary.dataset_id
        dataset_version_major = $visualDatasetMajor
        component_type        = "bar"
        config                = @{
            mode             = "summary"
            summary_field    = $visualField.key
            summary_type     = "count"
            category_field   = $visualField.key
            sort_field       = "summary_value"
            sort_direction   = "desc"
            number_of_points = 20
            value_format     = "integer"
        }
    }
} | ConvertTo-Json -Depth 20
$visualCreated = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/admin/components" -Headers $headers -ContentType "application/json" -Body $visualCreateBody -TimeoutSec 30
$visualDetail = Invoke-RestMethod -Uri "$BaseUrl/api/admin/components/$visualSlug" -Headers $headers -TimeoutSec 30
$visualVersion = $visualDetail.versions | Select-Object -First 1
Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/admin/components/$($visualCreated.id)/versions/$($visualVersion.id)/publish" -Headers $headers -TimeoutSec 30 | Out-Null
$visualBar = Invoke-RestMethod -Uri "$BaseUrl/api/components/$visualSlug/bar" -Headers $headers -TimeoutSec 30
if ($visualBar.materialization_state -ne "ready" -or $visualBar.component_type -ne "bar" -or -not $visualBar.points -or $visualBar.points.Count -lt 1) {
    throw "Sprint UAT failure: visual Bar component did not return ready points."
}

$componentsList = Invoke-Html -Uri "$BaseUrl/components" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $componentsList -Needles @("Components") -Context "components list"

$componentCreate = Invoke-Html -Uri "$BaseUrl/components/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $componentCreate -Needles @("Create Component") -Context "component create"

$componentDetail = Invoke-Html -Uri "$BaseUrl/components/$($seedComponent.slug)" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $componentDetail -Needles @("Component") -Context "component table preview"

$componentEdit = Invoke-Html -Uri "$BaseUrl/components/$($seedComponent.slug)/edit" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $componentEdit -Needles @("Edit Component") -Context "component edit"

$componentVersions = Invoke-Html -Uri "$BaseUrl/components/$($seedComponent.slug)/versions" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $componentVersions -Needles @("Component Versions") -Context "component versions"

$componentViewer = Invoke-Html -Uri "$BaseUrl/components/$($seedComponent.slug)/view" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $componentViewer -Needles @("Component") -Context "component viewer"

$visualViewer = Invoke-Html -Uri "$BaseUrl/components/$visualSlug/view" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $visualViewer -Needles @("Component") -Context "visual component viewer"

$workflowsList = Invoke-Html -Uri "$BaseUrl/workflows" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $workflowsList -Needles @("Workflows") -Context "workflow directory"

$workflowAssignments = Invoke-Html -Uri "$BaseUrl/workflows/assignments" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $workflowAssignments -Needles @("Workflow Assignments", "Workflows") -Context "workflow assignments"

$responsesList = Invoke-Html -Uri "$BaseUrl/responses" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $responsesList -Needles @("Responses") -Context "responses queue"

$nodeTypesList = Invoke-Html -Uri "$BaseUrl/administration/node-types" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $nodeTypesList -Needles @("Node Types") -Context "node-type directory"

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

Write-Host "`n== Sprint UAT checks passed for organization, forms, datasets, components, and seed flows. ==" -ForegroundColor Green
Write-Host "Next: if this was a sprint-completion run, keep the deployment open for UAT and log these pass markers."

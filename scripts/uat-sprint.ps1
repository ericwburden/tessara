[CmdletBinding()]
param(
    [string]$BaseUrl = "http://localhost:8080",
    [string]$DeploymentEvidencePath,
    [ValidateSet("fresh")][string]$ExpectedDataState,
    [string]$AcceptanceEvidencePath,
    [switch]$OverwriteAcceptanceEvidence,
    [switch]$DevelopmentMode
)
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$deploymentEvidenceCommon = Join-Path $PSScriptRoot "sprint-6a-deployment-evidence-common.ps1"
$acceptanceEvidenceCommon = Join-Path $PSScriptRoot "sprint-6a-acceptance-evidence-common.ps1"
$acceptanceEvidenceFullPath = $null
$deploymentEvidenceFullPath = $null
$deploymentEvidence = $null
$sensitiveTemporaryPaths = [Collections.Generic.List[string]]::new()
$currentRunSessions = [Collections.Generic.List[object]]::new()
$BaseUrl = $BaseUrl.TrimEnd('/')

if (-not (Test-Path -LiteralPath $acceptanceEvidenceCommon -PathType Leaf)) {
    throw "Could not find Sprint 6A acceptance evidence publisher at $acceptanceEvidenceCommon"
}
. $acceptanceEvidenceCommon
if ($OverwriteAcceptanceEvidence -and [string]::IsNullOrWhiteSpace($AcceptanceEvidencePath)) {
    throw "-OverwriteAcceptanceEvidence requires -AcceptanceEvidencePath."
}
if ($DevelopmentMode -and -not [string]::IsNullOrWhiteSpace($AcceptanceEvidencePath)) {
    throw "DevelopmentMode cannot produce Sprint acceptance evidence. Remove -DevelopmentMode or omit -AcceptanceEvidencePath."
}
if (-not $DevelopmentMode) {
    if ([string]::IsNullOrWhiteSpace($DeploymentEvidencePath) -or [string]::IsNullOrWhiteSpace($ExpectedDataState)) {
        throw "Sprint acceptance UAT requires -DeploymentEvidencePath and -ExpectedDataState fresh. Use -DevelopmentMode only for non-acceptance local diagnostics."
    }
    if (-not (Test-Path -LiteralPath $deploymentEvidenceCommon -PathType Leaf)) {
        throw "Could not find Sprint 6A deployment evidence validator at $deploymentEvidenceCommon"
    }
    . $deploymentEvidenceCommon
    $deploymentEvidenceFullPath = Resolve-Sprint6AAcceptanceEvidencePath `
        -RepositoryRoot $repoRoot `
        -Path $DeploymentEvidencePath
    if (-not [string]::IsNullOrWhiteSpace($AcceptanceEvidencePath)) {
        $acceptanceEvidenceFullPath = Resolve-Sprint6AAcceptanceEvidencePath `
            -RepositoryRoot $repoRoot `
            -Path $AcceptanceEvidencePath
        $null = Assert-Sprint6AAcceptanceEvidenceTargetAvailable `
            -EvidencePath $acceptanceEvidenceFullPath `
            -DeploymentEvidencePath $deploymentEvidenceFullPath `
            -Overwrite:$OverwriteAcceptanceEvidence
    }
    $deploymentEvidence = Assert-Sprint6ADeploymentEvidence `
        -RepositoryRoot $repoRoot `
        -EvidencePath $deploymentEvidenceFullPath `
        -BaseUrl $BaseUrl `
        -ExpectedDataState $ExpectedDataState
    if ([string]::IsNullOrWhiteSpace($acceptanceEvidenceFullPath)) {
        Write-Warning "No -AcceptanceEvidencePath was supplied. This run can diagnose acceptance behavior but will not retain durable UAT proof."
    }
} else {
    Write-Warning "DevelopmentMode skips deployment-evidence validation and cannot produce Sprint acceptance evidence."
}

Write-Host "`n== Sprint UAT (1) Local deployment sanity ==" -ForegroundColor Cyan
Write-Host "Use after local deployment refresh:"
Write-Host "  .\scripts\local-launch.ps1"
Write-Host "  .\scripts\uat-sprint.ps1 -BaseUrl '$BaseUrl' -DeploymentEvidencePath '<path>' -ExpectedDataState fresh -AcceptanceEvidencePath '<state-specific path>'"

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
        [string]$Context,
        [string[]]$ShellMarkers = @(
            "top-app-bar",
            "Search Tessara",
            "Primary navigation"
        )
    )

    foreach ($needle in $ShellMarkers + $Needles) {
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
    Register-Sprint6ACurrentRunSession -Sessions $currentRunSessions -Source bearer -Token ([string]$response.token)
    return $response.token
}

function Invoke-CurrentRunSessionLogout {
    param([Parameter(Mandatory)][object]$Session)

    if ($Session.source -eq 'bearer') {
        return Invoke-RestMethod `
            -Method Delete `
            -Uri "$BaseUrl/api/auth/logout" `
            -Headers @{ Authorization = "Bearer $($Session.token)" } `
            -TimeoutSec 30
    }
    try {
        if (-not (Test-Path -LiteralPath $Session.cookie_path -PathType Leaf)) {
            throw 'Current-run browser cookie is missing before exact logout.'
        }
        $null = Get-Sprint6ASecurePathInfo -Path $Session.cookie_path -RequireLeaf
        $response = & curl.exe -sS -f -X DELETE -b $Session.cookie_path -c $Session.cookie_path "$BaseUrl/api/auth/logout"
        if ($LASTEXITCODE -ne 0) {
            throw "Browser DELETE /api/auth/logout failed with exit code $LASTEXITCODE."
        }
        try { $browserLogout = $response | ConvertFrom-Json -NoEnumerate }
        catch { throw "Browser DELETE /api/auth/logout returned invalid JSON: $($_.Exception.Message)" }
        if ($null -eq $browserLogout.PSObject.Properties['signed_out'] -or
            $browserLogout.signed_out -isnot [bool] -or -not $browserLogout.signed_out) {
            throw 'Browser DELETE /api/auth/logout did not return exact signed_out=true.'
        }
        return $browserLogout
    } catch {
        if ([string]::IsNullOrWhiteSpace([string]$Session.token)) { throw }
        return Invoke-RestMethod `
            -Method Delete `
            -Uri "$BaseUrl/api/auth/logout" `
            -Headers @{ Authorization = "Bearer $($Session.token)" } `
            -TimeoutSec 30
    }
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

    $cookieJar = Register-Sprint6ASensitivePath -Paths $sensitiveTemporaryPaths -Path $cookieJar
    $payloadPath = Register-Sprint6ASensitivePath -Paths $sensitiveTemporaryPaths -Path $payloadPath
    try {
        Write-Sprint6AFileExclusive -Path $payloadPath -Content $loginBody
        New-Sprint6AExclusiveEmptyFile -Path $cookieJar

        $previousNativePreference = $PSNativeCommandUseErrorActionPreference
        $PSNativeCommandUseErrorActionPreference = $false
        try {
            $response = & curl.exe -sS -f -c $cookieJar -H "Content-Type: application/json" --data-binary ("@" + $payloadPath) "$BaseUrl/api/auth/login"
            $curlExitCode = $LASTEXITCODE
        } finally {
            $PSNativeCommandUseErrorActionPreference = $previousNativePreference
        }
        $cookieJar = Complete-Sprint6ABrowserLoginObservation `
            -Sessions $currentRunSessions `
            -CookiePath $cookieJar `
            -Response $response `
            -CurlExitCode $curlExitCode `
            -Context "Browser login for $Email"
    } finally {
        if (Test-Path -LiteralPath $payloadPath) {
            $null = Get-Sprint6ASecurePathInfo -Path $payloadPath -RequireLeaf
            Remove-Item -LiteralPath $payloadPath -Force -ErrorAction Stop
        }
        if (Test-Path -LiteralPath $payloadPath) {
            throw 'Browser login credential payload still exists after immediate cleanup.'
        }
    }

    return $cookieJar
}

try {
$adminToken = Get-ApiToken -Email "admin@tessara.local" -Password "tessara-dev-admin"
$headers = @{ Authorization = "Bearer $adminToken" }
$moduleInventory = Invoke-RestMethod -Uri "$BaseUrl/api/admin/modules" -Headers $headers -TimeoutSec 30
$independentDashboard = $moduleInventory.entries | Where-Object {
    $_.kind -eq "independently_deployed" -and
    $_.definition.id -eq "tessara.dashboards"
} | Select-Object -First 1
$seedSummary = $null
if (Test-Sprint6AShouldInvokeDemoSeed -ExpectedDataState $ExpectedDataState) {
    try {
        $seedSummary = Invoke-RestMethod -Method Post -Uri "$BaseUrl/api/demo/seed" -Headers $headers -TimeoutSec 30
    } catch {
        Assert-Sprint6ADemoSeedRefusalErrorRecord `
            -ErrorRecord $_ `
            -Context 'Sprint UAT demo seed'
    }
}
if ($null -eq $seedSummary) {
    $forms = Invoke-RestMethod -Uri "$BaseUrl/api/forms" -Headers $headers -TimeoutSec 30
    $datasets = Invoke-RestMethod -Uri "$BaseUrl/api/datasets" -Headers $headers -TimeoutSec 30
    $components = Invoke-RestMethod -Uri "$BaseUrl/api/components" -Headers $headers -TimeoutSec 30
    $dashboards = Invoke-RestMethod -Uri "$BaseUrl/api/dashboards" -Headers $headers -TimeoutSec 30
    $sessionForm = $forms | Where-Object { $_.slug -eq "demo-session-log" } | Select-Object -First 1
    $sessionDataset = $datasets | Where-Object { $_.slug -eq "demo-session-log" } | Select-Object -First 1
    $sessionTableComponent = $components | Where-Object { $_.slug -eq "demo-session-log-table" } | Select-Object -First 1
    $sessionDashboard = $dashboards | Where-Object { $_.name -eq "Demo Operations Dashboard" } | Select-Object -First 1
    if (-not $sessionForm -or -not $sessionDataset -or -not $sessionTableComponent -or
        (-not $independentDashboard -and -not $sessionDashboard)) {
        throw "Sprint UAT failure: required existing Demo Session Log assets could not be found."
    }

    $seedSummary = [pscustomobject]@{
        seed_version         = "uat-demo-v2"
        form_id              = $sessionForm.id
        dataset_id           = $sessionDataset.id
        component_version_id = $sessionTableComponent.current_version_id
        dashboard_id         = if ($sessionDashboard) { $sessionDashboard.id } else { $null }
    }
}
if ($independentDashboard) {
    $sprint6cSeedScript = Join-Path $PSScriptRoot "seed-sprint-6c-demo.ps1"
    $sprint6cSeed = (& $sprint6cSeedScript -BaseUrl $BaseUrl | Out-String) | ConvertFrom-Json
    if (
        $sprint6cSeed.seed_version -ne "sprint-6c-demo-v1" -or
        [int]$sprint6cSeed.dashboard_placements -ne 9
    ) {
        throw "Sprint UAT failure: Sprint 6C Dashboard seed did not produce nine placements."
    }
    $seedSummary.dashboard_id = $sprint6cSeed.dashboard_id
}
if ($seedSummary.seed_version -ne "uat-demo-v2") {
    throw "Sprint UAT failure: demo seed did not confirm expected uat-demo-v2."
}
$adminBrowserSession = New-BrowserSession -Email "admin@tessara.local" -Password "tessara-dev-admin"

$transitionEntries = @($moduleInventory.entries | Where-Object { $_.kind -eq "transitional_in_process" })
$expectedTransitionCount = if ($independentDashboard) { 6 } else { 7 }
if ($moduleInventory.schema_version -ne 1 -or $transitionEntries.Count -ne $expectedTransitionCount) {
    throw "Sprint UAT failure: Module inventory did not expose the expected deduplicated transition contributions alongside real modules."
}
$migrationContribution = $moduleInventory.entries | Where-Object {
    $_.descriptor.reserved_definition_id -eq "tessara.migration"
} | Select-Object -First 1
if (
    -not $migrationContribution `
    -or $migrationContribution.kind -ne "transitional_in_process" `
    -or $migrationContribution.descriptor.availability -ne "retired" `
    -or $migrationContribution.provider_eligible `
    -or $migrationContribution.supervisor_materializable `
    -or ($migrationContribution.PSObject.Properties.Name -contains "release") `
    -or ($migrationContribution.PSObject.Properties.Name -contains "instance")
) {
    throw "Sprint UAT failure: Migration did not remain a retired, non-deployable transition contribution."
}

$modulePolicy = Invoke-RestMethod -Uri "$BaseUrl/api/admin/navigation-policy" -Headers $headers -TimeoutSec 30
$moduleDestination = $modulePolicy.destinations | Where-Object {
    $_.id -eq "core.admin.modules" `
    -and $_.group_id -eq "core.admin" `
    -and $_.route -eq "/administration/modules" `
    -and -not $_.can_hide `
    -and -not $_.can_move_between_groups
} | Select-Object -First 1
$scopedRecordsDestination = $modulePolicy.destinations | Where-Object {
    $_.id -eq "tessara.reference.scoped-records.navigation" `
    -and $_.definition_id -eq "tessara.reference.scoped-records" `
    -and $_.semantic_destination -eq "tessara.reference.scoped-records.directory" `
    -and $_.route -eq "/reference/scoped-records" `
    -and $_.group_id -eq "core.main" `
    -and $_.visible
} | Select-Object -First 1
$sdkReferenceDestination = $modulePolicy.destinations | Where-Object {
    $_.id -eq "tessara.reference.module-sdk.navigation" `
    -and $_.definition_id -eq "tessara.reference.module-sdk" `
    -and $_.semantic_destination -eq "tessara.reference.module-sdk.root" `
    -and $_.route -eq "/reference/module-sdk" `
    -and $_.group_id -eq "core.main" `
    -and $_.visible
} | Select-Object -First 1
if (
    $modulePolicy.schema_version -ne 2 `
    -or -not $modulePolicy.can_manage_navigation `
    -or @($modulePolicy.groups).Count -lt 2 `
    -or -not ($modulePolicy.groups | Where-Object { $_.id -eq "core.main" }) `
    -or -not ($modulePolicy.groups | Where-Object { $_.id -eq "core.admin" }) `
    -or @($modulePolicy.destinations).Count -ne 15 `
    -or -not $moduleDestination `
    -or -not $scopedRecordsDestination `
    -or -not $sdkReferenceDestination
) {
    throw "Sprint UAT failure: schema-v2 navigation policy did not preserve required groups, exact membership, protected Module Management, and Scoped Records."
}

$shellNavigation = Invoke-RestMethod -Uri "$BaseUrl/api/shell/navigation" -Headers $headers -TimeoutSec 30
$shellItems = @($shellNavigation.groups | ForEach-Object { $_.items })
if (
    $shellNavigation.schema_version -ne 3 `
    -or $shellNavigation.state -ne "available" `
    -or -not ($shellItems | Where-Object { $_.key -eq "module_management" }) `
    -or ($shellItems | Where-Object { $_.key -eq "administration" }) `
    -or -not ($shellItems | Where-Object { $_.key -eq "user_management" }) `
    -or -not ($shellItems | Where-Object { $_.key -eq "roles_access" }) `
    -or -not ($shellItems | Where-Object { $_.key -eq "node_types" })
) {
    throw "Sprint UAT failure: schema-v3 administrator shell did not expose the four direct Core Admin destinations."
}

$removedAdministration = & curl.exe -sS -o NUL -D - -b $adminBrowserSession -w "STATUS:%{http_code}" "$BaseUrl/administration"
if ($LASTEXITCODE -ne 0 -or $removedAdministration -notcontains "STATUS:404" -or ($removedAdministration -match '^Location:')) {
    throw "Sprint UAT failure: /administration must be an ordinary 404 without redirect."
}

$moduleDirectory = Invoke-Html -Uri "$BaseUrl/administration/modules" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $moduleDirectory -Needles @(
    "Module inventory",
    "definitions",
    "Transitional — not independently deployable",
    "No Module Release",
    "No Module Instance",
    "Save navigation"
) -Context "Module Management directory"
if ($moduleDirectory -like "*/bridge/*") {
    throw "Sprint UAT failure: Module Management directory referenced a legacy bridge route."
}

$migrationDetail = Invoke-Html -Uri "$BaseUrl/administration/modules/tessara.migration" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $migrationDetail -Needles @(
    "Contribution retired",
    "The roadmap identity is retired and no current in-process product surface exists.",
    "No Module Release",
    "No Module Instance"
) -Context "retired Migration contribution detail"
if ($migrationDetail -like "*/bridge/*") {
    throw "Sprint UAT failure: retired Migration detail referenced a legacy bridge route."
}

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

$dashboardApi = Invoke-RestMethod -Uri "$BaseUrl/api/dashboards/$($seedSummary.dashboard_id)" -Headers $headers -TimeoutSec 30
if ($dashboardApi.placement_count -ne 9 -or $dashboardApi.placements.Count -ne 9) {
    throw "Sprint UAT failure: uat-demo-v2 Dashboard did not expose exactly 9 total placement envelopes."
}
$expectedDashboardKinds = @("table", "bar", "line", "pie", "donut", "stat_card")
$actualDashboardKinds = @($dashboardApi.placements | Where-Object { $_.availability -eq "available" } | ForEach-Object { $_.component.component_type } | Sort-Object -Unique)
foreach ($expectedKind in $expectedDashboardKinds) {
    if ($actualDashboardKinds -notcontains $expectedKind) {
        throw "Sprint UAT failure: seeded Dashboard is missing an available $expectedKind placement."
    }
}
foreach ($placement in $dashboardApi.placements) {
    if ($placement.grid_row -lt 1 -or $placement.grid_row -gt 240 -or
        $placement.grid_column -lt 1 -or $placement.grid_column -gt 12 -or
        $placement.grid_width -lt 1 -or ($placement.grid_column + $placement.grid_width - 1) -gt 12 -or
        $placement.grid_height -lt 1 -or
        ($placement.grid_row + $placement.grid_height - 1) -gt 240) {
        throw "Sprint UAT failure: seeded Dashboard placement has invalid typed geometry: $($placement | ConvertTo-Json -Depth 10)"
    }
    if ($placement.availability -ne "available") {
        continue
    }
    $kindPath = if ($placement.component.component_type -eq "stat_card") { "stat-card" } else { $placement.component.component_type }
    $executionUri = "$BaseUrl/api/components/$($placement.component.component_slug)/versions/$($placement.component.component_version_id)/$kindPath"
    if ($kindPath -eq "table") {
        $executionUri = "${executionUri}?page_size=1"
    }
    $execution = Invoke-RestMethod -Uri $executionUri -Headers $headers -TimeoutSec 30
    if ($execution.component_version_id -ne $placement.component.component_version_id -or $execution.materialization_state -ne "ready") {
        throw "Sprint UAT failure: exact-version Dashboard execution did not return the pinned ready Component version for placement $($placement.placement_id)."
    }
    if ($kindPath -eq "table" -and ($execution.pagination.page_size -ne 1 -or $execution.rows.Count -gt 1)) {
        throw "Sprint UAT failure: embedded Table endpoint did not honor bounded server paging."
    }
}

$dashboardsList = Invoke-Html -Uri "$BaseUrl/dashboards" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $dashboardsList -Needles @("Dashboards") -Context "dashboard directory" -ShellMarkers @("module-content")

$sdkReference = Invoke-Html -Uri "$BaseUrl/reference/module-sdk" -CookieJarPath $adminBrowserSession
Assert-Contains -Content $sdkReference -Needles @(
    "Module SDK Reference",
    "data-shell-state=`"active`"",
    "/_tessara/modules/tessara.reference.module-sdk/1.0.0/"
) -Context "canonical SDK reference"

$dashboardCreate = Invoke-Html -Uri "$BaseUrl/dashboards/new" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $dashboardCreate -Needles @("Create Dashboard") -Context "dashboard create" -ShellMarkers @("module-content")

$dashboardDetail = Invoke-Html -Uri "$BaseUrl/dashboards/$($seedSummary.dashboard_id)" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $dashboardDetail -Needles @("Dashboard Detail", "9 total placements") -Context "dashboard detail" -ShellMarkers @("module-content")

$dashboardEditor = Invoke-Html -Uri "$BaseUrl/dashboards/$($seedSummary.dashboard_id)/edit" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $dashboardEditor -Needles @("Dashboard") -Context "dashboard editor" -ShellMarkers @("module-content")

$dashboardViewer = Invoke-Html -Uri "$BaseUrl/dashboards/$($seedSummary.dashboard_id)/view" -CookieJarPath $adminBrowserSession
Assert-ProtectedShell -Content $dashboardViewer -Needles @("Dashboard", "Edit Dashboard") -Context "dashboard viewer" -ShellMarkers @("module-content")

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

Complete-Sprint6AAcceptanceRunCleanup `
    -Sessions $currentRunSessions `
    -SensitivePaths $sensitiveTemporaryPaths `
    -LogoutAction ${function:Invoke-CurrentRunSessionLogout}

if (-not $DevelopmentMode) {
    # Revalidate after every UAT assertion and mutation so retained proof is tied to
    # the deployment that remained live through completion of this exact pass.
    $deploymentEvidence = Assert-Sprint6ADeploymentEvidence `
        -RepositoryRoot $repoRoot `
        -EvidencePath $deploymentEvidenceFullPath `
        -BaseUrl $BaseUrl `
        -ExpectedDataState $ExpectedDataState
}

if (-not [string]::IsNullOrWhiteSpace($acceptanceEvidenceFullPath)) {
    $evidenceDocument = [pscustomobject][ordered]@{
        schema_version = $script:Sprint6AAcceptanceEvidenceSchemaVersion
        evidence_kind = 'tessara.sprint-6a.uat'
        status = 'passed'
        completed_at_utc = [DateTime]::UtcNow.ToString('o')
        expected_data_state = $ExpectedDataState
        base_url = $BaseUrl
        runner = [pscustomobject][ordered]@{
            path = 'scripts/uat-sprint.ps1'
            sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
        }
        checks = @(Get-Sprint6AAcceptanceExpectedChecks -Kind uat)
        deployment = [pscustomobject][ordered]@{
            evidence_sha256 = (Get-FileHash -LiteralPath $deploymentEvidenceFullPath -Algorithm SHA256).Hash.ToLowerInvariant()
            data_state = [string]$deploymentEvidence.snapshot.data.state
            source_commit = [string]$deploymentEvidence.snapshot.source.commit
            source_tree = [string]$deploymentEvidence.snapshot.source.tree
            image_id = [string]$deploymentEvidence.snapshot.release_image.image_id
            database_name = [string]$deploymentEvidence.snapshot.database_runtime.current_database
            installation_id = [string]$deploymentEvidence.snapshot.installation.id
            catalog_entries = [int]$deploymentEvidence.snapshot.catalog.definition_count
            built_in_seed_sha256 = [string]$deploymentEvidence.snapshot.built_in_seed.canonical_sha256
        }
        result = [pscustomobject][ordered]@{
            seed_version = [string]$seedSummary.seed_version
            dashboard_placements = [int]$dashboardApi.placement_count
            component_kinds = @($actualDashboardKinds)
            authorization_roles_checked = @('operator', 'respondent')
        }
    }
    $publication = Publish-Sprint6AAcceptanceEvidence `
        -EvidencePath $acceptanceEvidenceFullPath `
        -DeploymentEvidencePath $deploymentEvidenceFullPath `
        -RunnerFilePath $PSCommandPath `
        -Evidence $evidenceDocument `
        -Overwrite:$OverwriteAcceptanceEvidence
    Write-Host "Published Sprint 6A $ExpectedDataState UAT evidence: $($publication.evidence_path)" -ForegroundColor Green
}

Write-Host "`n== Sprint UAT checks passed for modules, organization, forms, datasets, components, dashboards, and seed flows. ==" -ForegroundColor Green
Write-Host "Next: if this was a sprint-completion run, keep the deployment open for UAT and retain the structured acceptance evidence."
} finally {
    Complete-Sprint6AAcceptanceRunCleanup `
        -Sessions $currentRunSessions `
        -SensitivePaths $sensitiveTemporaryPaths `
        -LogoutAction ${function:Invoke-CurrentRunSessionLogout} `
        -FinalAttempt
}

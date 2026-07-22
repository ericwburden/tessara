param(
    [switch]$KeepServices,
    [switch]$ComposeApi,
    [switch]$UseExistingService,
    [string]$BaseUrl = "http://127.0.0.1:8080",
    [int]$ApiTimeoutSeconds = 600,
    [string]$DeploymentEvidencePath,
    [ValidateSet("fresh")][string]$ExpectedDataState,
    [string]$AcceptanceEvidencePath,
    [switch]$OverwriteAcceptanceEvidence,
    [switch]$DevelopmentMode
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$tmpDir = Join-Path $repoRoot "tmp"
$apiOut = Join-Path $tmpDir "tessara-api.out.log"
$apiErr = Join-Path $tmpDir "tessara-api.err.log"
$baseUrl = $BaseUrl.TrimEnd('/')
$apiProcess = $null
$cargoCommand = $null
$deploymentEvidence = $null
$deploymentEvidenceCommon = Join-Path $PSScriptRoot "sprint-6a-deployment-evidence-common.ps1"
$acceptanceEvidenceCommon = Join-Path $PSScriptRoot "sprint-6a-acceptance-evidence-common.ps1"
$acceptanceEvidenceFullPath = $null
$deploymentEvidenceFullPath = $null
$sensitiveTemporaryPaths = [Collections.Generic.List[string]]::new()
$currentRunSessions = [Collections.Generic.List[object]]::new()

if (-not (Test-Path -LiteralPath $acceptanceEvidenceCommon -PathType Leaf)) {
    throw "Could not find Sprint 6A acceptance evidence publisher at $acceptanceEvidenceCommon"
}
. $acceptanceEvidenceCommon
$environmentSnapshot = Get-Sprint6AProcessEnvironmentSnapshot -Names @(
    'DATABASE_URL',
    'TEST_DATABASE_URL',
    'TESSARA_BIND_ADDR',
    'RUST_LOG'
)

if ($OverwriteAcceptanceEvidence -and [string]::IsNullOrWhiteSpace($AcceptanceEvidencePath)) {
    throw "-OverwriteAcceptanceEvidence requires -AcceptanceEvidencePath."
}
if ($DevelopmentMode -and -not [string]::IsNullOrWhiteSpace($AcceptanceEvidencePath)) {
    throw "DevelopmentMode cannot produce Sprint acceptance evidence. Remove -DevelopmentMode or omit -AcceptanceEvidencePath."
}
if (-not $DevelopmentMode) {
    if ([string]::IsNullOrWhiteSpace($DeploymentEvidencePath) -or [string]::IsNullOrWhiteSpace($ExpectedDataState)) {
        throw "Sprint acceptance smoke requires -DeploymentEvidencePath and -ExpectedDataState fresh. Use -DevelopmentMode only for non-acceptance local diagnostics."
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
    if ([string]::IsNullOrWhiteSpace($acceptanceEvidenceFullPath)) {
        Write-Warning "No -AcceptanceEvidencePath was supplied. This run can diagnose acceptance behavior but will not retain durable smoke proof."
    }
} else {
    Write-Warning "DevelopmentMode skips deployment-evidence validation and cannot produce Sprint acceptance evidence."
}

function Resolve-CargoCommand {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargo) {
        return $cargo.Source
    }

    $defaultCargo = Join-Path $HOME ".cargo\bin\cargo.exe"
    if (Test-Path $defaultCargo) {
        return $defaultCargo
    }

    throw "Unable to locate cargo. Add cargo to PATH or install it under $HOME\\.cargo\\bin."
}

function Invoke-Json {
    param(
        [string]$Method,
        [string]$Uri,
        [hashtable]$Headers = @{},
        [object]$Body = $null
    )

    $params = @{
        Method = $Method
        Uri = $Uri
        Headers = $Headers
        TimeoutSec = 30
    }

    if ($null -ne $Body) {
        $params.ContentType = "application/json"
        $params.Body = ($Body | ConvertTo-Json -Depth 20)
    }

    Invoke-RestMethod @params
}

function Invoke-CurrentRunSessionLogout {
    param([Parameter(Mandatory)][object]$Session)

    if ($Session.source -eq 'bearer') {
        return Invoke-RestMethod `
            -Method Delete `
            -Uri "$baseUrl/api/auth/logout" `
            -Headers @{ Authorization = "Bearer $($Session.token)" } `
            -TimeoutSec 30
    }
    try {
        if (-not (Test-Path -LiteralPath $Session.cookie_path -PathType Leaf)) {
            throw 'Current-run browser cookie is missing before exact logout.'
        }
        $null = Get-Sprint6ASecurePathInfo -Path $Session.cookie_path -RequireLeaf
        $response = & curl.exe -sS -f -X DELETE -b $Session.cookie_path -c $Session.cookie_path "$baseUrl/api/auth/logout"
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
            -Uri "$baseUrl/api/auth/logout" `
            -Headers @{ Authorization = "Bearer $($Session.token)" } `
            -TimeoutSec 30
    }
}

function Invoke-Html {
    param(
        [string]$Uri,
        [string]$CookieJarPath = $null
    )

    $arguments = @("-sS", "-f")
    if (-not [string]::IsNullOrWhiteSpace($CookieJarPath)) {
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

    foreach ($needle in @("app-root") + $Needles) {
        if ($Content -notlike "*$needle*") {
            throw "Smoke failure in $Context. Missing marker: $needle"
        }
    }
}

function New-BrowserSession {
    param(
        [string]$Email,
        [string]$Password
    )

    $cookieJar = Join-Path $tmpDir ("browser-" + [guid]::NewGuid().ToString() + ".txt")
    $payloadPath = Join-Path $tmpDir ("browser-login-" + [guid]::NewGuid().ToString() + ".json")
    $loginBody = @{
        email = $Email
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
            $response = & curl.exe `
                -sS `
                -f `
                -c $cookieJar `
                -H "Content-Type: application/json" `
                --data-binary ("@" + $payloadPath) `
                "$baseUrl/api/auth/login"
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

function Assert-LastExitCode {
    param([string]$CommandName)

    if ($LASTEXITCODE -ne 0) {
        throw "$CommandName failed with exit code $LASTEXITCODE"
    }
}

function Start-ComposeWithRetry {
    param(
        [string[]]$Arguments,
        [string]$CommandName,
        [int]$Attempts = 3
    )

    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        docker compose @Arguments | Out-Host
        if ($LASTEXITCODE -eq 0) {
            return
        }

        if ($attempt -eq $Attempts) {
            throw "$CommandName failed with exit code $LASTEXITCODE"
        }

        Start-Sleep -Seconds 3
        docker compose down -v | Out-Null
    }
}

function Invoke-PostgresSqlWithRetry {
    param(
        [string]$Sql,
        [string]$Database = "postgres",
        [int]$Attempts = 10
    )

    $previousNativePreference = $PSNativeCommandUseErrorActionPreference
    $PSNativeCommandUseErrorActionPreference = $false

    try {
    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        $result = docker compose exec -T postgres psql -U tessara -d $Database -tc $Sql 2>&1
        if ($LASTEXITCODE -eq 0) {
            return $result
        }

        if ($attempt -eq $Attempts) {
            throw "psql command failed after $Attempts attempts: $result"
        }

        Start-Sleep -Seconds 2
    }
    } finally {
        $PSNativeCommandUseErrorActionPreference = $previousNativePreference
    }
}

try {
    Set-Location $repoRoot
    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
    Remove-Item -LiteralPath $apiOut, $apiErr -ErrorAction SilentlyContinue
    $cargoCommand = Resolve-CargoCommand

    if ($UseExistingService) {
        Write-Host "Using existing Tessara service at $baseUrl" -ForegroundColor Cyan
    } elseif ($ComposeApi) {
        docker compose down --remove-orphans | Out-Host
        Assert-LastExitCode "docker compose down"
        Start-ComposeWithRetry -Arguments @("up", "-d", "--build") -CommandName "docker compose up"
    } else {
        docker compose down --remove-orphans | Out-Host
        Assert-LastExitCode "docker compose down"
        Start-ComposeWithRetry -Arguments @("up", "-d", "--wait", "postgres") -CommandName "docker compose up"
    }

    if (-not $UseExistingService) {
        $postgresDeadline = (Get-Date).AddSeconds(120)
        do {
            docker compose exec -T postgres pg_isready -U tessara -d postgres | Out-Null
            if ($LASTEXITCODE -eq 0) {
                break
            }
            Start-Sleep -Seconds 2
        } while ((Get-Date) -lt $postgresDeadline)

        if ((Get-Date) -ge $postgresDeadline) {
            throw "Timed out waiting for Postgres readiness"
        }
    }

    $env:DATABASE_URL = "postgres://tessara:tessara@localhost:5432/tessara"
    $env:TEST_DATABASE_URL = "postgres://tessara:tessara@localhost:5432/tessara_test"
    $env:TESSARA_BIND_ADDR = "127.0.0.1:8080"
    $env:RUST_LOG = "tessara_api=debug,sqlx=warn"

    if (-not $UseExistingService) {
        foreach ($databaseName in @("tessara", "tessara_test")) {
            $dbExists = Invoke-PostgresSqlWithRetry "SELECT 1 FROM pg_database WHERE datname = '$databaseName'"
            if (-not ($dbExists | Select-String "1" -Quiet)) {
                $null = Invoke-PostgresSqlWithRetry "CREATE DATABASE $databaseName"
            }
        }
    }

    if (-not $ComposeApi -and -not $UseExistingService) {
        & $cargoCommand test -p tessara-api --test demo_flow | Out-Host
        Assert-LastExitCode "cargo test -p tessara-api --test demo_flow"

        $null = Invoke-PostgresSqlWithRetry "DROP DATABASE tessara WITH (FORCE)"
        $null = Invoke-PostgresSqlWithRetry "CREATE DATABASE tessara"

        $apiProcess = Start-Process `
            -FilePath $cargoCommand `
            -ArgumentList @("run", "-p", "tessara-api") `
            -WorkingDirectory $repoRoot `
            -NoNewWindow `
            -PassThru `
            -RedirectStandardOutput $apiOut `
            -RedirectStandardError $apiErr
    }

    $deadline = (Get-Date).AddSeconds($ApiTimeoutSeconds)
    do {
        Start-Sleep -Seconds 2
        try {
            $health = Invoke-RestMethod -Uri "$baseUrl/health" -TimeoutSec 3
            if ($health -eq "ok") {
                break
            }
        } catch {
            if ($null -ne $apiProcess -and $apiProcess.HasExited) {
                throw "API exited before becoming healthy. stderr:`n$(Get-Content -Raw -LiteralPath $apiErr -ErrorAction SilentlyContinue)"
            }
        }
    } while ((Get-Date) -lt $deadline)

    if ((Get-Date) -ge $deadline) {
        throw "Timed out waiting for API health. stderr:`n$(Get-Content -Tail 80 -LiteralPath $apiErr -ErrorAction SilentlyContinue)"
    }

    if (-not $DevelopmentMode) {
        $deploymentEvidence = Assert-Sprint6ADeploymentEvidence `
            -RepositoryRoot $repoRoot `
            -EvidencePath $DeploymentEvidencePath `
            -BaseUrl $baseUrl `
            -ExpectedDataState $ExpectedDataState
    }

    $adminBrowserSession = New-BrowserSession -Email "admin@tessara.local" -Password "tessara-dev-admin"

    $appShell = Invoke-Html -Uri "$baseUrl/" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $appShell -Needles @("Home") -Context "application home shell"

    $loginShell = Invoke-Html -Uri "$baseUrl/login"
    if (
        -not ($loginShell -like "*Tessara Sign In*") `
        -or -not ($loginShell -like "*app-root*") `
        -or ($loginShell -like "*operator@tessara.local*")
    ) {
        throw "Expected login HTML to expose the native sign-in document without public demo credentials"
    }
    $organizationShell = Invoke-Html -Uri "$baseUrl/organization" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $organizationShell -Needles @("Organization") -Context "organization list shell"
    $formsShell = Invoke-Html -Uri "$baseUrl/forms" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formsShell -Needles @("Forms") -Context "forms list shell"
    $workflowsShell = Invoke-Html -Uri "$baseUrl/workflows" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $workflowsShell -Needles @("Workflows") -Context "workflows list shell"
    $workflowAssignmentsShell = Invoke-Html -Uri "$baseUrl/workflows/assignments" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $workflowAssignmentsShell -Needles @("Workflow Assignments", "Workflows") -Context "workflow assignments shell"
    $responsesShell = Invoke-Html -Uri "$baseUrl/responses" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $responsesShell -Needles @("Responses") -Context "responses list shell"
    $removedAdministration = & curl.exe -sS -o NUL -D - -b $adminBrowserSession -w "STATUS:%{http_code}" "$baseUrl/administration"
    if ($LASTEXITCODE -ne 0 -or $removedAdministration -notcontains "STATUS:404" -or ($removedAdministration -match '^Location:')) {
        throw "Smoke failure: /administration must be an ordinary 404 without redirect"
    }
    $usersShell = Invoke-Html -Uri "$baseUrl/administration/users" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $usersShell -Needles @("Users") -Context "users shell"
    $nodeTypesShell = Invoke-Html -Uri "$baseUrl/administration/node-types" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $nodeTypesShell -Needles @("Node Types") -Context "node type list shell"
    $rolesShell = Invoke-Html -Uri "$baseUrl/administration/roles" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $rolesShell -Needles @("Roles") -Context "roles shell"
    $modulesShell = Invoke-Html -Uri "$baseUrl/administration/modules" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $modulesShell -Needles @(
        "Module inventory",
        "7 definitions",
        "Transitional — not independently deployable",
        "No Module Release",
        "No Module Instance",
        "Save navigation"
    ) -Context "Module Management shell"
    if ($modulesShell -like "*/bridge/*") {
        throw "Smoke failure: Module Management directory referenced a legacy bridge route"
    }
    $migrationModuleShell = Invoke-Html -Uri "$baseUrl/administration/modules/tessara.migration" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $migrationModuleShell -Needles @(
        "Contribution retired",
        "The roadmap identity is retired and no current in-process product surface exists.",
        "No Module Release",
        "No Module Instance"
    ) -Context "retired Migration contribution shell"
    if ($migrationModuleShell -like "*/bridge/*") {
        throw "Smoke failure: retired Migration detail referenced a legacy bridge route"
    }
    $dashboardsShell = Invoke-Html -Uri "$baseUrl/dashboards" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardsShell -Needles @("Dashboards") -Context "dashboards shell"
    $datasetsShell = Invoke-Html -Uri "$baseUrl/datasets" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $datasetsShell -Needles @("Datasets") -Context "datasets shell"
    $datasetNewShell = Invoke-Html -Uri "$baseUrl/datasets/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $datasetNewShell -Needles @("Create Dataset") -Context "dataset create shell"

    $login = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "admin@tessara.local"; password = "tessara-dev-admin" }
    Register-Sprint6ACurrentRunSession -Sessions $currentRunSessions -Source bearer -Token ([string]$login.token)
    $headers = @{ Authorization = "Bearer $($login.token)" }
    $moduleInventory = Invoke-Json -Method "Get" -Uri "$baseUrl/api/admin/modules" -Headers $headers
    if ($moduleInventory.schema_version -ne 1 -or @($moduleInventory.entries).Count -ne 7) {
        throw "Smoke failure: Module inventory did not expose schema v1 with exactly seven transition contributions"
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
        throw "Smoke failure: Migration did not remain retired and non-deployable"
    }

    $modulePolicy = Invoke-Json -Method "Get" -Uri "$baseUrl/api/admin/navigation-policy" -Headers $headers
    $moduleDestination = $modulePolicy.destinations | Where-Object {
        $_.id -eq "core.admin.modules" `
        -and $_.group_id -eq "core.admin" `
        -and $_.route -eq "/administration/modules" `
        -and -not $_.can_hide `
        -and -not $_.can_move_between_groups
    } | Select-Object -First 1
    if (
        $modulePolicy.schema_version -ne 2 `
        -or -not $modulePolicy.can_manage_navigation `
        -or @($modulePolicy.groups).Count -lt 2 `
        -or -not ($modulePolicy.groups | Where-Object { $_.id -eq "core.main" }) `
        -or -not ($modulePolicy.groups | Where-Object { $_.id -eq "core.admin" }) `
        -or @($modulePolicy.destinations).Count -ne 13 `
        -or -not $moduleDestination
    ) {
        throw "Smoke failure: schema-v2 navigation policy did not preserve required groups, exact membership, and protected Module Management"
    }

    $shellNavigation = Invoke-Json -Method "Get" -Uri "$baseUrl/api/shell/navigation" -Headers $headers
    $shellItems = @($shellNavigation.groups | ForEach-Object { $_.items })
    if (
        $shellNavigation.schema_version -ne 2 `
        -or $shellNavigation.state -ne "available" `
        -or -not ($shellItems | Where-Object { $_.key -eq "module_management" }) `
        -or ($shellItems | Where-Object { $_.key -eq "administration" }) `
        -or -not ($shellItems | Where-Object { $_.key -eq "user_management" }) `
        -or -not ($shellItems | Where-Object { $_.key -eq "roles_access" }) `
        -or -not ($shellItems | Where-Object { $_.key -eq "node_types" })
    ) {
        throw "Smoke failure: schema-v2 administrator shell did not expose the four direct Core Admin destinations"
    }

    $formsModule = $moduleInventory.entries | Where-Object {
        $_.descriptor.reserved_definition_id -eq "tessara.forms"
    } | Select-Object -First 1
    $descriptorResponse = Invoke-WebRequest `
        -Uri "$baseUrl/api/admin/modules/tessara.forms/descriptor" `
        -Headers $headers `
        -TimeoutSec 30 `
        -UseBasicParsing
    $descriptorEtag = [string]$descriptorResponse.Headers.ETag
    $expectedDescriptorEtag = '"' + [string]$formsModule.source_digest + '"'
    if (
        -not $formsModule `
        -or $descriptorResponse.StatusCode -ne 200 `
        -or ([string]$descriptorResponse.Headers."Content-Type" -notlike "application/json*") `
        -or $descriptorEtag -cnotmatch '^"sha256:[0-9a-f]{64}"$' `
        -or $descriptorEtag -cne $expectedDescriptorEtag
    ) {
        throw "Smoke failure: Forms descriptor did not expose a quoted HTTP ETag whose opaque value exactly matched inventory provenance"
    }
    $seed = $null
    if (Test-Sprint6AShouldInvokeDemoSeed -ExpectedDataState $ExpectedDataState) {
        try {
            $seed = Invoke-Json -Method "Post" -Uri "$baseUrl/api/demo/seed" -Headers $headers
        } catch {
            Assert-Sprint6ADemoSeedRefusalErrorRecord `
                -ErrorRecord $_ `
                -Context 'Smoke demo seed'
        }
    }
    if ($null -eq $seed) {
        $formsForSeed = Invoke-Json -Method "Get" -Uri "$baseUrl/api/forms" -Headers $headers
        $datasetsForSeed = Invoke-Json -Method "Get" -Uri "$baseUrl/api/datasets" -Headers $headers
        $componentsForSeed = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components" -Headers $headers
        $dashboardsForSeed = Invoke-Json -Method "Get" -Uri "$baseUrl/api/dashboards" -Headers $headers
        $nodesForSeed = Invoke-Json -Method "Get" -Uri "$baseUrl/api/nodes" -Headers $headers
        $submissionsForSeed = Invoke-Json -Method "Get" -Uri "$baseUrl/api/submissions" -Headers $headers
        $sessionForm = $formsForSeed | Where-Object { $_.slug -eq "demo-session-log" } | Select-Object -First 1
        $sessionDataset = $datasetsForSeed | Where-Object { $_.slug -eq "demo-session-log" } | Select-Object -First 1
        $sessionTableComponent = $componentsForSeed | Where-Object { $_.slug -eq "demo-session-log-table" } | Select-Object -First 1
        $dashboardForSeed = $dashboardsForSeed | Where-Object { $_.name -eq "Demo Operations Dashboard" } | Select-Object -First 1
        $submissionForSeed = $submissionsForSeed | Where-Object { $_.form_name -eq "Demo Session Log" -and $_.status -eq "submitted" } | Select-Object -First 1
        if (-not $sessionForm -or -not $sessionDataset -or -not $sessionTableComponent -or -not $dashboardForSeed -or -not $submissionForSeed -or -not $nodesForSeed) {
            throw "Smoke failure: required existing Demo Session Log assets could not be found."
        }
        $seed = [pscustomobject]@{
            seed_version          = "uat-demo-v2"
            organization_node_id  = ($nodesForSeed | Select-Object -First 1).id
            form_id               = $sessionForm.id
            submission_id         = $submissionForSeed.id
            dataset_id            = $sessionDataset.id
            component_version_id  = $sessionTableComponent.current_version_id
            dashboard_id          = $dashboardForSeed.id
            analytics_values      = 1
        }
    }
    $summary = Invoke-Json -Method "Get" -Uri "$baseUrl/api/summary" -Headers $headers
    $operatorLogin = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "operator@tessara.local"; password = "tessara-dev-operator" }
    Register-Sprint6ACurrentRunSession -Sessions $currentRunSessions -Source bearer -Token ([string]$operatorLogin.token)
    $operatorHeaders = @{ Authorization = "Bearer $($operatorLogin.token)" }
    $respondentLogin = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "respondent@tessara.local"; password = "tessara-dev-respondent" }
    Register-Sprint6ACurrentRunSession -Sessions $currentRunSessions -Source bearer -Token ([string]$respondentLogin.token)
    $respondentHeaders = @{ Authorization = "Bearer $($respondentLogin.token)" }
    $delegatorLogin = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/auth/login" `
        -Body @{ email = "delegator@tessara.local"; password = "tessara-dev-delegator" }
    Register-Sprint6ACurrentRunSession -Sessions $currentRunSessions -Source bearer -Token ([string]$delegatorLogin.token)
    $delegatorHeaders = @{ Authorization = "Bearer $($delegatorLogin.token)" }
    if ($summary.published_form_versions -lt 1 -or $summary.submitted_submissions -lt 1 -or $summary.datasets -lt 1 -or $summary.components -lt 1 -or $summary.dashboards -lt 1) {
        throw "Expected application summary to include seeded published forms, submissions, datasets, components, and dashboards"
    }
    $nodes = Invoke-Json -Method "Get" -Uri "$baseUrl/api/nodes" -Headers $headers
    $dashboard = Invoke-Json -Method "Get" -Uri "$baseUrl/api/dashboards/$($seed.dashboard_id)" -Headers $headers
    $datasetDetail = Invoke-Json -Method "Get" -Uri "$baseUrl/api/datasets/$($seed.dataset_id)" -Headers $headers
    $dataset = Invoke-Json -Method "Get" -Uri "$baseUrl/api/datasets/$($seed.dataset_id)/table" -Headers $headers
    $components = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components" -Headers $headers
    $seedComponent = $components | Where-Object { $_.current_version_id -eq $seed.component_version_id } | Select-Object -First 1
    if (-not $seedComponent) {
        throw "Expected seeded component version $($seed.component_version_id) to appear in the component directory"
    }
    $componentTable = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components/$($seedComponent.slug)/table" -Headers $headers
    $operatorMe = Invoke-Json -Method "Get" -Uri "$baseUrl/api/me" -Headers $operatorHeaders
    $operatorNodes = Invoke-Json -Method "Get" -Uri "$baseUrl/api/nodes?q=Demo" -Headers $operatorHeaders
    $respondentOptions = Invoke-Json -Method "Get" -Uri "$baseUrl/api/responses/options" -Headers $respondentHeaders
    $respondentPending = Invoke-Json -Method "Get" -Uri "$baseUrl/api/workflow-assignments/pending" -Headers $respondentHeaders
    $delegatorMe = Invoke-Json -Method "Get" -Uri "$baseUrl/api/me" -Headers $delegatorHeaders
    $delegateAccountId = $delegatorMe.delegations[0].account_id
    $delegatedOptions = Invoke-Json -Method "Get" -Uri "$baseUrl/api/responses/options?delegate_account_id=$delegateAccountId" -Headers $delegatorHeaders
    $respondentFormsDenied = $false
    $operatorNodeTypeAdminDenied = $false
    $respondentNodeTypeAdminDenied = $false
    try {
        Invoke-Json -Method "Get" -Uri "$baseUrl/api/forms" -Headers $respondentHeaders | Out-Null
    } catch {
        $respondentFormsDenied = $_.Exception.Message -like "*403*"
    }
    try {
        Invoke-Json -Method "Get" -Uri "$baseUrl/api/admin/node-types" -Headers $operatorHeaders | Out-Null
    } catch {
        $operatorNodeTypeAdminDenied = $_.Exception.Message -like "*403*"
    }
    try {
        Invoke-Json -Method "Get" -Uri "$baseUrl/api/admin/node-types" -Headers $respondentHeaders | Out-Null
    } catch {
        $respondentNodeTypeAdminDenied = $_.Exception.Message -like "*403*"
    }
    $readableNodeTypes = Invoke-Json -Method "Get" -Uri "$baseUrl/api/node-types" -Headers $headers

    if ($seed.analytics_values -lt 1) {
        throw "Expected at least one analytics value, got $($seed.analytics_values)"
    }
    $adminBrowserSession = New-BrowserSession -Email "admin@tessara.local" -Password "tessara-dev-admin"
    if ($nodes.Count -lt 1) {
        throw "Expected at least one node, got $($nodes.Count)"
    }
    if ($dashboard.placement_count -lt 1 -or $dashboard.placements.Count -ne $dashboard.placement_count) {
        throw "Expected Dashboard placement_count to match a non-empty placement envelope list, got: $($dashboard | ConvertTo-Json -Depth 20)"
    }
    foreach ($placement in $dashboard.placements) {
        if ($placement.grid_row -lt 1 -or $placement.grid_column -lt 1 -or
            $placement.grid_width -lt 1 -or $placement.grid_height -lt 1 -or
            ($placement.grid_column + $placement.grid_width - 1) -gt 12 -or
            ($placement.grid_row + $placement.grid_height - 1) -gt 240) {
            throw "Expected every Dashboard placement to expose valid typed grid geometry, got: $($placement | ConvertTo-Json -Depth 10)"
        }
    }
    for ($leftIndex = 0; $leftIndex -lt $dashboard.placements.Count; $leftIndex++) {
        $left = $dashboard.placements[$leftIndex]
        for ($rightIndex = $leftIndex + 1; $rightIndex -lt $dashboard.placements.Count; $rightIndex++) {
            $right = $dashboard.placements[$rightIndex]
            $overlap = $left.grid_column -le ($right.grid_column + $right.grid_width - 1) -and
                $right.grid_column -le ($left.grid_column + $left.grid_width - 1) -and
                $left.grid_row -le ($right.grid_row + $right.grid_height - 1) -and
                $right.grid_row -le ($left.grid_row + $left.grid_height - 1)
            if ($overlap) {
                throw "Expected Dashboard seed placements not to overlap, got: $($dashboard.placements | ConvertTo-Json -Depth 20)"
            }
        }
    }
    if ($seed.seed_version -eq "uat-demo-v2" -and $dashboard.placement_count -ne 9) {
        throw "Expected uat-demo-v2 Dashboard to contain 9 typed placements, got $($dashboard.placement_count)"
    }
    $hasExpectedDatasetValue = $dataset.rows | Where-Object {
        $_.values.PSObject.Properties.Value -contains "42"
    }
    if ($dataset.rows.Count -lt 1 -or -not $hasExpectedDatasetValue) {
        throw "Expected dataset value 42, got: $($dataset | ConvertTo-Json -Depth 20)"
    }
    if ($componentTable.materialization_state -ne "ready" -or $componentTable.rows.Count -lt 1) {
        throw "Expected seeded component table to render ready rows, got: $($componentTable | ConvertTo-Json -Depth 20)"
    }
    $seededVisualBar = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components/demo-session-log-bar/bar" -Headers $headers
    $hasCompletedBarSeries = $seededVisualBar.points | Where-Object { $_.comparison -eq "Completed as planned" -and $_.color -eq "var(--semantic-primary)" }
    $hasIncompleteBarSeries = $seededVisualBar.points | Where-Object { $_.comparison -eq "Did not complete as planned" -and $_.color -eq "var(--semantic-warning)" }
    if ($seededVisualBar.materialization_state -ne "ready" -or $seededVisualBar.component_type -ne "bar" -or $seededVisualBar.legend_title -ne "Completion Status" -or -not $hasCompletedBarSeries -or -not $hasIncompleteBarSeries) {
        throw "Expected seeded Demo Session Bar to expose configured labels and colors, got: $($seededVisualBar | ConvertTo-Json -Depth 20)"
    }
    $seededVisualLine = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components/demo-session-log-line/line" -Headers $headers
    if ($seededVisualLine.materialization_state -ne "ready" -or $seededVisualLine.component_type -ne "line" -or $seededVisualLine.points.Count -lt 1) {
        throw "Expected seeded Demo Session Line to render ready points, got: $($seededVisualLine | ConvertTo-Json -Depth 20)"
    }
    foreach ($seededSliceVisual in @(
        @{ Slug = "demo-session-completion-pie"; Kind = "pie" },
        @{ Slug = "demo-session-completion-donut"; Kind = "donut" }
    )) {
        $seededSlices = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components/$($seededSliceVisual.Slug)/$($seededSliceVisual.Kind)" -Headers $headers
        if ($seededSlices.materialization_state -ne "ready" -or $seededSlices.component_type -ne $seededSliceVisual.Kind -or $seededSlices.legend_title -ne "Completion Status" -or -not ($seededSlices.slices | Where-Object { $_.category -eq "Did not complete as planned" -and $_.color -eq "var(--semantic-warning)" })) {
            throw "Expected seeded Demo Session $($seededSliceVisual.Kind) to expose configured legend, labels, and colors, got: $($seededSlices | ConvertTo-Json -Depth 20)"
        }
    }
    $seededStatCard = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components/demo-session-total-participants-stat-card/stat-card" -Headers $headers
    if ($seededStatCard.materialization_state -ne "ready" -or $seededStatCard.component_type -ne "stat_card" -or $seededStatCard.stat.label -ne "Total participants" -or $seededStatCard.stat.supporting_text -ne "Submitted Demo Session Log entries") {
        throw "Expected seeded Demo Session StatCard to expose configured display text, got: $($seededStatCard | ConvertTo-Json -Depth 20)"
    }
    $visualField = $datasetDetail.output_fields | Select-Object -First 1
    if (-not $visualField -or -not $visualField.key) {
        throw "Expected seeded dataset to expose an output field for visual component coverage"
    }
    $visualSlug = "smoke-visual-bar-$([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())"
    $visualComponent = Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/admin/components" `
        -Headers $headers `
        -Body @{
            name        = "Smoke Visual Bar"
            slug        = $visualSlug
            description = "Sprint 4B smoke visual component fixture."
            version     = @{
                dataset_id            = $seed.dataset_id
                dataset_version_major = $datasetDetail.current_version_major
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
        }
    $visualDetail = Invoke-Json -Method "Get" -Uri "$baseUrl/api/admin/components/$visualSlug" -Headers $headers
    $visualVersion = $visualDetail.versions | Select-Object -First 1
    Invoke-Json `
        -Method "Post" `
        -Uri "$baseUrl/api/admin/components/$($visualComponent.id)/versions/$($visualVersion.id)/publish" `
        -Headers $headers | Out-Null
    $visualBar = Invoke-Json -Method "Get" -Uri "$baseUrl/api/components/$visualSlug/bar" -Headers $headers
    if ($visualBar.materialization_state -ne "ready" -or $visualBar.component_type -ne "bar" -or $visualBar.points.Count -lt 1) {
        throw "Expected visual Bar component to render ready points, got: $($visualBar | ConvertTo-Json -Depth 20)"
    }
    if (-not ($operatorMe.capabilities -contains "forms:read") -or $operatorMe.scope_nodes.Count -lt 1) {
        throw "Expected operator account context to include effective forms read capability and scoped nodes"
    }
    if (-not ($operatorNodes | Where-Object { $_.name -eq "Demo Program Family Outreach" }) -or ($operatorNodes | Where-Object { $_.name -eq "Demo Partner Community Bridge" })) {
        throw "Expected operator node list to stay within assigned scope"
    }
    if ($respondentOptions.assignments.Count -lt 1) {
        throw "Expected respondent response options to return assigned response starts"
    }
    if ($respondentPending.Count -lt 1) {
        throw "Expected respondent pending workflow assignments to return assigned start work"
    }
    if ($delegatedOptions.assignments.Count -lt 1) {
        throw "Expected delegated response options to support delegated response context"
    }
    if (-not $respondentFormsDenied) {
        throw "Expected respondent access to /api/forms to be forbidden"
    }
    if (-not $operatorNodeTypeAdminDenied -or -not $respondentNodeTypeAdminDenied) {
        throw "Expected operator and respondent access to /api/admin/node-types to be forbidden"
    }
    if ($readableNodeTypes.Count -lt 1 -or -not ($readableNodeTypes | Where-Object { $_.singular_label -and $_.plural_label })) {
        throw "Expected readable node-type catalog to include singular/plural labels"
    }

    $organizationDetail = Invoke-Html -Uri "$baseUrl/organization/$($seed.organization_node_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $organizationDetail -Needles @("Organization Detail") -Context "organization detail shell"
    $organizationNew = Invoke-Html -Uri "$baseUrl/organization/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $organizationNew -Needles @("Create Organization") -Context "organization create shell"
    $formDetail = Invoke-Html -Uri "$baseUrl/forms/$($seed.form_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formDetail -Needles @("Form Detail") -Context "form detail shell"
    $formNew = Invoke-Html -Uri "$baseUrl/forms/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formNew -Needles @("Create Form") -Context "form create shell"
    $formEdit = Invoke-Html -Uri "$baseUrl/forms/$($seed.form_id)/edit" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $formEdit -Needles @("Edit Form") -Context "form edit shell"
    $responseDetail = Invoke-Html -Uri "$baseUrl/responses/$($seed.submission_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $responseDetail -Needles @("Response Detail") -Context "response detail shell"
    $responseNew = Invoke-Html -Uri "$baseUrl/responses/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $responseNew -Needles @("Start Response") -Context "response create shell"
    $dashboardDetailPage = Invoke-Html -Uri "$baseUrl/dashboards/$($seed.dashboard_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardDetailPage -Needles @("Dashboard Detail") -Context "dashboard detail shell"
    $dashboardNew = Invoke-Html -Uri "$baseUrl/dashboards/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardNew -Needles @("Create Dashboard") -Context "dashboard create shell"
    $dashboardEditorPage = Invoke-Html -Uri "$baseUrl/dashboards/$($seed.dashboard_id)/edit" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardEditorPage -Needles @("Dashboard") -Context "dashboard editor shell"
    $dashboardViewerPage = Invoke-Html -Uri "$baseUrl/dashboards/$($seed.dashboard_id)/view" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $dashboardViewerPage -Needles @("Dashboard") -Context "dashboard viewer shell"
    $datasetDetailPage = Invoke-Html -Uri "$baseUrl/datasets/$($seed.dataset_id)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $datasetDetailPage -Needles @("Dataset Detail") -Context "dataset detail shell"
    $datasetEditPage = Invoke-Html -Uri "$baseUrl/datasets/$($seed.dataset_id)/edit" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $datasetEditPage -Needles @("Edit Dataset") -Context "dataset edit shell"
    $componentsPage = Invoke-Html -Uri "$baseUrl/components" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $componentsPage -Needles @("Components") -Context "component directory shell"
    $componentNewPage = Invoke-Html -Uri "$baseUrl/components/new" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $componentNewPage -Needles @("Create Component") -Context "component create shell"
    $componentDetailPage = Invoke-Html -Uri "$baseUrl/components/$($seedComponent.slug)" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $componentDetailPage -Needles @("Component") -Context "component table preview shell"
    $componentEditPage = Invoke-Html -Uri "$baseUrl/components/$($seedComponent.slug)/edit" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $componentEditPage -Needles @("Edit Component") -Context "component edit shell"
    $componentVersionsPage = Invoke-Html -Uri "$baseUrl/components/$($seedComponent.slug)/versions" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $componentVersionsPage -Needles @("Component Versions") -Context "component versions shell"
    $componentViewerPage = Invoke-Html -Uri "$baseUrl/components/$($seedComponent.slug)/view" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $componentViewerPage -Needles @("Component") -Context "component viewer shell"
    $visualViewerPage = Invoke-Html -Uri "$baseUrl/components/$visualSlug/view" -CookieJarPath $adminBrowserSession
    Assert-ProtectedShell -Content $visualViewerPage -Needles @("Component") -Context "visual component viewer shell"

    Complete-Sprint6AAcceptanceRunCleanup `
        -Sessions $currentRunSessions `
        -SensitivePaths $sensitiveTemporaryPaths `
        -LogoutAction ${function:Invoke-CurrentRunSessionLogout} `
        -EnvironmentSnapshot $environmentSnapshot

    if (-not $DevelopmentMode) {
        # Revalidate after every smoke assertion. The retained pass must bind to the
        # deployment that was still live when the checks completed, not only at startup.
        $deploymentEvidence = Assert-Sprint6ADeploymentEvidence `
            -RepositoryRoot $repoRoot `
            -EvidencePath $deploymentEvidenceFullPath `
            -BaseUrl $baseUrl `
            -ExpectedDataState $ExpectedDataState
    }

    $result = [pscustomobject]@{
        status = "passed"
        organization_node_id = $seed.organization_node_id
        dashboard_id = $seed.dashboard_id
        dataset_rows = $dataset.rows.Count
        component_rows = $componentTable.rows.Count
        seeded_visual_points = $seededVisualBar.points.Count
        visual_points = $visualBar.points.Count
        first_dataset_participants = $dataset.rows[0].values.participants
        deployment = if ($null -eq $deploymentEvidence) { $null } else {
            [pscustomobject]@{
                data_state = [string]$deploymentEvidence.snapshot.data.state
                image_id = [string]$deploymentEvidence.snapshot.release_image.image_id
                source_commit = [string]$deploymentEvidence.snapshot.source.commit
                database_name = [string]$deploymentEvidence.snapshot.database_runtime.current_database
            }
        }
        acceptance_evidence = $null
    }

    if (-not [string]::IsNullOrWhiteSpace($acceptanceEvidenceFullPath)) {
        $evidenceDocument = [pscustomobject][ordered]@{
            schema_version = $script:Sprint6AAcceptanceEvidenceSchemaVersion
            evidence_kind = 'tessara.sprint-6a.smoke'
            status = 'passed'
            completed_at_utc = [DateTime]::UtcNow.ToString('o')
            expected_data_state = $ExpectedDataState
            base_url = $baseUrl
            runner = [pscustomobject][ordered]@{
                path = 'scripts/smoke.ps1'
                sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
            }
            checks = @(Get-Sprint6AAcceptanceExpectedChecks -Kind smoke)
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
                dataset_rows = [int]$dataset.rows.Count
                component_rows = [int]$componentTable.rows.Count
                seeded_visual_points = [int]$seededVisualBar.points.Count
                visual_points = [int]$visualBar.points.Count
            }
        }
        $publication = Publish-Sprint6AAcceptanceEvidence `
            -EvidencePath $acceptanceEvidenceFullPath `
            -DeploymentEvidencePath $deploymentEvidenceFullPath `
            -RunnerFilePath $PSCommandPath `
            -Evidence $evidenceDocument `
            -Overwrite:$OverwriteAcceptanceEvidence
        $result.acceptance_evidence = $publication
        Write-Host "Published Sprint 6A $ExpectedDataState smoke evidence: $($publication.evidence_path)" -ForegroundColor Green
    }

    $result | ConvertTo-Json -Depth 10
}
finally {
    $cleanupFailure = $null
    try {
        Complete-Sprint6AAcceptanceRunCleanup `
            -Sessions $currentRunSessions `
            -SensitivePaths $sensitiveTemporaryPaths `
            -LogoutAction ${function:Invoke-CurrentRunSessionLogout} `
            -EnvironmentSnapshot $environmentSnapshot `
            -FinalAttempt
    } catch {
        $cleanupFailure = $_
    }
    $serviceFailure = $null
    try {
        if ($null -ne $apiProcess -and -not $apiProcess.HasExited) {
            Stop-Process -Id $apiProcess.Id -Force
        }

        if (-not $KeepServices -and -not $UseExistingService) {
            docker compose down -v | Out-Host
            Assert-LastExitCode "docker compose down"
        }
    } catch {
        $serviceFailure = $_
    }
    if ($null -ne $cleanupFailure) {
        throw $cleanupFailure
    }
    if ($null -ne $serviceFailure) {
        throw $serviceFailure
    }
}

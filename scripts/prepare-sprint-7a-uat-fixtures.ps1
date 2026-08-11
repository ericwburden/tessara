[CmdletBinding()]
param(
    [string]$BaseUrl = "http://127.0.0.1:8086",
    [string]$AdminEmail = "admin@tessara.local",
    [string]$AdminPassword = "tessara-dev-admin",
    [string]$ScopedPassword = "tessara-sprint-7a-scoped",
    [string]$MixedPassword = "tessara-sprint-7a-mixed",
    [string]$NoAnalyticsPassword = "tessara-sprint-7a-restricted",
    [string]$ComposeProject = "tessara-sprint-7a",
    [string]$OutputPath,
    [switch]$OwnerControlledSeed,
    [switch]$VerifyOnly,
    [switch]$Overwrite,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

function Assert-Uuid([string]$Value, [string]$Label) {
    try { $null = [guid]::ParseExact($Value, "D") } catch { throw "$Label is not a canonical UUID." }
    $Value
}

function Get-PostgresContainer {
    [array]$containers = @(& docker ps --no-trunc `
        --filter "label=com.docker.compose.project=$ComposeProject" `
        --filter "label=com.docker.compose.service=postgres" `
        --format "{{.ID}}") | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    if ($LASTEXITCODE -ne 0 -or $containers.Count -ne 1 -or $containers[0] -notmatch '^[0-9a-f]{64}$') {
        throw "Expected exactly one running PostgreSQL container for Compose project '$ComposeProject'."
    }
    [string]$containers[0]
}

function Invoke-Postgres {
    param(
        [Parameter(Mandatory)][string]$Container,
        [Parameter(Mandatory)][ValidateSet("tessara_core", "tessara_module_components", "tessara_module_dashboards")][string]$Database,
        [Parameter(Mandatory)][string]$Sql,
        [switch]$Json
    )
    $arguments = @("exec", "-i", $Container, "psql", "-X", "-v", "ON_ERROR_STOP=1", "-U", "tessara_bootstrap", "-d", $Database)
    if ($Json) { $arguments += @("-qAt") }
    $output = $Sql | & docker @arguments
    if ($LASTEXITCODE -ne 0) { throw "PostgreSQL fixture operation failed in '$Database'." }
    if ($Json) { return (($output -join "`n").Trim() | ConvertFrom-Json) }
    $output
}

function Get-JsonBody([object]$Response, [string]$Label) {
    if ($Response.status -ne 200) { throw "$Label returned HTTP $($Response.status)." }
    $document = $Response.body | ConvertFrom-Json
    if ($document -is [Array]) {
        foreach ($entry in $document) { $entry }
    } else {
        $document
    }
}

function Ensure-Role {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][AllowEmptyCollection()][string[]]$CapabilityKeys,
        [Parameter(Mandatory)][object[]]$Capabilities,
        [Parameter(Mandatory)][object[]]$Roles,
        [Parameter(Mandatory)][string]$Token
    )
    $ids = @($CapabilityKeys | ForEach-Object {
        $key = $_
        $matches = @($Capabilities | Where-Object key -CEQ $key)
        if ($matches.Count -ne 1) { throw "Expected exactly one capability '$key'." }
        [string]$matches[0].id
    })
    $matches = @($Roles | Where-Object name -CEQ $Name)
    if ($matches.Count -gt 1) { throw "Role '$Name' is not unique." }
    if ($matches.Count -eq 0) {
        $created = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/roles" -Method POST -Token $Token -Body @{ name = $Name; capability_ids = $ids }
        if ($created.status -ne 200) { throw "Role '$Name' creation returned HTTP $($created.status)." }
        return [string](($created.body | ConvertFrom-Json).id)
    }
    $roleId = [string]$matches[0].id
    $detail = Get-JsonBody (Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/roles/$roleId" -Token $Token) "Role '$Name' detail"
    $current = @($detail.capabilities | ForEach-Object key | Sort-Object)
    $expected = @($CapabilityKeys | Sort-Object)
    if (($current -join "`n") -cne ($expected -join "`n")) {
        $updated = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/roles/$roleId" -Method PUT -Token $Token -Body @{ capability_ids = $ids }
        if ($updated.status -ne 200) { throw "Role '$Name' update returned HTTP $($updated.status)." }
    }
    $roleId
}

function Ensure-Actor {
    param(
        [Parameter(Mandatory)][string]$Email,
        [Parameter(Mandatory)][string]$DisplayName,
        [Parameter(Mandatory)][string]$Password,
        [Parameter(Mandatory)][string[]]$RoleIds,
        [Parameter(Mandatory)][object[]]$Users,
        [Parameter(Mandatory)][string]$Token
    )
    $matches = @($Users | Where-Object email -CEQ $Email)
    if ($matches.Count -gt 1) { throw "Actor '$Email' is not unique." }
    $payload = @{ email = $Email; display_name = $DisplayName; password = $Password; is_active = $true; role_ids = $RoleIds }
    $response = if ($matches.Count -eq 0) {
        Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users" -Method POST -Token $Token -Body $payload
    } else {
        $accountId = [string]$matches[0].id
        $detail = Get-JsonBody (Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users/$accountId" -Token $Token) "Actor '$Email' detail"
        $currentRoles = @($detail.roles | ForEach-Object id | Sort-Object)
        $expectedRoles = @($RoleIds | Sort-Object)
        $credentialResponse = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/login" -Method POST -Body @{ email = $Email; password = $Password }
        $credentialWorks = $credentialResponse.status -eq 200
        if ($credentialWorks) {
            $credentialToken = [string](($credentialResponse.body | ConvertFrom-Json).token)
            $credentialLogout = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $credentialToken
            if ($credentialLogout.status -notin 200,204) { throw "Actor '$Email' credential self-check cleanup failed." }
        }
        if ($detail.email -CEQ $Email -and $detail.display_name -CEQ $DisplayName -and $detail.is_active -and
            ($currentRoles -join "`n") -CEQ ($expectedRoles -join "`n") -and $credentialWorks) {
            return $accountId
        }
        Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users/$accountId" -Method PUT -Token $Token -Body $payload
    }
    if ($response.status -ne 200) { throw "Actor '$Email' preparation returned HTTP $($response.status)." }
    [string](($response.body | ConvertFrom-Json).id)
}

function Get-LiveInventory {
    param([Parameter(Mandatory)][string]$Container)
    $contract = Assert-Sprint7AFixtureContract
    $componentInventoryProjection = if ($script:SplitComponentOwnership) {
        "'components', NULL"
    } else {
        @"
  'components', (SELECT json_agg(json_build_object('version_id',cv.id,'type',cv.component_type,'authority_revision',cv.authority_revision) ORDER BY cv.id)
    FROM component_versions cv WHERE cv.id IN ('$($contract.component_versions.stat)'::uuid,'$($contract.component_versions.table)'::uuid,'$($contract.component_versions.chart)'::uuid,'$($contract.component_versions.blocked)'::uuid))
"@
    }
    $core = Invoke-Postgres -Container $Container -Database tessara_core -Json -Sql @"
SELECT json_build_object(
  'actors', (SELECT json_object_agg(a.email, a.id) FROM accounts a WHERE a.email IN
    ('$($contract.actors.administrator)','$($contract.actors.scoped_operator)','$($contract.actors.mixed_scope_operator)','$($contract.actors.no_analytics_actor)')),
  'role_assignments', (SELECT json_agg(json_build_object('email',a.email,'role',r.name,'node_id',ra.node_id) ORDER BY a.email,r.name,ra.node_id)
    FROM role_assignments ra JOIN accounts a ON a.id=ra.account_id JOIN roles r ON r.id=ra.role_id
    WHERE a.email IN ('$($contract.actors.scoped_operator)','$($contract.actors.mixed_scope_operator)','$($contract.actors.no_analytics_actor)')),
  'datasets', (SELECT json_agg(json_build_object('id',d.id,'name',d.name,'authority_revision',d.authority_revision,'row_count',dr.materialized_row_count) ORDER BY d.id)
    FROM datasets d JOIN dataset_revisions dr ON dr.dataset_id=d.id AND dr.status='published'
    WHERE d.id IN ('$($contract.datasets.four_tier)'::uuid,'$($contract.datasets.blocked)'::uuid)),
  'row_tiers', (SELECT json_agg(json_build_object('label',label,'tier',__restriction_tier) ORDER BY label)
    FROM dataset_materialized.dataset_major_01980000000270008000000000000003_v1),
  $componentInventoryProjection,
  'identifier_specimens', json_build_object(
    'known_blocked_exists',(SELECT EXISTS(SELECT 1 FROM datasets WHERE id='$($contract.identifier_specimens.known_blocked)'::uuid)),
    'random_absent',(SELECT NOT EXISTS(SELECT 1 FROM datasets WHERE id='$($contract.identifier_specimens.random)'::uuid))),
  'negative_services', json_build_object(
    'undeclared_definition_absent',(SELECT NOT EXISTS(SELECT 1 FROM module_definition_reservations WHERE definition_id='$($contract.actors.undeclared_service)')),
    'wrong_instance_absent',(SELECT NOT EXISTS(SELECT 1 FROM module_instances WHERE id='$($contract.actors.wrong_service_instance)'::uuid))),
  'security_revisions', (SELECT row_to_json(r) FROM (SELECT authorization_revision,organization_revision FROM core_security_revisions WHERE singleton=true) r)
);
"@
    if ($script:SplitComponentOwnership) {
        $componentInventory = Invoke-Postgres -Container $Container -Database tessara_module_components -Json -Sql @"
SELECT json_build_object(
  'components', (SELECT json_agg(json_build_object('version_id',cv.id,'type',cv.component_type,'authority_revision',cv.authority_revision) ORDER BY cv.id)
    FROM component_versions cv WHERE cv.id IN ('$($contract.component_versions.stat)'::uuid,'$($contract.component_versions.table)'::uuid,'$($contract.component_versions.chart)'::uuid,'$($contract.component_versions.blocked)'::uuid))
);
"@
        $core.components = $componentInventory.components
    }
    $dashboardIds = if ($OwnerControlledSeed) {
        "'$($contract.dashboards.mixed)'::uuid"
    } else {
        "'$($contract.dashboards.mixed)'::uuid,'$($contract.dashboards.blocked)'::uuid"
    }
    $dashboard = Invoke-Postgres -Container $Container -Database tessara_module_dashboards -Json -Sql @"
SELECT json_build_object(
  'dashboards', (SELECT json_agg(json_build_object('id',d.id,'authority_revision',d.authority_revision,'scope_nodes',(SELECT json_agg(node_id ORDER BY node_id) FROM dashboard_scope_nodes WHERE dashboard_id=d.id)) ORDER BY d.id)
    FROM dashboards d WHERE d.id IN ($dashboardIds)),
  'placements', (SELECT json_agg(json_build_object('id',p.id,'dashboard_id',p.dashboard_id,'component_version_id',p.component_reference->>'resource_id') ORDER BY p.id)
    FROM dashboard_placements p WHERE p.id IN ('$($contract.placements.stat)'::uuid,'$($contract.placements.table)'::uuid,'$($contract.placements.chart)'::uuid,'$($contract.placements.blocked)'::uuid))
);
"@
    [ordered]@{ contract = $contract; core = $core; dashboard = $dashboard }
}

function Test-IsInsideOwnerControlledSeedExclusion {
    param([Parameter(Mandatory)][System.Management.Automation.Language.Ast]$Ast)

    $ancestor = $Ast
    while ($null -ne $ancestor) {
        if ($ancestor -is [System.Management.Automation.Language.StatementBlockAst] -and
            $ancestor.Parent -is [System.Management.Automation.Language.IfStatementAst]) {
            foreach ($clause in $ancestor.Parent.Clauses) {
                if ($clause.Item2.Extent.StartOffset -ne $ancestor.Extent.StartOffset -or
                    $clause.Item2.Extent.EndOffset -ne $ancestor.Extent.EndOffset) {
                    continue
                }
                $unaryExpressions = @($clause.Item1.FindAll({
                    param($candidate)
                    $candidate -is [System.Management.Automation.Language.UnaryExpressionAst]
                }, $true))
                $variables = @($clause.Item1.FindAll({
                    param($candidate)
                    $candidate -is [System.Management.Automation.Language.VariableExpressionAst]
                }, $true))
                if ($unaryExpressions.Count -eq 1 -and
                    $variables.Count -eq 1 -and
                    $unaryExpressions[0].TokenKind -eq [System.Management.Automation.Language.TokenKind]::Not -and
                    $unaryExpressions[0].Child -eq $variables[0] -and
                    $variables[0].VariablePath.UserPath -ceq 'OwnerControlledSeed' -and
                    $clause.Item1.Extent.StartOffset -eq $unaryExpressions[0].Extent.StartOffset -and
                    $clause.Item1.Extent.EndOffset -eq $unaryExpressions[0].Extent.EndOffset) {
                    return $true
                }
            }
        }
        $ancestor = $ancestor.Parent
    }
    $false
}

function Test-Sprint8AOwnerControlledSeedBoundary {
    param([Parameter(Mandatory)][string]$Path)

    $tokens = $null
    $parseErrors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile(
        (Resolve-Path -LiteralPath $Path).Path,
        [ref]$tokens,
        [ref]$parseErrors
    )
    if (@($parseErrors).Count -ne 0) {
        throw "The owner-controlled fixture boundary cannot be audited because '$Path' does not parse."
    }

    $productMutationPattern = '(?im)\b(?:INSERT\s+INTO|UPDATE|DELETE\s+FROM|TRUNCATE(?:\s+TABLE)?|CREATE\s+TABLE)\s+(?:nodes|datasets|dataset_scope_nodes|dataset_revisions|dataset_major_materializations|dataset_materialized\.[A-Za-z0-9_.$()]+|components|component_versions|component_version_change_events|dashboard_organization_nodes|dashboards|dashboard_scope_nodes|dashboard_placements)\b'
    $productMutationStrings = @($ast.FindAll({
        param($candidate)
        ($candidate -is [System.Management.Automation.Language.StringConstantExpressionAst] -or
            $candidate -is [System.Management.Automation.Language.ExpandableStringExpressionAst]) -and
        $candidate.Extent.Text -match $productMutationPattern
    }, $true))
    if ($productMutationStrings.Count -eq 0) {
        throw 'The shared Sprint 7A fixture product setup disappeared instead of remaining behind its legacy-only boundary.'
    }
    $unguardedLines = @($productMutationStrings | Where-Object {
        -not (Test-IsInsideOwnerControlledSeedExclusion -Ast $_)
    } | ForEach-Object { $_.Extent.StartLineNumber })
    if ($unguardedLines.Count -ne 0) {
        throw "Sprint 8A OwnerControlledSeed can reach product-table mutation text at line(s) $($unguardedLines -join ', ')."
    }

    $securityMutationStrings = @($ast.FindAll({
        param($candidate)
        ($candidate -is [System.Management.Automation.Language.StringConstantExpressionAst] -or
            $candidate -is [System.Management.Automation.Language.ExpandableStringExpressionAst]) -and
        $candidate.Extent.Text -match '(?im)\b(?:DELETE\s+FROM|INSERT\s+INTO)\s+role_assignments\b'
    }, $true))
    if ($securityMutationStrings.Count -ne 1 -or
        (Test-IsInsideOwnerControlledSeedExclusion -Ast $securityMutationStrings[0])) {
        throw 'Owner-controlled fixture preparation must retain one Core role-assignment reconciliation outside the legacy product boundary.'
    }

    $readBackCommands = @($ast.FindAll({
        param($candidate)
        $candidate -is [System.Management.Automation.Language.CommandAst] -and
        $candidate.GetCommandName() -in @('Get-LiveInventory', 'Assert-LiveInventory')
    }, $true))
    foreach ($commandName in @('Get-LiveInventory', 'Assert-LiveInventory')) {
        $commands = @($readBackCommands | Where-Object { $_.GetCommandName() -ceq $commandName })
        if ($commands.Count -ne 1 -or (Test-IsInsideOwnerControlledSeedExclusion -Ast $commands[0])) {
            throw "Owner-controlled fixture verification must execute exactly one unguarded '$commandName' read-back command."
        }
    }
}

function Assert-LiveInventory([object]$Inventory) {
    $contract = $Inventory.contract
    foreach ($email in @($contract.actors.administrator, $contract.actors.scoped_operator, $contract.actors.mixed_scope_operator, $contract.actors.no_analytics_actor)) {
        if ([string]::IsNullOrWhiteSpace([string]$Inventory.core.actors.$email)) { throw "Required UAT actor '$email' is missing." }
    }
    if (@($Inventory.core.datasets).Count -ne 2) { throw "UAT Dataset inventory is incomplete." }
    if (@($Inventory.core.components).Count -ne 4) { throw "UAT ComponentVersion inventory is incomplete." }
    $expectedDashboardCount = if ($OwnerControlledSeed) { 1 } else { 2 }
    if (@($Inventory.dashboard.dashboards).Count -ne $expectedDashboardCount) { throw "UAT Dashboard inventory is incomplete." }
    if (@($Inventory.dashboard.placements).Count -ne 4) { throw "Mixed Dashboard placement inventory is incomplete." }
    if (-not $Inventory.core.identifier_specimens.known_blocked_exists -or -not $Inventory.core.identifier_specimens.random_absent) {
        throw "Known-blocked and random identifier specimens are not semantically distinct."
    }
    if (-not $Inventory.core.negative_services.undeclared_definition_absent -or -not $Inventory.core.negative_services.wrong_instance_absent) {
        throw "Undeclared and wrong-service negative fixtures are not cleanly isolated."
    }
    $tiers = @($Inventory.core.row_tiers | ForEach-Object tier | Sort-Object -Unique)
    if (($tiers -join ',') -cne 'confidential,internal,public,restricted') { throw "Four-tier Dataset rows are incomplete." }
    $types = @($Inventory.core.components | ForEach-Object type | Sort-Object -Unique)
    foreach ($type in @('bar','stat_card','table')) {
        if ($types -cnotcontains $type) { throw "UAT ComponentVersion type '$type' is missing." }
    }
    if ((@($Inventory.core.datasets | Measure-Object authority_revision -Maximum).Maximum) -le 1 -or
        (@($Inventory.core.components | Measure-Object authority_revision -Maximum).Maximum) -le 1 -or
        (@($Inventory.dashboard.dashboards | Measure-Object authority_revision -Maximum).Maximum) -le 1) {
        throw "Provider authority freshness specimens are incomplete."
    }
    $mixed = @($Inventory.core.role_assignments | Where-Object email -CEQ $contract.actors.mixed_scope_operator)
    $base = @($mixed | Where-Object role -CEQ 'sprint-7a-mixed-base')
    $tier = @($mixed | Where-Object role -CEQ 'sprint-7a-mixed-confidential')
    if ($base.Count -ne 1 -or [string]$base[0].node_id -cne [string]$contract.scope_nodes.subtree_a -or
        $tier.Count -ne 1 -or [string]$tier[0].node_id -cne [string]$contract.scope_nodes.subtree_b) {
        throw "Mixed-scope actor does not have the exact disjoint capability assignments."
    }
    if ([int64]$Inventory.core.security_revisions.authorization_revision -le 1 -or
        [int64]$Inventory.core.security_revisions.organization_revision -le 1) {
        throw "Current and deliberately stale security revision specimens cannot be derived."
    }
}

if ($SelfTest) {
    Test-Sprint7AAcceptanceContract
    $contract = Assert-Sprint7AFixtureContract
    foreach ($group in @($contract.scope_nodes, $contract.datasets, $contract.component_versions, $contract.dashboards, $contract.placements)) {
        foreach ($property in $group.psobject.Properties) {
            $null = Assert-Uuid ([string]$property.Value) "Fixture identity '$($property.Name)'"
        }
    }
    $source = Get-Content -LiteralPath $PSCommandPath -Raw
    if ($source -cnotmatch 'status,lifecycle_state,config,published_at,authority_revision,resource_revision' -or
        $source -cnotmatch "'published','active'.*now\(\),2,1" -or
        $source -cnotmatch "component_versions\.lifecycle_state IS DISTINCT FROM 'active'") {
        throw "Published ComponentVersion fixtures must declare the active lifecycle state and initial resource revision."
    }
    if ($source -cnotmatch 'tessara_module_components' -or
        $source -cnotmatch 'SplitComponentOwnership' -or
        $source -cnotmatch "kind='module_instance'" -or
        $source -cnotmatch "resource_type='tessara.components.component_version'") {
        throw "The shared fixture preparer must preserve independently owned Component storage and references for Sprint 8A."
    }
    Test-Sprint8AOwnerControlledSeedBoundary -Path $PSCommandPath
    Write-Host "Sprint 7A UAT fixture preparation self-test passed."
    return
}

$contract = Assert-Sprint7AFixtureContract
$script:SplitComponentOwnership = $ComposeProject -ceq "tessara-sprint-8a"
$container = Get-PostgresContainer
if (-not $VerifyOnly) {
    $token = Get-Sprint7AToken -BaseUrl $BaseUrl -Email $AdminEmail -Password $AdminPassword
    $capabilities = @(Get-JsonBody (Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/capabilities" -Token $token) "Capability catalog")
    $roles = @(Get-JsonBody (Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/roles" -Token $token) "Role catalog")
    $scopedRole = Ensure-Role -Name "sprint-7a-scoped-operator" -CapabilityKeys @('datasets:read','datasets:read_restricted','components:read','dashboards:read') -Capabilities $capabilities -Roles $roles -Token $token
    $mixedBaseRole = Ensure-Role -Name "sprint-7a-mixed-base" -CapabilityKeys @('datasets:read','components:read','dashboards:read') -Capabilities $capabilities -Roles $roles -Token $token
    $mixedTierRole = Ensure-Role -Name "sprint-7a-mixed-confidential" -CapabilityKeys @('datasets:read_confidential') -Capabilities $capabilities -Roles $roles -Token $token
    $noAnalyticsRole = Ensure-Role -Name "sprint-7a-no-analytics" -CapabilityKeys @() -Capabilities $capabilities -Roles $roles -Token $token
    $users = @(Get-JsonBody (Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/admin/users" -Token $token) "User catalog")
    $scopedId = Ensure-Actor -Email $contract.actors.scoped_operator -DisplayName "Sprint 7A Scoped Operator" -Password $ScopedPassword -RoleIds @($scopedRole) -Users $users -Token $token
    $mixedId = Ensure-Actor -Email $contract.actors.mixed_scope_operator -DisplayName "Sprint 7A Mixed-Scope Operator" -Password $MixedPassword -RoleIds @($mixedBaseRole,$mixedTierRole) -Users $users -Token $token
    $restrictedId = Ensure-Actor -Email $contract.actors.no_analytics_actor -DisplayName "Sprint 7A No-Analytics Actor" -Password $NoAnalyticsPassword -RoleIds @($noAnalyticsRole) -Users $users -Token $token
    foreach ($pair in @(@($scopedId,$contract.scope_nodes.subtree_a),@($restrictedId,$contract.scope_nodes.subtree_a))) {
        $accessPath = "/api/admin/users/$($pair[0])/access"
        $currentAccess = Get-JsonBody (Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $accessPath -Token $token) "Actor access"
        $currentScope = @($currentAccess.scope_nodes | ForEach-Object node_id)
        if ($currentScope.Count -ne 1 -or [string]$currentScope[0] -CNE [string]$pair[1] -or @($currentAccess.delegations).Count -ne 0) {
            $access = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path $accessPath -Method PUT -Token $token -Body @{ scope_node_ids = @($pair[1]); delegate_account_ids = @() }
            if ($access.status -ne 200) { throw "Actor scope preparation returned HTTP $($access.status)." }
        }
    }
    foreach ($value in @($scopedId,$mixedId,$restrictedId,$scopedRole,$mixedBaseRole,$mixedTierRole,$noAnalyticsRole)) { $null = Assert-Uuid $value "Prepared identity" }
    $roleQuote = '$role$'

    # Sprint 8A product state comes from signed owner bootstraps. This branch is
    # retained only for the Sprint 7A fixture topology.
    if (-not $OwnerControlledSeed) {
    $referenceRevision = Invoke-Postgres -Container $container -Database tessara_core -Json -Sql "SELECT json_build_object('id',id,'table',materialized_table) FROM dataset_revisions WHERE dataset_id='$($contract.datasets.four_tier)'::uuid AND status='published'"
    $referenceTable = [string]$referenceRevision.table
    if ($referenceTable -notmatch '^dataset_[0-9a-f]{32}$') { throw "Reference Dataset materialized table identity is invalid." }
    $sqlQuote = '$uat$'
    $blockedNode = Invoke-Postgres -Container $container -Database tessara_core -Json -Sql "SELECT COALESCE((SELECT json_build_object('name',name) FROM nodes WHERE id='$($contract.scope_nodes.subtree_b)'::uuid),'null'::json)"
    $nodeSql = if ($null -eq $blockedNode -or [string]$blockedNode.name -CNE 'Tessara UAT Blocked Organization') {
@"
INSERT INTO nodes(id,node_type_id,parent_node_id,name)
SELECT '$($contract.scope_nodes.subtree_b)'::uuid,node_type_id,NULL,'Tessara UAT Blocked Organization' FROM nodes WHERE id='$($contract.scope_nodes.subtree_a)'::uuid
ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name WHERE nodes.name IS DISTINCT FROM EXCLUDED.name;
"@
    } else { '' }
    $legacyComponentSql = if ($script:SplitComponentOwnership) { '' } else {
@"
INSERT INTO components(id,name,slug,description) VALUES
  ('$($script:Sprint7AFixture.chart_component_id)'::uuid,'Sprint 7A Tier Chart','sprint-7a-tier-chart','UAT fixture'),
  ('$($script:Sprint7AFixture.blocked_component_id)'::uuid,'Sprint 7A Blocked Component','sprint-7a-blocked-component','UAT fixture')
ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug,description=EXCLUDED.description;
INSERT INTO component_versions(id,component_id,dataset_id,dataset_version_major,binding_mode,component_type,version_number,version_label,version_note,status,lifecycle_state,config,published_at,authority_revision,resource_revision) VALUES
  ('$($contract.component_versions.chart)'::uuid,'$($script:Sprint7AFixture.chart_component_id)'::uuid,'$($contract.datasets.four_tier)'::uuid,1,'major_line','bar',1,'1.0.0','UAT fixture','published','active','{"mode":"summary","summary_field":"label","summary_type":"count","category_field":"label","sort_field":"summary_value","sort_direction":"desc","number_of_points":20,"value_format":"integer"}'::jsonb,now(),2,1),
  ('$($contract.component_versions.blocked)'::uuid,'$($script:Sprint7AFixture.blocked_component_id)'::uuid,'$($contract.datasets.blocked)'::uuid,1,'major_line','table',1,'1.0.0','UAT fixture','published','active','{"visible_columns":["label"]}'::jsonb,now(),2,1)
ON CONFLICT(id) DO UPDATE SET config=EXCLUDED.config,lifecycle_state='active',authority_revision=GREATEST(component_versions.authority_revision,2)
WHERE component_versions.config IS DISTINCT FROM EXCLUDED.config OR component_versions.lifecycle_state IS DISTINCT FROM 'active' OR component_versions.authority_revision < 2;
"@
    }
    $coreSql = @"
BEGIN;
SET LOCAL client_min_messages TO warning;
$nodeSql

UPDATE dataset_revisions SET
  generated_sql=${sqlQuote}SELECT * FROM (VALUES
    ('uat7a-public','public','UAT7A-PUBLIC'),
    ('uat7a-internal','internal','UAT7A-INTERNAL'),
    ('uat7a-restricted','restricted','UAT7A-RESTRICTED'),
    ('uat7a-confidential','confidential','UAT7A-CONFIDENTIAL-BLOCKED')
  ) AS fixture(__row_id,__restriction_tier,label)
  UNION ALL
  SELECT 'uat7a-page-' || page_row, 'public', 'UAT7A-PAGE-' || lpad(page_row::text, 2, '0')
  FROM generate_series(1,26) AS paging_fixture(page_row)${sqlQuote},
  restriction_policy='{"internal_field_key":"tier_internal","restricted_field_key":"tier_restricted","confidential_field_key":"tier_confidential"}'::jsonb,
  materialized_row_count=30, materialized_at=now()
WHERE id='$($referenceRevision.id)'::uuid AND (
  generated_sql NOT LIKE '%UAT7A-CONFIDENTIAL-BLOCKED%'
  OR generated_sql NOT LIKE '%UAT7A-PAGE-%'
  OR restriction_policy IS DISTINCT FROM '{"internal_field_key":"tier_internal","restricted_field_key":"tier_restricted","confidential_field_key":"tier_confidential"}'::jsonb
  OR materialized_row_count IS DISTINCT FROM 30
);
TRUNCATE dataset_materialized.$referenceTable;
INSERT INTO dataset_materialized.$referenceTable(__row_id,__restriction_tier,label) VALUES
  ('uat7a-public','public','UAT7A-PUBLIC'),('uat7a-internal','internal','UAT7A-INTERNAL'),
  ('uat7a-restricted','restricted','UAT7A-RESTRICTED'),('uat7a-confidential','confidential','UAT7A-CONFIDENTIAL-BLOCKED');
INSERT INTO dataset_materialized.$referenceTable(__row_id,__restriction_tier,label)
SELECT 'uat7a-page-' || page_row, 'public', 'UAT7A-PAGE-' || lpad(page_row::text, 2, '0')
FROM generate_series(1,26) AS paging_fixture(page_row);
TRUNCATE dataset_materialized.dataset_major_01980000000270008000000000000003_v1;
INSERT INTO dataset_materialized.dataset_major_01980000000270008000000000000003_v1
  (__row_id,__restriction_tier,__source_dataset_revision_id,__source_dataset_version_major,__source_dataset_version_minor,__source_dataset_version_patch,__source_dataset_semantic_version,label)
SELECT '$($referenceRevision.id):'||__row_id,__restriction_tier,'$($referenceRevision.id)'::uuid,1,0,0,'v1.0.0',label
FROM dataset_materialized.$referenceTable;
UPDATE dataset_major_materializations SET materialized_row_count=30,materialized_at=now(),rebuild_status='ready'
WHERE dataset_id='$($contract.datasets.four_tier)'::uuid AND version_major=1;

INSERT INTO datasets(id,name,slug,grain,authority_revision) VALUES('$($contract.datasets.blocked)'::uuid,'Sprint 7A Blocked Dataset','sprint-7a-blocked-dataset','node',2)
ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug,authority_revision=GREATEST(datasets.authority_revision,2)
WHERE datasets.name IS DISTINCT FROM EXCLUDED.name OR datasets.slug IS DISTINCT FROM EXCLUDED.slug OR datasets.authority_revision < 2;
INSERT INTO dataset_scope_nodes(dataset_id,node_id) VALUES('$($contract.datasets.blocked)'::uuid,'$($contract.scope_nodes.subtree_b)'::uuid) ON CONFLICT DO NOTHING;
INSERT INTO dataset_revisions(id,dataset_id,version_number,version_label,version_major,version_minor,version_patch,semantic_bump,started_new_major_line,status,published_at,initial_source,operations,generated_sql,output_fields,definition_metadata,materialized_schema,materialized_table,materialized_row_count,materialized_at)
VALUES('$($script:Sprint7AFixture.blocked_dataset_revision_id)'::uuid,'$($contract.datasets.blocked)'::uuid,1,'1.0.0',1,0,0,'INITIAL',true,'published',now(),'{"kind":"uat_fixture"}'::jsonb,'[]'::jsonb,
  'SELECT ''blocked-row''::text AS __row_id, ''public''::text AS __restriction_tier, ''UAT7A-BLOCKED-DATASET''::text AS label',
  '[{"id":"01980000-0002-7000-8000-00000000000a","key":"label","label":"Label","source_alias":"uat_fixture","source_field_key":"label","field_type":"text","position":0}]'::jsonb,
  jsonb_build_object('name','Sprint 7A Blocked Dataset','slug','sprint-7a-blocked-dataset','grain','node','visibility_node_ids',jsonb_build_array('$($contract.scope_nodes.subtree_b)'::uuid)),
  'dataset_materialized','dataset_01980000000270008000000000000009',1,now())
ON CONFLICT(id) DO NOTHING;
CREATE TABLE IF NOT EXISTS dataset_materialized.dataset_01980000000270008000000000000009(__row_id text,__restriction_tier text,label text);
TRUNCATE dataset_materialized.dataset_01980000000270008000000000000009;
INSERT INTO dataset_materialized.dataset_01980000000270008000000000000009 VALUES('blocked-row','public','UAT7A-BLOCKED-DATASET');
CREATE TABLE IF NOT EXISTS dataset_materialized.dataset_major_01980000000270008000000000000008_v1(__row_id text,__restriction_tier text,__source_dataset_revision_id uuid,__source_dataset_version_major integer,__source_dataset_version_minor integer,__source_dataset_version_patch integer,__source_dataset_semantic_version text,label text);
TRUNCATE dataset_materialized.dataset_major_01980000000270008000000000000008_v1;
INSERT INTO dataset_materialized.dataset_major_01980000000270008000000000000008_v1 VALUES('01980000-0002-7000-8000-000000000009:blocked-row','public','$($script:Sprint7AFixture.blocked_dataset_revision_id)'::uuid,1,0,0,'v1.0.0','UAT7A-BLOCKED-DATASET');
INSERT INTO dataset_major_materializations(dataset_id,version_major,materialized_schema,materialized_table,materialized_row_count,materialized_at,rebuild_status)
VALUES('$($contract.datasets.blocked)'::uuid,1,'dataset_materialized','dataset_major_01980000000270008000000000000008_v1',1,now(),'ready')
ON CONFLICT(dataset_id,version_major) DO UPDATE SET materialized_row_count=1,materialized_at=now(),rebuild_status='ready';

$legacyComponentSql
COMMIT;
"@
    $null = Invoke-Postgres -Container $container -Database tessara_core -Sql $coreSql

    $componentOwner = $null
    if ($script:SplitComponentOwnership) {
        $componentOwner = Invoke-Postgres -Container $container -Database tessara_core -Json -Sql @"
SELECT json_build_object('installation_id',installation_id,'module_instance_id',id)
FROM module_instances
WHERE definition_id='tessara.components' AND identity_state='live' AND installed AND deployed AND enabled;
"@
        if ($null -eq $componentOwner -or [string]::IsNullOrWhiteSpace([string]$componentOwner.module_instance_id)) {
            throw "Sprint 8A Component Module Instance ownership could not be resolved."
        }
        $datasetReference = { param([string]$DatasetId) (@{ reference=@{ installation_id=[string]$componentOwner.installation_id; owner=@{kind='core_installation';installation_id=[string]$componentOwner.installation_id};resource_type='tessara.transition.dataset_major_line';resource_id="$DatasetId@1" } } | ConvertTo-Json -Compress) }
        $componentSql = @"
BEGIN;
SET LOCAL client_min_messages TO warning;
INSERT INTO components(id,external_key,name,slug,description) VALUES
  ('$($script:Sprint7AFixture.metric_component_id)'::uuid,'sprint-7a-metric-card','Reference Metric Card','sprint-7a-metric-card','UAT fixture'),
  ('$($script:Sprint7AFixture.table_component_id)'::uuid,'sprint-7a-record-table','Reference Records Table','sprint-7a-record-table','UAT fixture'),
  ('$($script:Sprint7AFixture.chart_component_id)'::uuid,'sprint-7a-tier-chart','Sprint 7A Tier Chart','sprint-7a-tier-chart','UAT fixture'),
  ('$($script:Sprint7AFixture.blocked_component_id)'::uuid,'sprint-7a-blocked-component','Sprint 7A Blocked Component','sprint-7a-blocked-component','UAT fixture')
ON CONFLICT(id) DO UPDATE SET external_key=EXCLUDED.external_key,name=EXCLUDED.name,slug=EXCLUDED.slug,description=EXCLUDED.description;
INSERT INTO component_versions(id,component_id,dataset_reference,dataset_scope_node_ids,component_type,status,lifecycle_state,resource_revision,authority_revision,version_number,version_label,version_note,config) VALUES
  ('$($contract.component_versions.stat)'::uuid,'$($script:Sprint7AFixture.metric_component_id)'::uuid,'$(& $datasetReference $contract.datasets.four_tier)'::jsonb,ARRAY['$($contract.scope_nodes.subtree_a)'::uuid],'stat_card','published','active',1,2,1,'1.0.0','UAT fixture','{"label":"Reference metric","summary_type":"row_count","summary_field":""}'::jsonb),
  ('$($contract.component_versions.table)'::uuid,'$($script:Sprint7AFixture.table_component_id)'::uuid,'$(& $datasetReference $contract.datasets.four_tier)'::jsonb,ARRAY['$($contract.scope_nodes.subtree_a)'::uuid],'table','published','active',1,2,1,'1.0.0','UAT fixture','{"visible_columns":["label"]}'::jsonb),
  ('$($contract.component_versions.chart)'::uuid,'$($script:Sprint7AFixture.chart_component_id)'::uuid,'$(& $datasetReference $contract.datasets.four_tier)'::jsonb,ARRAY['$($contract.scope_nodes.subtree_a)'::uuid],'bar','published','active',1,2,1,'1.0.0','UAT fixture','{"mode":"summary","summary_field":"label","summary_type":"count","category_field":"label","sort_field":"summary_value","sort_direction":"desc","number_of_points":20,"value_format":"integer"}'::jsonb),
  ('$($contract.component_versions.blocked)'::uuid,'$($script:Sprint7AFixture.blocked_component_id)'::uuid,'$(& $datasetReference $contract.datasets.blocked)'::jsonb,ARRAY['$($contract.scope_nodes.subtree_b)'::uuid],'table','published','active',1,2,1,'1.0.0','UAT fixture','{"visible_columns":["label"]}'::jsonb)
ON CONFLICT(id) DO UPDATE SET dataset_reference=EXCLUDED.dataset_reference,dataset_scope_node_ids=EXCLUDED.dataset_scope_node_ids,config=EXCLUDED.config,lifecycle_state='active',authority_revision=GREATEST(component_versions.authority_revision,2)
WHERE component_versions.dataset_reference IS DISTINCT FROM EXCLUDED.dataset_reference OR component_versions.dataset_scope_node_ids IS DISTINCT FROM EXCLUDED.dataset_scope_node_ids OR component_versions.config IS DISTINCT FROM EXCLUDED.config OR component_versions.lifecycle_state IS DISTINCT FROM 'active' OR component_versions.authority_revision < 2;
COMMIT;
"@
        $null = Invoke-Postgres -Container $container -Database tessara_module_components -Sql $componentSql
    }

    $componentReference = if ($script:SplitComponentOwnership) {
        { param([string]$VersionId) (@{ installation_id=[string]$componentOwner.installation_id; owner=@{kind='module_instance';installation_id=[string]$componentOwner.installation_id;module_instance_id=[string]$componentOwner.module_instance_id};resource_type='tessara.components.component_version';resource_id=$VersionId } | ConvertTo-Json -Compress) }
    } else {
        { param([string]$VersionId) (@{ installation_id=$script:Sprint7AFixture.installation_id; owner=@{kind='core_installation';installation_id=$script:Sprint7AFixture.installation_id};resource_type='tessara.transition.component_version';resource_id=$VersionId } | ConvertTo-Json -Compress) }
    }
    $chartReference = & $componentReference $contract.component_versions.chart
    $blockedReference = & $componentReference $contract.component_versions.blocked
    $dashboardSql = @"
BEGIN;
SET LOCAL client_min_messages TO warning;
INSERT INTO dashboard_organization_nodes(node_id,node_name,node_type_name,parent_node_id,node_path,active,projection_revision)
VALUES('$($contract.scope_nodes.subtree_b)'::uuid,'Tessara UAT Blocked Organization','Organization',NULL,'Tessara UAT Blocked Organization',true,1)
ON CONFLICT(node_id) DO UPDATE SET node_name=EXCLUDED.node_name,active=true;
INSERT INTO dashboards(id,name,description,authority_revision) VALUES('$($contract.dashboards.blocked)'::uuid,'Sprint 7A Blocked Dashboard','UAT blocked Dashboard fixture',2)
ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,authority_revision=GREATEST(dashboards.authority_revision,2)
WHERE dashboards.name IS DISTINCT FROM EXCLUDED.name OR dashboards.authority_revision < 2;
INSERT INTO dashboard_scope_nodes(dashboard_id,node_id) VALUES('$($contract.dashboards.blocked)'::uuid,'$($contract.scope_nodes.subtree_b)'::uuid) ON CONFLICT DO NOTHING;
INSERT INTO dashboard_placements(id,dashboard_id,component_reference,position,config) VALUES
  ('$($contract.placements.chart)'::uuid,'$($contract.dashboards.mixed)'::uuid,'$chartReference'::jsonb,96,'{"width":6,"height":4,"placement_key":"tier-chart"}'::jsonb),
  ('$($contract.placements.blocked)'::uuid,'$($contract.dashboards.mixed)'::uuid,'$blockedReference'::jsonb,144,'{"width":6,"height":4,"placement_key":"blocked-scope"}'::jsonb)
ON CONFLICT(id) DO UPDATE SET component_reference=EXCLUDED.component_reference,config=EXCLUDED.config
WHERE dashboard_placements.component_reference IS DISTINCT FROM EXCLUDED.component_reference OR dashboard_placements.config IS DISTINCT FROM EXCLUDED.config;
COMMIT;
"@
    $null = Invoke-Postgres -Container $container -Database tessara_module_dashboards -Sql $dashboardSql
    }
    # These scoped role assignments are Core security fixtures, not product
    # seed. They remain active for both Sprint 7A and Sprint 8A.
    $securitySql = @"
BEGIN;
SET LOCAL client_min_messages TO warning;
DO ${roleQuote}
BEGIN
  IF (SELECT jsonb_agg(jsonb_build_array(account_id,role_id,node_id) ORDER BY account_id,role_id,node_id)
      FROM role_assignments WHERE account_id IN ('$scopedId'::uuid,'$mixedId'::uuid,'$restrictedId'::uuid))
     IS DISTINCT FROM (SELECT jsonb_agg(jsonb_build_array(account_id,role_id,node_id) ORDER BY account_id,role_id,node_id) FROM (VALUES
       ('$scopedId'::uuid,'$scopedRole'::uuid,'$($contract.scope_nodes.subtree_a)'::uuid),
       ('$mixedId'::uuid,'$mixedBaseRole'::uuid,'$($contract.scope_nodes.subtree_a)'::uuid),
       ('$mixedId'::uuid,'$mixedTierRole'::uuid,'$($contract.scope_nodes.subtree_b)'::uuid),
       ('$restrictedId'::uuid,'$noAnalyticsRole'::uuid,'$($contract.scope_nodes.subtree_a)'::uuid)
     ) expected(account_id,role_id,node_id)) THEN
    DELETE FROM role_assignments WHERE account_id IN ('$scopedId'::uuid,'$mixedId'::uuid,'$restrictedId'::uuid);
    INSERT INTO role_assignments(account_id,role_id,node_id) VALUES
      ('$scopedId'::uuid,'$scopedRole'::uuid,'$($contract.scope_nodes.subtree_a)'::uuid),
      ('$mixedId'::uuid,'$mixedBaseRole'::uuid,'$($contract.scope_nodes.subtree_a)'::uuid),
      ('$mixedId'::uuid,'$mixedTierRole'::uuid,'$($contract.scope_nodes.subtree_b)'::uuid),
      ('$restrictedId'::uuid,'$noAnalyticsRole'::uuid,'$($contract.scope_nodes.subtree_a)'::uuid);
  END IF;
END
${roleQuote};
COMMIT;
"@
    $null = Invoke-Postgres -Container $container -Database tessara_core -Sql $securitySql
    $logout = Invoke-Sprint7ARequest -BaseUrl $BaseUrl -Path "/api/auth/logout" -Method DELETE -Token $token
    if ($logout.status -notin 200,204) { throw "Administrator fixture session cleanup returned HTTP $($logout.status)." }
}

$inventory = Get-LiveInventory -Container $container
Assert-LiveInventory $inventory
$security = $inventory.core.security_revisions
$datasetRevision = [int64](@($inventory.core.datasets | Measure-Object authority_revision -Maximum).Maximum)
$componentRevision = [int64](@($inventory.core.components | Measure-Object authority_revision -Maximum).Maximum)
$dashboardRevision = [int64](@($inventory.dashboard.dashboards | Measure-Object authority_revision -Maximum).Maximum)
$result = [ordered]@{
    schema_version = 1
    evidence_kind = "tessara.sprint-7a.uat-fixture-inventory"
    generated_at = [DateTimeOffset]::UtcNow.ToString("o")
    compose_project = $ComposeProject
    contract_sha256 = Get-Sprint7AFileSha256 -Path $script:Sprint7AFixtureContractPath
    inventory = $inventory
    freshness_specimens = [ordered]@{
        current = [ordered]@{
            authorization_revision = [int64]$security.authorization_revision
            organization_revision = [int64]$security.organization_revision
            dataset_authority_revision = $datasetRevision
            component_authority_revision = $componentRevision
            dashboard_authority_revision = $dashboardRevision
        }
        stale = [ordered]@{
            authorization_revision = [int64]$security.authorization_revision - 1
            organization_revision = [int64]$security.organization_revision - 1
            dataset_authority_revision = $datasetRevision - 1
            component_authority_revision = $componentRevision - 1
            dashboard_authority_revision = $dashboardRevision - 1
        }
        secret_grant_bytes_retained = $false
    }
    secrets_retained = $false
    passed = $true
}
if (-not [string]::IsNullOrWhiteSpace($OutputPath)) {
    Publish-Sprint7AEvidence -Document $result -OutputPath $OutputPath -Overwrite:$Overwrite | Out-Null
}
$result | ConvertTo-Json -Depth 30

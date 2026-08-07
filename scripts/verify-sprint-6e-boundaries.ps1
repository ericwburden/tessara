[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

$forbidden = & rg -n `
    "tessara-web-dashboards|tessara_web_dashboards|tessara-web\s*=|tessara-api|tessara-core|tessara-web-components" `
    (Join-Path $repoRoot "crates/tessara-dashboard-module") `
    (Join-Path $repoRoot "crates/tessara-dashboard-ui") `
    (Join-Path $repoRoot "crates/tessara-dashboard-placement-renderer") `
    (Join-Path $repoRoot "crates/tessara-dashboards") `
    (Join-Path $repoRoot "crates/tessara-components-contract") `
    -g "!target/**"
if ($LASTEXITCODE -eq 0 -or $forbidden) {
    throw "Dashboard release boundary contains a forbidden root/Core/feature dependency:`n$forbidden"
}
if ($LASTEXITCODE -ne 1) {
    throw "Dashboard boundary audit failed to execute."
}

$placementRendererRoot = Join-Path $repoRoot "crates/tessara-dashboard-placement-renderer"
$copiedComponentResponses = `
    "tessara-datasets-contract|tessara_datasets_contract|struct\s+Component(Table|TablePagination|TableColumn|TableRow|Visual|StatValue|VisualPoint|VisualSlice)\b"
$rendererBoundaryViolations = & rg -n `
    $copiedComponentResponses `
    $placementRendererRoot `
    -g "!target/**"
if ($LASTEXITCODE -eq 0 -or $rendererBoundaryViolations) {
    throw "Dashboard placement rendering must consume Component execution responses through tessara-components-contract; direct Dataset-contract dependencies and copied Component response DTOs are forbidden:`n$rendererBoundaryViolations"
}
if ($LASTEXITCODE -ne 1) {
    throw "Dashboard placement-renderer contract audit failed to execute."
}

$placementRendererManifest = Get-Content -LiteralPath `
    (Join-Path $placementRendererRoot "Cargo.toml") -Raw
$placementRendererSource = Get-ChildItem -LiteralPath `
    (Join-Path $placementRendererRoot "src") -Filter "*.rs" -File |
    Sort-Object FullName |
    ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw } |
    Out-String
if ($placementRendererManifest -notmatch '(?m)^tessara-components-contract\s*=' -or
    $placementRendererSource -notmatch 'tessara_components_contract') {
    throw "Dashboard placement rendering must depend on and consume the canonical tessara-components-contract response types."
}

$componentsContractSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-components-contract/src/lib.rs") -Raw
$componentsContractManifest = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-components-contract/Cargo.toml") -Raw
if ($componentsContractManifest -match 'tessara-datasets-contract' -or
    $componentsContractSource -match 'pub\s+dataset_reference\s*:') {
    throw "The Component render contract must not expose or depend on the unused Dataset provider identity."
}
foreach ($requiredContractFragment in @(
    'pub enum ComponentRenderResponse',
    'pub struct ComponentTableResponse',
    'pub struct ComponentVisualResponse',
    'pub enum ComponentRenderKind'
)) {
    if (-not $componentsContractSource.Contains($requiredContractFragment)) {
        throw "The Components contract is missing canonical render boundary '$requiredContractFragment'."
    }
}

$componentProviderSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-component-module/src/provider.rs") -Raw
$componentProductSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-component-module/src/product.rs") -Raw
$componentProviderTestBoundary = $componentProviderSource.IndexOf('#[cfg(test)]', [StringComparison]::Ordinal)
$componentProviderProductionSource = if ($componentProviderTestBoundary -ge 0) {
    $componentProviderSource.Substring(0, $componentProviderTestBoundary)
} else {
    $componentProviderSource
}
if ($componentProviderSource -notmatch 'Result<ComponentRenderResponse,\s*ComponentModuleError>' -or
    $componentProviderSource -notmatch 'Result<Json<ComponentRenderResponse>,\s*ComponentModuleError>' -or
    $componentProductSource -notmatch 'Result<Json<ComponentRenderResponse>,\s*ComponentModuleError>') {
    throw "Component provider and product render routes must produce the canonical typed ComponentRenderResponse."
}
if ([regex]::Matches($componentProviderProductionSource, '(?m)^\s+body: Bytes,\s*$').Count -ne 3 -or
    [regex]::Matches($componentProviderProductionSource, 'serde_json::from_slice\(&body\)').Count -ne 2 -or
    [regex]::Matches($componentProviderProductionSource, 'require_json_content_type\(&headers\)').Count -ne 2 -or
    $componentProviderProductionSource -match 'serde_json::to_vec\(&request\)' -or
    -not $componentProviderSource.Contains('fn exact_body_routes_retain_json_content_type_enforcement()')) {
    throw "Component provider routes must validate signed service requests against the exact inbound body before typed request decoding."
}

$componentResolveStart = $componentProviderSource.IndexOf('async fn resolve(', [StringComparison]::Ordinal)
$componentCatalogStart = $componentProviderSource.IndexOf('async fn catalog(', $componentResolveStart, [StringComparison]::Ordinal)
$componentRenderStart = $componentProviderSource.IndexOf('async fn render(', $componentCatalogStart, [StringComparison]::Ordinal)
$componentRenderEnd = $componentProviderSource.IndexOf('fn missing_policy(', $componentRenderStart, [StringComparison]::Ordinal)
$providerValidationStart = $componentProviderSource.IndexOf('async fn validate_provider_request(', [StringComparison]::Ordinal)
$providerValidationEnd = $componentProviderSource.IndexOf('async fn load_version(', $providerValidationStart, [StringComparison]::Ordinal)
$componentSectionIndices = @(
    $componentResolveStart, $componentCatalogStart, $componentRenderStart,
    $componentRenderEnd, $providerValidationStart, $providerValidationEnd
)
if (@($componentSectionIndices | Where-Object { $_ -lt 0 }).Count -ne 0) {
    throw "Component provider boundary audit could not isolate the canonical route and authorization functions."
}
$componentResolveSource = $componentProviderSource.Substring(
    $componentResolveStart,
    $componentCatalogStart - $componentResolveStart
)
$componentRenderSource = $componentProviderSource.Substring(
    $componentRenderStart,
    $componentRenderEnd - $componentRenderStart
)
$providerValidationSource = $componentProviderSource.Substring(
    $providerValidationStart,
    $providerValidationEnd - $providerValidationStart
)
foreach ($routeSource in @($componentResolveSource, $componentRenderSource)) {
    $verificationIndex = $routeSource.IndexOf('validate_provider_request(', [StringComparison]::Ordinal)
    $decodeIndex = $routeSource.IndexOf('serde_json::from_slice(&body)', [StringComparison]::Ordinal)
    if ($verificationIndex -lt 0 -or $decodeIndex -le $verificationIndex -or
        -not $routeSource.Contains('&body,')) {
        throw "Component provider request authorization and exact-body verification must precede typed request decoding."
    }
}
foreach ($requiredFragment in @(
    'ProviderResourceAssertion::Forbidden',
    'ProviderResourceAssertion::Required',
    'ProviderResourceAssertion::Required => Some(',
    '.ok_or(ComponentModuleError::Forbidden)?',
    'resource_assertion: expected_resource_assertion',
    'canonical_body_digest: sha256_hex(body)'
)) {
    if (-not $providerValidationSource.Contains($requiredFragment)) {
        throw "Component provider authorization validation omits exact boundary fragment '$requiredFragment'."
    }
}
foreach ($requiredFragment in @(
    'ProviderResourceAssertion::Required,',
    'let resource_assertion = ResourceAuthorizationAssertionV2 {',
    'component_grant.payload.resource_assertion.as_ref() != Some(&resource_assertion)',
    '!canonical_nonempty_scope(&request.dashboard_scope_node_ids)',
    'render_authorized_on_same_governing_node(',
    'authority_revision != request.resource_authority_revision'
)) {
    if (-not $componentRenderSource.Contains($requiredFragment)) {
        throw "Component render provider omits exact resource-assertion or same-governing-node enforcement '$requiredFragment'."
    }
}

$coreDatasetProviderSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-api/src/dataset_provider.rs") -Raw
$coreAuthorizationExchangeSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-api/src/module_authorization_exchange.rs") -Raw
if ([regex]::Matches($coreDatasetProviderSource, '(?m)^\s+body: Bytes,\s*$').Count -ne 6 -or
    [regex]::Matches($coreDatasetProviderSource, 'serde_json::from_slice\(&body\)').Count -ne 6 -or
    [regex]::Matches($coreDatasetProviderSource, 'require_json_content_type\(&headers\)').Count -ne 6 -or
    $coreDatasetProviderSource -match 'serde_json::to_vec\(&request\)' -or
    [regex]::Matches($coreAuthorizationExchangeSource, '(?m)^\s+body: Bytes,\s*$').Count -ne 1 -or
    [regex]::Matches($coreAuthorizationExchangeSource, 'serde_json::from_slice\(&body\)').Count -ne 1 -or
    [regex]::Matches($coreAuthorizationExchangeSource, 'require_json_content_type\(&headers\)').Count -ne 1 -or
    $coreAuthorizationExchangeSource -match 'serde_json::to_vec\(&request\)') {
    throw "Core provider and exchange routes must preserve exact inbound body bytes through module service request verification."
}
if ([regex]::Matches(
        $coreDatasetProviderSource,
        '(?s)authorize\(\s*&state,\s*&headers,.*?&body,\s*\)\s*\.await\?;.*?serde_json::from_slice\(&body\)'
    ).Count -ne 5 -or
    $coreDatasetProviderSource -notmatch '(?s)validate_materializing_principal_with_authorization\(.*?body:\s*&body,.*?\)\s*\.await\?;.*?serde_json::from_slice\(&body\)' -or
    $coreAuthorizationExchangeSource -notmatch '(?s)validate_for_principal\(.*?body:\s*&body,.*?\)\s*\.await\?;.*?serde_json::from_slice\(&body\)') {
    throw "Core exchange and Dataset provider authorization must consume exact request bytes before typed JSON decoding."
}
foreach ($requiredFragment in @(
    'resource_assertion_is_authorized(',
    'request.resource_assertion.as_ref()',
    'resource_assertion: request.resource_assertion',
    'downstream_exchange_accepts_only_scope_authorized_resource_assertions'
)) {
    if (-not $coreAuthorizationExchangeSource.Contains($requiredFragment)) {
        throw "Core authorization exchange omits exact resource-assertion propagation proof '$requiredFragment'."
    }
}
$moduleServiceRequestSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-api/src/module_service_requests.rs") -Raw
foreach ($requiredFragment in @(
    'pub(crate) fn require_json_content_type(headers: &HeaderMap)',
    'canonical_body_digest: sha256_hex(expectation.body)',
    'exact_body_routes_retain_json_content_type_enforcement'
)) {
    if (-not $moduleServiceRequestSource.Contains($requiredFragment)) {
        throw "Core module service request verification omits exact-body/content-type contract '$requiredFragment'."
    }
}

$dashboardCompositionSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-dashboard-module/src/composition.rs") -Raw
if ($dashboardCompositionSource -notmatch 'serde_json::from_slice' -or
    $dashboardCompositionSource -notmatch 'ComponentRenderResponse' -or
    $dashboardCompositionSource -notmatch '\.validate_for\(') {
    throw "Dashboard composition must deserialize and validate the canonical Component render response before returning it."
}

$dashboardRenderStart = $dashboardCompositionSource.IndexOf('async fn render_placement(', [StringComparison]::Ordinal)
$dashboardRenderEnd = $dashboardCompositionSource.IndexOf('fn decode_component_render_response(', $dashboardRenderStart, [StringComparison]::Ordinal)
$dashboardProjectionStart = $dashboardCompositionSource.IndexOf('async fn load_placements_with_authorization(', [StringComparison]::Ordinal)
$dashboardProjectionEnd = $dashboardCompositionSource.IndexOf('async fn load_stored_placements(', $dashboardProjectionStart, [StringComparison]::Ordinal)
$dashboardSectionIndices = @(
    $dashboardRenderStart, $dashboardRenderEnd,
    $dashboardProjectionStart, $dashboardProjectionEnd
)
if (@($dashboardSectionIndices | Where-Object { $_ -lt 0 }).Count -ne 0) {
    throw "Dashboard authorization boundary audit could not isolate render and placement projection functions."
}
$dashboardRenderSource = $dashboardCompositionSource.Substring(
    $dashboardRenderStart,
    $dashboardRenderEnd - $dashboardRenderStart
)
$dashboardProjectionSource = $dashboardCompositionSource.Substring(
    $dashboardProjectionStart,
    $dashboardProjectionEnd - $dashboardProjectionStart
)
foreach ($requiredFragment in @(
    'authorized_dashboard_scope(&grant.payload, READ_CAPABILITY, &dashboard_scope)',
    'restrict_component_attempt_for_dashboard_projection(',
    'dashboard_scope_node_ids: authorized_dashboard_scope,',
    'let resource_assertion = component_resource_assertion(',
    'Some(resource_assertion.clone()),',
    'render_authorized_on_same_governing_node(',
    '&request.dashboard_scope_node_ids,',
    '&resource_assertion.governing_organization_ids,'
)) {
    if (-not $dashboardRenderSource.Contains($requiredFragment)) {
        throw "Dashboard mediated render omits exact actor-scope/resource-assertion enforcement '$requiredFragment'."
    }
}
foreach ($requiredFragment in @(
    'let dashboard_capability = if editor {',
    'restrict_component_attempt_for_dashboard_projection('
)) {
    if (-not $dashboardProjectionSource.Contains($requiredFragment)) {
        throw "Dashboard placement projection omits metadata-redaction enforcement '$requiredFragment'."
    }
}
foreach ($requiredFragment in @(
    'fn component_resource_assertion(',
    'fn render_authorized_on_same_governing_node(',
    'fn component_visible_on_dashboard_scope(',
    'component_resource_assertions_require_exact_canonical_identity',
    'mediated_render_requires_one_node_across_dashboard_and_component_authority',
    'mediated_render_sends_only_the_actor_authorized_dashboard_scope_intersection',
    'dashboard_projection_redacts_disjoint_component_metadata',
    'editor_and_viewer_projection_use_their_independent_dashboard_capabilities'
)) {
    if (-not $dashboardCompositionSource.Contains($requiredFragment)) {
        throw "Dashboard composition omits durable authorization boundary proof '$requiredFragment'."
    }
}

$dashboardDependenciesSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-dashboard-module/src/dependencies.rs") -Raw
$proposedReferenceStart = $dashboardDependenciesSource.IndexOf('async fn proposed_reference(', [StringComparison]::Ordinal)
$proposedReferenceEnd = $dashboardDependenciesSource.IndexOf('async fn resolve_finding(', $proposedReferenceStart, [StringComparison]::Ordinal)
$refreshPlacementStart = $dashboardDependenciesSource.IndexOf('async fn refresh_placement(', [StringComparison]::Ordinal)
$refreshPlacementEnd = $dashboardDependenciesSource.IndexOf('fn correlation_id(', $refreshPlacementStart, [StringComparison]::Ordinal)
$dependencySectionIndices = @(
    $proposedReferenceStart, $proposedReferenceEnd,
    $refreshPlacementStart, $refreshPlacementEnd
)
if (@($dependencySectionIndices | Where-Object { $_ -lt 0 }).Count -ne 0) {
    throw "Dashboard dependency boundary audit could not isolate refresh and replacement authorization paths."
}
$proposedReferenceSource = $dashboardDependenciesSource.Substring(
    $proposedReferenceStart,
    $proposedReferenceEnd - $proposedReferenceStart
)
$refreshPlacementSource = $dashboardDependenciesSource.Substring(
    $refreshPlacementStart,
    $refreshPlacementEnd - $refreshPlacementStart
)
foreach ($requiredFragment in @(
    'DependencyAction::Upgrade',
    'DependencyAction::Replace',
    'authorized_dashboard_scope(grant, MANAGE_CAPABILITY, &dashboard_scope)',
    'restrict_component_attempt_for_dashboard_projection(',
    'component_scope_within_dashboard_scope(',
    '&metadata.scope_node_ids,',
    '&managed_dashboard_scope,'
)) {
    if (-not $proposedReferenceSource.Contains($requiredFragment)) {
        throw "Dashboard Upgrade/Replace authorization omits exact fully-contained scope enforcement '$requiredFragment'."
    }
}
foreach ($requiredFragment in @(
    'canonical_nonempty_scope(component_scope_node_ids)',
    'canonical_nonempty_scope(dashboard_scope_node_ids)',
    '.all(|node_id| dashboard_scope_node_ids.binary_search(node_id).is_ok())',
    'replacement_scope_must_be_nonempty_canonical_and_contained_by_dashboard_scope'
)) {
    if (-not $dashboardDependenciesSource.Contains($requiredFragment)) {
        throw "Dashboard replacement-scope boundary omits durable containment proof '$requiredFragment'."
    }
}
$refreshRestrictionIndex = $refreshPlacementSource.IndexOf('restrict_component_attempt_for_dashboard_projection(', [StringComparison]::Ordinal)
$refreshProjectionIndex = $refreshPlacementSource.IndexOf('project_component_resolution_for_visibility(', [StringComparison]::Ordinal)
$observationInsertIndex = $refreshPlacementSource.IndexOf('INSERT INTO dashboard_dependency_observations', [StringComparison]::Ordinal)
$findingInsertIndex = $refreshPlacementSource.IndexOf('UPSERT_DEPENDENCY_FINDING_SQL', [StringComparison]::Ordinal)
if ($refreshRestrictionIndex -lt 0 -or
    -not $refreshPlacementSource.Contains('inbound_dashboard_grant,') -or
    -not $refreshPlacementSource.Contains('MANAGE_CAPABILITY,') -or
    -not $refreshPlacementSource.Contains('dashboard_scope_node_ids,') -or
    $refreshProjectionIndex -le $refreshRestrictionIndex -or
    $observationInsertIndex -le $refreshProjectionIndex -or
    $findingInsertIndex -le $observationInsertIndex) {
    throw "Dashboard dependency refresh must apply MANAGE-capability joint-scope redaction before projection and before persisting observations or findings."
}
$componentIntegrationSource = Get-Content -LiteralPath `
    (Join-Path $repoRoot "crates/tessara-component-module/tests/product_integration.rs") -Raw
foreach ($requiredFragment in @(
    "altered_body.body.push(b' ');",
    'changing an otherwise equivalent JSON body byte without re-signing must fail closed',
    'let exact_resource_assertion = ResourceAuthorizationAssertionV2 {',
    'let mut stale_resource_assertion = exact_resource_assertion.clone();',
    'let mut wrong_resource_assertion = exact_resource_assertion;'
)) {
    if (-not $componentIntegrationSource.Contains($requiredFragment)) {
        throw "Component integration coverage omits exact request/resource-assertion negative proof '$requiredFragment'."
    }
}

$rootReferences = & rg -n "tessara-dashboard-ui|tessara_dashboard_ui" `
    (Join-Path $repoRoot "crates/tessara-web") `
    (Join-Path $repoRoot "crates/tessara-api")
if ($LASTEXITCODE -eq 0 -or $rootReferences) {
    throw "Core/root web still consumes Dashboard UI source:`n$rootReferences"
}
if ($LASTEXITCODE -ne 1) {
    throw "Root Dashboard UI ownership audit failed to execute."
}

$composeOverride = Get-Content -LiteralPath `
    (Join-Path $repoRoot "deploy/sprint-6e/compose.override.yaml") -Raw
if ($composeOverride -notmatch "traefik\.http\.routers\.tessara-core\.entrypoints:\s*web") {
    throw "Sprint 6E must isolate the public Core router to the web entrypoint."
}
foreach ($slot in @("baseline", "candidate")) {
    $route = Get-Content -LiteralPath `
        (Join-Path $repoRoot "deploy/sprint-6e/dashboard-route.$slot.yaml") -Raw
    if ($route -notmatch "entryPoints:\s*\[module\]") {
        throw "Sprint 6E Dashboard slot '$slot' is not isolated to the module entrypoint."
    }
}
$slotSwitch = Get-Content -LiteralPath `
    (Join-Path $repoRoot "scripts/set-sprint-6e-dashboard-slot.ps1") -Raw
if ($slotSwitch -notmatch "docker kill --signal=HUP" `
    -or $slotSwitch -notmatch "gateway_restart_count") {
    throw "Sprint 6E slot switching must reload Traefik without restarting it."
}

Write-Host "Sprint 6E Dashboard source and package boundaries passed."

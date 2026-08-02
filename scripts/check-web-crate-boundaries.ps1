[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

function Invoke-CargoMetadata {
    param(
        [Parameter(Mandatory)]
        [string]$Platform
    )

    $output = & cargo metadata --locked --format-version 1 --filter-platform $Platform 2>&1
    if ($LASTEXITCODE -ne 0) {
        $output | ForEach-Object { $_.ToString() } | Write-Error
        throw "cargo metadata failed for $Platform"
    }

    $output -join "`n" | ConvertFrom-Json
}

function Assert-PackageTreeContainsNoFrameworks {
    param(
        [Parameter(Mandatory)]
        [string]$Platform,

        [Parameter(Mandatory)]
        [string]$PackageName,

        [Parameter(Mandatory)]
        [string[]]$ForbiddenPackages,

        [Parameter(Mandatory)]
        [string]$Description
    )

    # Package-scoped Cargo trees avoid attributing features enabled by unrelated
    # workspace members to a shared transitive dependency of this package.
    $output = & cargo tree --locked -p $PackageName --target $Platform --edges normal,build,dev --prefix none --format "{p}" 2>&1
    if ($LASTEXITCODE -ne 0) {
        $output | ForEach-Object { $_.ToString() } | Write-Error
        throw "cargo tree failed for $PackageName on $Platform"
    }

    $resolvedPackageNames = @(
        foreach ($line in @($output)) {
            if ($line.ToString() -match '^(?<name>\S+)\s+v\d') {
                $Matches.name
            }
        }
    )
    $violations = @(
        $resolvedPackageNames |
            Where-Object { $_ -in $ForbiddenPackages } |
            Sort-Object -Unique
    )

    if ($violations.Count -gt 0) {
        throw "$Description`nRejected resolved package(s): $($violations -join ', ')"
    }
}

function Get-DependencyKindLabel {
    param($Dependency)

    $kinds = @()
    foreach ($kind in @($Dependency.dep_kinds)) {
        $kindName = if ($null -eq $kind.kind) { "normal" } else { [string]$kind.kind }
        if ($kind.target) {
            $kindName = "$kindName target=$($kind.target)"
        }
        $kinds += $kindName
    }

    if ($kinds.Count -eq 0) {
        return "normal"
    }

    ($kinds | Sort-Object -Unique) -join ","
}

function New-MetadataGraph {
    param(
        [Parameter(Mandatory)]
        $Metadata
    )

    $packageNamesById = @{}
    foreach ($package in $Metadata.packages) {
        $packageNamesById[$package.id] = $package.name
    }

    $workspacePackageNames = @{}
    foreach ($id in @($Metadata.workspace_members)) {
        if ($packageNamesById.ContainsKey($id)) {
            $workspacePackageNames[$packageNamesById[$id]] = $true
        }
    }

    $edgesById = @{}
    foreach ($node in @($Metadata.resolve.nodes)) {
        $edges = @()
        foreach ($dependency in @($node.deps)) {
            if (-not $packageNamesById.ContainsKey($dependency.pkg)) {
                continue
            }

            $edges += [pscustomobject]@{
                fromId = $node.id
                fromName = $packageNamesById[$node.id]
                toId = $dependency.pkg
                toName = $packageNamesById[$dependency.pkg]
                alias = $dependency.name
                kind = Get-DependencyKindLabel -Dependency $dependency
            }
        }
        $edgesById[$node.id] = $edges
    }

    [pscustomobject]@{
        packageNamesById = $packageNamesById
        workspacePackageNames = $workspacePackageNames
        edgesById = $edgesById
    }
}

function Find-PackageId {
    param(
        [Parameter(Mandatory)]
        $Graph,

        [Parameter(Mandatory)]
        [string]$PackageName
    )

    foreach ($entry in $Graph.packageNamesById.GetEnumerator()) {
        if ($entry.Value -eq $PackageName) {
            return $entry.Key
        }
    }

    throw "Package not present in metadata graph: $PackageName"
}

function Format-Path {
    param(
        [Parameter(Mandatory)]
        [array]$Path
    )

    ($Path | ForEach-Object {
        if ($_.alias -and $_.alias -ne $_.toName) {
            "$($_.fromName) --[$($_.kind); alias=$($_.alias)]--> $($_.toName)"
        } else {
            "$($_.fromName) --[$($_.kind)]--> $($_.toName)"
        }
    }) -join "`n"
}

function Assert-NoDependencyPath {
    param(
        [Parameter(Mandatory)]
        $Graph,

        [Parameter(Mandatory)]
        [string]$StartPackage,

        [Parameter(Mandatory)]
        [scriptblock]$IsForbiddenPackage,

        [Parameter(Mandatory)]
        [string]$Description
    )

    $startId = Find-PackageId -Graph $Graph -PackageName $StartPackage
    $visited = @{}
    $queue = New-Object System.Collections.Generic.Queue[object]
    $queue.Enqueue([pscustomobject]@{ id = $startId; path = @() })
    $visited[$startId] = $true

    while ($queue.Count -gt 0) {
        $current = $queue.Dequeue()
        foreach ($edge in @($Graph.edgesById[$current.id])) {
            $path = @($current.path) + $edge
            if (& $IsForbiddenPackage $edge.toName $path) {
                throw "$Description`nRejected dependency path:`n$(Format-Path -Path $path)"
            }

            if (-not $visited.ContainsKey($edge.toId)) {
                $visited[$edge.toId] = $true
                $queue.Enqueue([pscustomobject]@{ id = $edge.toId; path = $path })
            }
        }
    }
}

function Assert-SourceDoesNotMatch {
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [Parameter(Mandatory)]
        [string]$Pattern,

        [Parameter(Mandatory)]
        [string]$Description
    )

    $matches = rg --line-number --color never $Pattern $Path 2>$null
    if ($LASTEXITCODE -eq 0) {
        throw "$Description`n$($matches -join "`n")"
    }
    if ($LASTEXITCODE -gt 1) {
        throw "Source audit failed for $Path"
    }
}

function Write-ReviewAidMatches {
    param(
        [Parameter(Mandatory)]
        [string]$Path,

        [Parameter(Mandatory)]
        [string]$Pattern,

        [Parameter(Mandatory)]
        [string]$Description
    )

    $matches = rg --line-number --color never $Pattern $Path 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host $Description -ForegroundColor Yellow
        $matches | ForEach-Object { Write-Host $_ -ForegroundColor DarkYellow }
    } elseif ($LASTEXITCODE -gt 1) {
        throw "Review-aid source audit failed for $Path"
    }
}

$platforms = @("x86_64-pc-windows-msvc", "wasm32-unknown-unknown")
$domainCrates = @(
    "tessara-analytics",
    "tessara-auth",
    "tessara-core",
    "tessara-dashboards",
    "tessara-datasets",
    "tessara-forms",
    "tessara-hierarchy",
    "tessara-submissions"
)
$frameworkNeutralContractCrates = @("tessara-module-contract")
$domainTransitiveForbidden = @("leptos", "axum", "sqlx", "gloo-net")
$domainDirectForbidden = @("web-sys", "js-sys", "wasm-bindgen")
$domainForbidden = $domainTransitiveForbidden + $domainDirectForbidden

Push-Location $repoRoot
try {
    foreach ($platform in $platforms) {
        Write-Host "Checking Cargo dependency boundaries for $platform" -ForegroundColor Cyan
        $metadata = Invoke-CargoMetadata -Platform $platform
        $graph = New-MetadataGraph -Metadata $metadata

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-datasets" -Description "tessara-web-datasets must not depend on root/API/sibling web feature crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-datasets", "tessara-web-data-ops", "tessara-web-http", "tessara-module-ui"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-forms" -Description "tessara-web-forms must not depend on root/API/sibling web feature crates or router/meta crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "leptos_router", "leptos_meta") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-forms", "tessara-web-http", "tessara-module-ui"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-workflows" -Description "tessara-web-workflows must not depend on root/API/sibling web feature crates or router/meta crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "leptos_router", "leptos_meta") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-workflows", "tessara-web-http", "tessara-module-ui"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-responses" -Description "tessara-web-responses must not depend on root/API/sibling web feature crates or router/meta crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "leptos_router", "leptos_meta") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-responses", "tessara-web-http", "tessara-module-ui"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-organization" -Description "tessara-web-organization must not depend on root/API/sibling web feature crates or router/meta crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "leptos_router", "leptos_meta") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-organization", "tessara-web-http", "tessara-module-ui"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-module-ui" -Description "tessara-module-ui must not depend on root/API/web feature crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api") -or
                ($name -like "tessara-web-*" -and $name -ne "tessara-module-ui")
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-http" -Description "tessara-web-http must remain policy-neutral and independent of root/API/web feature crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "leptos", "leptos_router", "leptos_meta") -or
                ($name -like "tessara-web-*" -and $name -ne "tessara-web-http")
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-dashboard-ui" -Description "tessara-dashboard-ui must not depend on root/API/router/meta, Components/data-ops, or unapproved web feature crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "leptos_router", "leptos_meta", "tessara-web-components", "tessara-web-data-ops") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-http", "tessara-web-component-viewer"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-component-viewer" -Description "tessara-web-component-viewer must remain a route-free presentation leaf with no root/API/feature dependencies." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "leptos_router", "leptos_meta") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-component-viewer", "tessara-web-http", "tessara-module-ui"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-components" -Description "tessara-web-components may use data-ops, shared UI, and the viewer leaf, but not root/API/Dashboard or other feature crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api", "tessara-dashboard-ui") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-components", "tessara-web-data-ops", "tessara-web-http", "tessara-module-ui", "tessara-web-component-viewer"))
        }

        foreach ($crate in $domainCrates) {
            if (-not $graph.workspacePackageNames.ContainsKey($crate)) {
                continue
            }
            Assert-NoDependencyPath -Graph $graph -StartPackage $crate -Description "$crate must not depend on web/server transport or UI frameworks." -IsForbiddenPackage {
                param($name, $path)
                $name -in $domainTransitiveForbidden -or
                    ($name -in $domainDirectForbidden -and $path.Count -eq 1)
            }
        }

        foreach ($crate in $frameworkNeutralContractCrates) {
            Assert-NoDependencyPath -Graph $graph -StartPackage $crate -Description "$crate must remain a pure, framework-neutral contract crate." -IsForbiddenPackage {
                param($name, $path)
                $name -in $domainTransitiveForbidden -or
                    ($name -in $domainDirectForbidden -and $path.Count -eq 1)
            }
        }
    }

    Assert-SourceDoesNotMatch -Path "crates\tessara-web-datasets\src" -Pattern "crate::(features|ui|utils|routes|state|types::route_params)|AppShell|require_authenticated_route|leptos_router|leptos_meta" -Description "tessara-web-datasets must not import root app, route, shell, auth, or router/meta concepts."
    Assert-SourceDoesNotMatch -Path "crates\tessara-web-forms\src" -Pattern "AppShell|require_route_params|FormRouteParams|crate::routes|leptos_router|leptos_meta|features::organization|features::workflows|features::responses|features::datasets|features::administration|features::shared|crate::features::forms|pub\(in crate::features::forms\)" -Description "tessara-web-forms must not import root route, shell, router/meta, old forms namespace, or sibling web feature concepts."
    Assert-SourceDoesNotMatch -Path "crates\tessara-web-workflows\src" -Pattern "AppShell|require_route_params|WorkflowRouteParams|crate::routes|leptos_router|leptos_meta|features::forms|features::organization|features::responses|features::datasets|features::administration|features::operations|features::shared|crate::features::workflows|pub\(in crate::features::workflows\)" -Description "tessara-web-workflows must not import root route, shell, router/meta, old workflows namespace, or sibling web feature concepts."
    Assert-SourceDoesNotMatch -Path "crates\tessara-web-responses\src" -Pattern "AppShell|require_route_params|SubmissionRouteParams|crate::routes|leptos_router|leptos_meta|features::forms|features::workflows|features::organization|features::administration|features::shared|crate::features::responses|pub\(in crate::features::responses\)" -Description "tessara-web-responses must not import root route, shell, router/meta, old responses namespace, or sibling web feature concepts."
    Assert-SourceDoesNotMatch -Path "crates\tessara-web-organization\src" -Pattern "AppShell|require_route_params|NodeRouteParams|crate::routes|leptos_router|leptos_meta|features::forms|features::workflows|features::responses|features::datasets|features::administration|features::shared|crate::features::organization|pub\(in crate::features::organization\)" -Description "tessara-web-organization must not import root route, shell, router/meta, old organization namespace, or sibling web feature concepts."
    Assert-SourceDoesNotMatch -Path "crates\tessara-dashboard-ui\src" -Pattern "AppShell|ShellSessionBootstrap|ApplicationBootstrap|ApplicationRenderContext|crate::(app|features|routes|state|ui)|types::route_params|require_route_params|leptos_router|leptos_meta|tessara_api|tessara_web_components|tessara_web_data_ops|features::(components|data_ops)|use_(location|navigate|params|query|resolved_path)" -Description "tessara-dashboard-ui must not import root route/shell/state, router/meta, API, Components, or data-ops authoring concepts."
    Assert-SourceDoesNotMatch -Path "crates\tessara-web-component-viewer\src" -Pattern "AppShell|ShellSessionBootstrap|ApplicationBootstrap|ApplicationRenderContext|DashboardPlacement|DashboardRoute|crate::(app|document|features|routes|state|ui|editor|versions|publishing)|types::route_params|require_route_params|use_(location|navigate|params|query|resolved_path)|redirect_to_login|set_href|leptos_router|leptos_meta|tessara_api|tessara_dashboards|tessara_web_components|tessara_web_dashboards|tessara_web_data_ops|features::(dashboards|components|data_ops)" -Description "tessara-web-component-viewer must not import route/shell/login, Dashboard, Components authoring/version-management, or data-ops authoring concepts."

    Write-ReviewAidMatches -Path "crates\tessara-module-ui\src" -Pattern "datasets|forms|workflows|responses|organization|administration|AppShell|ShellSession|require_authenticated_route" -Description "Review-aid matches in tessara-module-ui source:"

    Write-Host "Web crate boundary checks passed." -ForegroundColor Green
    # `rg` uses exit code 1 for an expected no-match result. Do not leak that
    # internal probe state to a parent validation script after every assertion
    # has passed.
    $global:LASTEXITCODE = 0
} finally {
    Pop-Location
}

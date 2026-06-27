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

    $output = & cargo metadata --format-version 1 --filter-platform $Platform 2>&1
    if ($LASTEXITCODE -ne 0) {
        $output | ForEach-Object { $_.ToString() } | Write-Error
        throw "cargo metadata failed for $Platform"
    }

    $output -join "`n" | ConvertFrom-Json
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
            if (& $IsForbiddenPackage $edge.toName) {
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
$domainForbidden = @("leptos", "axum", "sqlx", "gloo-net", "web-sys", "js-sys", "wasm-bindgen")

Push-Location $repoRoot
try {
    foreach ($platform in $platforms) {
        Write-Host "Checking Cargo dependency boundaries for $platform" -ForegroundColor Cyan
        $metadata = Invoke-CargoMetadata -Platform $platform
        $graph = New-MetadataGraph -Metadata $metadata

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-datasets" -Description "tessara-web-datasets must not depend on root/API/sibling web feature crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api") -or
                ($name -like "tessara-web-*" -and $name -notin @("tessara-web-datasets", "tessara-web-ui"))
        }

        Assert-NoDependencyPath -Graph $graph -StartPackage "tessara-web-ui" -Description "tessara-web-ui must not depend on root/API/web feature crates." -IsForbiddenPackage {
            param($name)
            $name -in @("tessara-web", "tessara-api") -or
                ($name -like "tessara-web-*" -and $name -ne "tessara-web-ui")
        }

        foreach ($crate in $domainCrates) {
            if (-not $graph.workspacePackageNames.ContainsKey($crate)) {
                continue
            }
            Assert-NoDependencyPath -Graph $graph -StartPackage $crate -Description "$crate must not depend on web/server transport or UI frameworks." -IsForbiddenPackage {
                param($name)
                $name -in $domainForbidden
            }
        }
    }

    Assert-SourceDoesNotMatch -Path "crates\tessara-web-datasets\src" -Pattern "crate::(features|ui|utils|routes|state|types::route_params)|AppShell|require_authenticated_route|leptos_router|leptos_meta" -Description "tessara-web-datasets must not import root app, route, shell, auth, or router/meta concepts."

    Write-ReviewAidMatches -Path "crates\tessara-web-ui\src" -Pattern "datasets|forms|workflows|responses|organization|administration|AppShell|ShellSession|require_authenticated_route" -Description "Review-aid matches in tessara-web-ui source:"

    Write-Host "Web crate boundary checks passed." -ForegroundColor Green
} finally {
    Pop-Location
}

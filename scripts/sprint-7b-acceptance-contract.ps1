Set-StrictMode -Version Latest

. (Join-Path $PSScriptRoot "sprint-7a-acceptance-contract.ps1")

$script:Sprint7BRepositoryRoot = Split-Path -Parent $PSScriptRoot
$script:Sprint7BFixture = [ordered]@{
    component_id = "01980000-0002-7000-8000-000000000004"
    component_version_id = "01980000-0001-7000-8000-000000000001"
    dashboard_id = "01980000-0003-7000-8000-000000000001"
    placement_id = "01980000-0003-7000-8000-000000000002"
}

function Test-Sprint7BAcceptanceContract {
    foreach ($entry in $script:Sprint7BFixture.GetEnumerator()) {
        try {
            $null = [guid]::ParseExact([string]$entry.Value, "D")
        } catch {
            throw "Sprint 7B fixture '$($entry.Key)' is not a UUID."
        }
    }

    $manualRoot = Join-Path $script:Sprint7BRepositoryRoot "docs/sprints/sprint-7b-uat"
    foreach ($index in 1..9) {
        $name = "uat-7b-{0:D2}.md" -f $index
        if (-not (Test-Path -LiteralPath (Join-Path $manualRoot $name) -PathType Leaf)) {
            throw "Manual Sprint 7B UAT script is missing: $name"
        }
    }

    foreach ($reference in @(
        "component-versions-desktop-dark.png",
        "component-versions-mobile-light.png",
        "dashboard-editor-desktop-dark.png",
        "dashboard-editor-mobile-light.png"
    )) {
        $path = Join-Path $script:Sprint7BRepositoryRoot "docs/sprints/sprint-7b-ui-review/reference/$reference"
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Approved Sprint 7B visual reference is missing: $reference"
        }
    }
}

function Assert-Sprint7BJsonProperty {
    param(
        [Parameter(Mandatory)][object]$Object,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Context
    )
    if ($null -eq $Object.PSObject.Properties[$Name]) {
        throw "Sprint 7B acceptance failed: $Context omitted '$Name'."
    }
}

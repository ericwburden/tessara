[CmdletBinding()]
param(
    [string]$ApplyResponsePath,
    [string]$OutputPath = "target/sprint-8b-uat/fixture-receipt.json",
    [string]$ComposeProject,
    [string]$ReferenceFixturePath = "deploy/sprint-8b/fixtures/reference-fixture-contract.json",
    [string]$BlueprintPath = "deploy/sprint-8b/blueprints/reference.json",
    [string]$CoreBaseUrl,
    [string]$OwnerActionEmail = "admin@tessara.local",
    [string]$OwnerActionPassword = "tessara-dev-admin",
    [string]$OwnerActionPublicKey,
    [switch]$SelfTest
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$requestedSelfTest = [bool]$SelfTest
. (Join-Path $PSScriptRoot "sprint-8b-harness-isolation.ps1")
$SelfTest = $requestedSelfTest
$responseOwnerActionPath = "/api/admin/responses/owner-actions"
$responseOwnerActionMediaType =
    "application/vnd.tessara.responses.owner-action+json;version=1"
$defaultCoreAuthorizationPublicKey = "C1E62bSSQBXKCQLtB5BE06xdvsIwbwaUjBDajrbjny0"

function Read-Sprint8BFixtureJson {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label
    )

    $resolved = Resolve-Sprint8BRepositoryPath -Path $Path
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "$Label is missing: $resolved"
    }
    try {
        Get-Content -LiteralPath $resolved -Raw | ConvertFrom-Json -Depth 100
    } catch {
        throw "$Label is not valid JSON: $($_.Exception.Message)"
    }
}

function Get-Sprint8BOwnerReceipt {
    param(
        [Parameter(Mandatory)][object[]]$Receipts,
        [Parameter(Mandatory)][string]$Owner
    )

    $matches = @($Receipts | Where-Object { [string]$_.owner -ceq $Owner })
    if ($matches.Count -ne 1) {
        throw "Expected exactly one owner receipt for '$Owner'; found $($matches.Count)."
    }
    $matches[0]
}

function Get-Sprint8BReceiptResourceValue {
    param(
        [Parameter(Mandatory)]$ResourceIds,
        [Parameter(Mandatory)][string]$ResourceKey
    )

    $property = $ResourceIds.PSObject.Properties[$ResourceKey]
    if ($null -eq $property) {
        throw "Owner receipt omits logical resource key '$ResourceKey'."
    }
    [string]$property.Value
}

function Assert-Sprint8BCanonicalUuid {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -cnotmatch '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
        throw "$Label is not a canonical UUID."
    }
    $Value
}

$script:Sprint8BJsonStringOptions = [Text.Json.JsonSerializerOptions]::new()
$script:Sprint8BJsonStringOptions.Encoder =
    [Text.Encodings.Web.JavaScriptEncoder]::UnsafeRelaxedJsonEscaping

function ConvertTo-Sprint8BJcsJson {
    param([AllowNull()]$Value)

    if ($null -eq $Value) { return "null" }
    if ($Value -is [string] -or $Value -is [char]) {
        return [Text.Json.JsonSerializer]::Serialize(
            [string]$Value,
            $script:Sprint8BJsonStringOptions
        )
    }
    if ($Value -is [bool]) { return $(if ($Value) { "true" } else { "false" }) }
    if ($Value -is [byte] -or $Value -is [sbyte] -or
        $Value -is [int16] -or $Value -is [uint16] -or
        $Value -is [int32] -or $Value -is [uint32] -or
        $Value -is [int64] -or $Value -is [uint64]) {
        return [Convert]::ToString($Value, [Globalization.CultureInfo]::InvariantCulture)
    }
    if ($Value -is [single] -or $Value -is [double] -or $Value -is [decimal]) {
        $number = [Convert]::ToString($Value, [Globalization.CultureInfo]::InvariantCulture)
        if ($number -match 'NaN|Infinity') { throw "JCS cannot encode a non-finite number." }
        return $number
    }
    if ($Value -is [Collections.IDictionary]) {
        $names = [string[]]@($Value.Keys | ForEach-Object { [string]$_ })
        [Array]::Sort($names, [StringComparer]::Ordinal)
        return "{" + (@($names | ForEach-Object {
            "$(ConvertTo-Sprint8BJcsJson $_):$(ConvertTo-Sprint8BJcsJson $Value[$_])"
        }) -join ",") + "}"
    }
    if ($Value -is [pscustomobject]) {
        $names = [string[]]@($Value.PSObject.Properties.Name)
        [Array]::Sort($names, [StringComparer]::Ordinal)
        return "{" + (@($names | ForEach-Object {
            "$(ConvertTo-Sprint8BJcsJson $_):$(ConvertTo-Sprint8BJcsJson $Value.$_)"
        }) -join ",") + "}"
    }
    if ($Value -is [Collections.IEnumerable]) {
        return "[" + (@($Value | ForEach-Object { ConvertTo-Sprint8BJcsJson $_ }) -join ",") + "]"
    }
    throw "JCS cannot encode value type '$($Value.GetType().FullName)'."
}

function Get-Sprint8BJcsDigest {
    param([Parameter(Mandatory)]$Value)
    "sha256:$(Get-Sprint7ASha256 -Text (ConvertTo-Sprint8BJcsJson $Value))"
}

function Assert-Sprint8BExactProperties {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string[]]$Expected,
        [Parameter(Mandatory)][string]$Label
    )
    $actual = [string[]]@($Value.PSObject.Properties.Name)
    [Array]::Sort($actual, [StringComparer]::Ordinal)
    $expectedSorted = [string[]]@($Expected)
    [Array]::Sort($expectedSorted, [StringComparer]::Ordinal)
    if (($actual -join "`n") -cne ($expectedSorted -join "`n")) {
        throw "$Label property set is not exact."
    }
}

function Assert-Sprint8BFormSchema {
    param(
        [Parameter(Mandatory)]$Schema,
        [Parameter(Mandatory)][string]$SchemaJson,
        [Parameter(Mandatory)]$FormDefinition,
        [Parameter(Mandatory)][string]$FormId,
        [Parameter(Mandatory)][string]$FormVersionId,
        [Parameter(Mandatory)]$Scopes
    )

    Assert-Sprint8BExactProperties -Value $Schema -Label "Core FormVersion schema" -Expected @(
        "schema_version", "form_id", "form_version_id", "form_name", "form_slug",
        "version_label", "version_major", "source_scope_node_ids", "source_scope_revision",
        "source_scope_digest", "content_revision", "content_digest", "sections", "fields"
    )
    $expectedMajor = 0
    if (-not [int]::TryParse(([string]$FormDefinition.version_label).Split('.')[0], [ref]$expectedMajor)) {
        throw "Reference Form '$($FormDefinition.resource_key)' has no canonical major version."
    }
    if ([int]$Schema.schema_version -ne 1 -or
        [string]$Schema.form_id -cne $FormId -or
        [string]$Schema.form_version_id -cne $FormVersionId -or
        [string]$Schema.form_name -cne ([string]$FormDefinition.name).Trim() -or
        [string]$Schema.form_slug -cne [string]$FormDefinition.slug -or
        [string]$Schema.version_label -cne [string]$FormDefinition.version_label -or
        [int]$Schema.version_major -ne $expectedMajor) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' has substituted Form identity content."
    }

    $expectedScope = [string[]]@($FormDefinition.scope_node_keys | ForEach-Object {
        [string]$Scopes.PSObject.Properties[[string]$_].Value
    })
    [Array]::Sort($expectedScope, [StringComparer]::Ordinal)
    $actualScope = [string[]]@($Schema.source_scope_node_ids | ForEach-Object {
        Assert-Sprint8BCanonicalUuid -Value ([string]$_) -Label "Form source scope"
    })
    if (($actualScope -join "`n") -cne ($expectedScope -join "`n")) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' source scope is not exact and ordered."
    }
    $scopeDigest = Get-Sprint8BJcsDigest -Value @($Schema.source_scope_node_ids)
    if ([string]$Schema.source_scope_digest -cne $scopeDigest -or
        [string]$Schema.source_scope_revision -cne $scopeDigest) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' source-scope digest does not recompute."
    }

    if (@($Schema.sections).Count -ne 1) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' must contain its exact Response section."
    }
    $section = @($Schema.sections)[0]
    Assert-Sprint8BExactProperties -Value $section -Label "Form section" `
        -Expected @("section_id", "key", "label", "position")
    $sectionId = Assert-Sprint8BCanonicalUuid -Value ([string]$section.section_id) `
        -Label "Form section"
    if ([string]$section.key -cne $sectionId -or [string]$section.label -cne "Response" -or
        [int]$section.position -ne 0) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' section content is not exact."
    }

    $expectedFields = @($FormDefinition.fields | Sort-Object `
        @{ Expression = { [int]$_.position } },
        @{ Expression = { [int]$_.grid_row } },
        @{ Expression = { [int]$_.grid_column } },
        @{ Expression = { [string]$_.label } },
        @{ Expression = { [string]$_.key } })
    $actualFields = @($Schema.fields)
    if ($actualFields.Count -ne $expectedFields.Count -or $actualFields.Count -eq 0) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' field set is not exact."
    }
    $fieldIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    for ($index = 0; $index -lt $expectedFields.Count; $index++) {
        $expected = $expectedFields[$index]
        $actual = $actualFields[$index]
        Assert-Sprint8BExactProperties -Value $actual -Label "Form field" -Expected @(
            "field_id", "key", "label", "field_type", "required", "options", "section_id",
            "position", "grid_row", "grid_column"
        )
        $fieldId = Assert-Sprint8BCanonicalUuid -Value ([string]$actual.field_id) -Label "Form field"
        if (-not $fieldIds.Add($fieldId) -or
            [string]$actual.key -cne [string]$expected.key -or
            [string]$actual.label -cne ([string]$expected.label).Trim() -or
            [string]$actual.field_type -cne [string]$expected.field_type -or
            [bool]$actual.required -ne [bool]$expected.required -or
            @($actual.options).Count -ne 0 -or
            [string]$actual.section_id -cne $sectionId -or
            [int]$actual.position -ne [int]$expected.position -or
            [int]$actual.grid_row -ne [int]$expected.grid_row -or
            [int]$actual.grid_column -ne [int]$expected.grid_column) {
            throw "Core FormVersion schema '$($FormDefinition.resource_key)' field/type/options/layout is not exact."
        }
    }

    $content = [ordered]@{
        schema_version = [int]$Schema.schema_version
        form_id = [string]$Schema.form_id
        form_version_id = [string]$Schema.form_version_id
        form_name = [string]$Schema.form_name
        form_slug = [string]$Schema.form_slug
        version_label = $Schema.version_label
        version_major = $Schema.version_major
        source_scope_node_ids = @($Schema.source_scope_node_ids)
        source_scope_revision = [string]$Schema.source_scope_revision
        source_scope_digest = [string]$Schema.source_scope_digest
        sections = @($Schema.sections)
        fields = @($Schema.fields)
    }
    $contentDigest = Get-Sprint8BJcsDigest -Value $content
    if ([string]$Schema.content_digest -cne $contentDigest -or
        [string]$Schema.content_revision -cne $contentDigest) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' content digest does not recompute."
    }
    if ($SchemaJson -cne ($Schema | ConvertTo-Json -Depth 100 -Compress)) {
        throw "Core FormVersion schema '$($FormDefinition.resource_key)' is not its exact compact canonical receipt encoding."
    }
    [pscustomobject][ordered]@{
        schema = $Schema
        schema_json = $SchemaJson
        source_scope_digest = $scopeDigest
        content_digest = $contentDigest
        exact_field_type_option_layout = "passed"
    }
}

function ConvertFrom-Sprint8BHttpContent {
    param([AllowEmptyString()][AllowNull()]$Content)

    if ($Content -is [byte[]]) {
        return [System.Text.Encoding]::UTF8.GetString([byte[]]$Content)
    }
    [string]$Content
}

function Invoke-Sprint8BFixtureHttpRequest {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Body,
        [string]$ContentType = "application/json",
        [hashtable]$Headers = @{}
    )

    $response = Invoke-WebRequest -Uri "$($BaseUrl.TrimEnd('/'))$Path" -Method POST `
        -Headers $Headers -ContentType $ContentType -Body $Body -UseBasicParsing `
        -SkipHttpErrorCheck -TimeoutSec 30
    # PowerShell exposes response bodies with unregistered vendor media types as
    # bytes on some hosts. Preserve the exact UTF-8 wire body instead of
    # stringifying the byte array as "System.Byte[]".
    $content = ConvertFrom-Sprint8BHttpContent -Content $response.Content
    if ([int]$response.StatusCode -ne 200) {
        $errorCode = ""
        try {
            $errorDocument = $content | ConvertFrom-Json -Depth 30 -DateKind String
            $errorCode = if ($null -ne $errorDocument.error) {
                [string]$errorDocument.error.code
            } else {
                [string]$errorDocument.code
            }
        } catch {}
        throw "Core fixture request 'POST $Path' returned HTTP $([int]$response.StatusCode) ($errorCode)."
    }
    try { $document = $content | ConvertFrom-Json -Depth 100 -DateKind String } catch {
        throw "Core fixture request 'POST $Path' did not return JSON."
    }
    [pscustomobject][ordered]@{
        status = [int]$response.StatusCode
        content_type = [string]$response.Headers.'Content-Type'
        body = $content
        body_sha256 = Get-Sprint7ASha256 -Text $content
        document = $document
    }
}

function Get-Sprint8BFixtureOwnerToken {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Email,
        [Parameter(Mandatory)][string]$Password
    )

    $body = [ordered]@{ email = $Email; password = $Password } |
        ConvertTo-Json -Depth 10 -Compress
    $response = Invoke-Sprint8BFixtureHttpRequest -BaseUrl $BaseUrl `
        -Path "/api/auth/login" -Body $body
    if ([string]::IsNullOrWhiteSpace([string]$response.document.token)) {
        throw "Core fixture login omitted its bearer token."
    }
    [string]$response.document.token
}

function Test-Sprint8BResponseOwnerReceiptSignature {
    param(
        [Parameter(Mandatory)]$Envelope,
        [Parameter(Mandatory)][string]$PublicKey
    )

    $verifier = Join-Path $PSScriptRoot "verify-sprint-8b-response-owner-receipt.mjs"
    if (-not (Test-Path -LiteralPath $verifier -PathType Leaf)) {
        throw "Response owner receipt verifier is missing: $verifier"
    }
    $envelopeJson = $Envelope | ConvertTo-Json -Depth 100 -Compress
    $output = @($envelopeJson | & node $verifier --public-key $PublicKey 2>&1 |
        ForEach-Object { [string]$_ })
    if ($LASTEXITCODE -ne 0) {
        throw "Response owner receipt signature verification failed: $($output -join ' ')"
    }
    try { $proof = ($output -join "`n") | ConvertFrom-Json -Depth 20 } catch {
        throw "Response owner receipt verifier emitted invalid proof JSON."
    }
    if ([string]$proof.state -cne "passed" -or
        [string]$proof.signing_input_sha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw "Response owner receipt verifier did not return an exact passed proof."
    }
    $proof
}

function Assert-Sprint8BResponseOwnerActionReceipt {
    param(
        [Parameter(Mandatory)]$Response,
        [Parameter(Mandatory)][string]$RawBody,
        [Parameter(Mandatory)][string]$IdempotencyKey,
        [Parameter(Mandatory)][string]$ExpectedLogicalKey,
        [Parameter(Mandatory)][string]$ExpectedAction,
        [Parameter(Mandatory)][string]$ExpectedActorId,
        [Parameter(Mandatory)][string]$ExpectedFormId,
        [Parameter(Mandatory)][string]$ExpectedFormVersionId,
        [Parameter(Mandatory)][string]$ExpectedNodeId,
        [string]$ExpectedResponseId,
        [Parameter(Mandatory)][string]$ExpectedChangeKind,
        [AllowNull()][string]$ExpectedTombstoneReason,
        [Parameter(Mandatory)][string]$PublicKey
    )

    Assert-Sprint8BExactProperties -Value $Response -Label "Response owner action response" `
        -Expected @("schema_version", "replayed", "signed_receipt")
    if ([int]$Response.schema_version -ne 1) {
        throw "Response owner action response schema version is not exact."
    }
    $envelope = $Response.signed_receipt
    $signatureProof = Test-Sprint8BResponseOwnerReceiptSignature -Envelope $envelope `
        -PublicKey $PublicKey
    $payload = $envelope.payload
    Assert-Sprint8BExactProperties -Value $payload -Label "Response owner action receipt payload" `
        -Expected @(
            "schema_version", "logical_key", "action", "actor_account_id", "response_id",
            "form_id", "form_version_id", "node_id", "method", "path", "raw_body_digest",
            "idempotency_key_digest", "committed_at", "export"
        )
    Assert-Sprint8BExactProperties -Value $payload.export -Label "Response owner export receipt" `
        -Expected @("change_sequence", "change_kind", "tombstone_reason", "content_digest")
    $responseId = Assert-Sprint8BCanonicalUuid -Value ([string]$payload.response_id) `
        -Label "Response owner receipt Response"
    $committedAt = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParseExact(
        [string]$payload.committed_at,
        "yyyy-MM-dd'T'HH:mm:ss.ffffff'Z'",
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::AssumeUniversal,
        [ref]$committedAt
    )) {
        throw "Response owner action receipt committed_at is not canonical UTC microsecond time."
    }
    if ([int]$payload.schema_version -ne 1 -or
        [string]$payload.logical_key -cne $ExpectedLogicalKey -or
        [string]$payload.action -cne $ExpectedAction -or
        [string]$payload.actor_account_id -cne $ExpectedActorId -or
        [string]$payload.form_id -cne $ExpectedFormId -or
        [string]$payload.form_version_id -cne $ExpectedFormVersionId -or
        [string]$payload.node_id -cne $ExpectedNodeId -or
        (-not [string]::IsNullOrWhiteSpace($ExpectedResponseId) -and
            $responseId -cne $ExpectedResponseId) -or
        [string]$payload.method -cne "POST" -or
        [string]$payload.path -cne $responseOwnerActionPath -or
        [string]$payload.raw_body_digest -cne "sha256:$(Get-Sprint7ASha256 -Text $RawBody)" -or
        [string]$payload.idempotency_key_digest -cne
            "sha256:$(Get-Sprint7ASha256 -Text $IdempotencyKey)" -or
        [uint64]$payload.export.change_sequence -eq 0 -or
        [string]$payload.export.change_kind -cne $ExpectedChangeKind -or
        [string]$payload.export.tombstone_reason -cne $ExpectedTombstoneReason -or
        [string]$payload.export.content_digest -cnotmatch '^sha256:[0-9a-f]{64}$') {
        throw "Response owner action receipt does not bind the exact action, owner identities, request, and export change."
    }
    [pscustomobject][ordered]@{
        response_id = $responseId
        change_sequence = [uint64]$payload.export.change_sequence
        envelope = $envelope
        signature_verification = $signatureProof
    }
}

function Invoke-Sprint8BResponseOwnerAction {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Token,
        [Parameter(Mandatory)]$Request,
        [Parameter(Mandatory)][string]$IdempotencyKey,
        [Parameter(Mandatory)][string]$ExpectedActorId,
        [Parameter(Mandatory)][string]$ExpectedFormId,
        [Parameter(Mandatory)][string]$ExpectedFormVersionId,
        [Parameter(Mandatory)][string]$ExpectedNodeId,
        [string]$ExpectedResponseId,
        [Parameter(Mandatory)][string]$ExpectedChangeKind,
        [AllowNull()][string]$ExpectedTombstoneReason,
        [Parameter(Mandatory)][string]$PublicKey
    )

    $rawBody = $Request | ConvertTo-Json -Depth 100 -Compress
    $headers = @{
        Authorization = "Bearer $Token"
        "x-idempotency-key" = $IdempotencyKey
    }
    $arguments = @{
        BaseUrl = $BaseUrl
        Path = $responseOwnerActionPath
        Body = $rawBody
        ContentType = $responseOwnerActionMediaType
        Headers = $headers
    }
    $initial = Invoke-Sprint8BFixtureHttpRequest @arguments
    if ([bool]$initial.document.replayed) {
        throw "First Response owner action unexpectedly replayed."
    }
    $validated = Assert-Sprint8BResponseOwnerActionReceipt -Response $initial.document `
        -RawBody $rawBody -IdempotencyKey $IdempotencyKey `
        -ExpectedLogicalKey ([string]$Request.logical_key) `
        -ExpectedAction ([string]$Request.action) -ExpectedActorId $ExpectedActorId `
        -ExpectedFormId $ExpectedFormId -ExpectedFormVersionId $ExpectedFormVersionId `
        -ExpectedNodeId $ExpectedNodeId -ExpectedResponseId $ExpectedResponseId `
        -ExpectedChangeKind $ExpectedChangeKind -ExpectedTombstoneReason $ExpectedTombstoneReason `
        -PublicKey $PublicKey

    $replay = Invoke-Sprint8BFixtureHttpRequest @arguments
    if (-not [bool]$replay.document.replayed -or
        (($replay.document.signed_receipt | ConvertTo-Json -Depth 100 -Compress) -cne
            ($initial.document.signed_receipt | ConvertTo-Json -Depth 100 -Compress))) {
        throw "Response owner action idempotent replay did not return the exact signed receipt."
    }
    Assert-Sprint8BResponseOwnerActionReceipt -Response $replay.document `
        -RawBody $rawBody -IdempotencyKey $IdempotencyKey `
        -ExpectedLogicalKey ([string]$Request.logical_key) `
        -ExpectedAction ([string]$Request.action) -ExpectedActorId $ExpectedActorId `
        -ExpectedFormId $ExpectedFormId -ExpectedFormVersionId $ExpectedFormVersionId `
        -ExpectedNodeId $ExpectedNodeId -ExpectedResponseId $validated.response_id `
        -ExpectedChangeKind $ExpectedChangeKind -ExpectedTombstoneReason $ExpectedTombstoneReason `
        -PublicKey $PublicKey | Out-Null

    [pscustomobject][ordered]@{
        logical_key = [string]$Request.logical_key
        action = [string]$Request.action
        response_id = [string]$validated.response_id
        idempotency_key_digest = "sha256:$(Get-Sprint7ASha256 -Text $IdempotencyKey)"
        raw_body_digest = "sha256:$(Get-Sprint7ASha256 -Text $rawBody)"
        change_sequence = [uint64]$validated.change_sequence
        change_kind = $ExpectedChangeKind
        tombstone_reason = $ExpectedTombstoneReason
        replay_verified = $true
        signed_receipt = $validated.envelope
        signature_verification = $validated.signature_verification
    }
}

function Get-Sprint8BCoreFixtureIdentities {
    param(
        [Parameter(Mandatory)]$CoreReceipt,
        [Parameter(Mandatory)]$ReferenceFixture,
        [Parameter(Mandatory)]$Blueprint
    )

    $scopeKeys = @("scope.full", "scope.restricted", "scope.confidential", "scope.disjoint")
    $actorKeys = @($ReferenceFixture.actors | ForEach-Object { [string]$_.key })
    $formKeys = @($ReferenceFixture.form_versions | ForEach-Object { [string]$_.key })
    $preInitialResponseKeys = @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.outside-scope"
    )
    $expectedKeys = @(
        $scopeKeys
        $actorKeys
        $formKeys | ForEach-Object {
            "$_.form_id"; "$_.form_version_id"; "$_.dataset_source"; "$_.schema"
        }
        $preInitialResponseKeys
    ) | Sort-Object
    $actualKeys = @($CoreReceipt.resource_ids.PSObject.Properties.Name | Sort-Object)
    if (($expectedKeys -join "`n") -cne ($actualKeys -join "`n")) {
        throw "Core owner receipt keys are not set-equal to the canonical Sprint 8B Core fixture identities."
    }

    $fixtureActors = @($ReferenceFixture.actors | Where-Object {
        [string]$_.key -cne "actor.admin"
    })
    $blueprintActors = @($Blueprint.core.bootstrap.value.actors)
    if ((@($fixtureActors.key | Sort-Object) -join "`n") -cne
        (@($blueprintActors.resource_key | Sort-Object) -join "`n")) {
        throw "Reference Blueprint actor keys are not set-equal to the non-admin fixture actors."
    }
    foreach ($fixtureActor in $fixtureActors) {
        $actorKey = [string]$fixtureActor.key
        $blueprintActor = @($blueprintActors | Where-Object {
            [string]$_.resource_key -ceq $actorKey
        })
        if ($blueprintActor.Count -ne 1) {
            throw "Reference Blueprint actor '$actorKey' is missing or duplicated."
        }
        $fixtureCapabilities = @($fixtureActor.capabilities | ForEach-Object { [string]$_ })
        $blueprintCapabilities = @($blueprintActor[0].capabilities | ForEach-Object { [string]$_ })
        if (@($fixtureCapabilities | Sort-Object -Unique).Count -ne $fixtureCapabilities.Count -or
            @($blueprintCapabilities | Sort-Object -Unique).Count -ne $blueprintCapabilities.Count -or
            ((@($fixtureCapabilities | Sort-Object) -join "`n") -cne
                (@($blueprintCapabilities | Sort-Object) -join "`n")) -or
            @($blueprintActor[0].scope_node_keys).Count -ne 1 -or
            [string]$blueprintActor[0].scope_node_keys[0] -cne [string]$fixtureActor.scope) {
            throw "Reference Blueprint actor '$actorKey' capability/scope tuple does not match the fixture contract."
        }
    }

    $scopes = [ordered]@{}
    foreach ($key in $scopeKeys) {
        $scopes[$key] = Assert-Sprint8BCanonicalUuid `
            -Value (Get-Sprint8BReceiptResourceValue -ResourceIds $CoreReceipt.resource_ids `
                -ResourceKey $key) -Label "Core scope '$key'"
    }
    $actors = [ordered]@{}
    foreach ($key in $actorKeys) {
        $actors[$key] = Assert-Sprint8BCanonicalUuid `
            -Value (Get-Sprint8BReceiptResourceValue -ResourceIds $CoreReceipt.resource_ids `
                -ResourceKey $key) -Label "Core actor '$key'"
    }
    $forms = [ordered]@{}
    $formDefinitions = @($Blueprint.core.bootstrap.value.forms)
    $definitionKeyText = @($formDefinitions.resource_key | Sort-Object) -join "`n"
    $fixtureKeyText = @($formKeys | Sort-Object) -join "`n"
    if ($definitionKeyText -cne $fixtureKeyText) {
        throw "Reference Blueprint Form definitions are not set-equal to the fixture contract."
    }
    foreach ($key in $formKeys) {
        $definition = @($formDefinitions | Where-Object {
            [string]$_.resource_key -ceq $key
        })
        if ($definition.Count -ne 1) {
            throw "Reference Blueprint does not define exact Form '$key'."
        }
        $formId = Assert-Sprint8BCanonicalUuid `
            -Value (Get-Sprint8BReceiptResourceValue -ResourceIds $CoreReceipt.resource_ids `
                -ResourceKey "$key.form_id") -Label "Core Form '$key'"
        $formVersionId = Assert-Sprint8BCanonicalUuid `
            -Value (Get-Sprint8BReceiptResourceValue -ResourceIds $CoreReceipt.resource_ids `
                -ResourceKey "$key.form_version_id") -Label "Core FormVersion '$key'"
        $sourceJson = Get-Sprint8BReceiptResourceValue -ResourceIds $CoreReceipt.resource_ids `
            -ResourceKey "$key.dataset_source"
        try { $source = $sourceJson | ConvertFrom-Json -Depth 20 } catch {
            throw "Core Dataset source '$key' is not canonical JSON."
        }
        $sourceKeys = @($source.PSObject.Properties.Name)
        if (($sourceKeys -join "`n") -cne (@("kind", "alias", "form_id", "form_version_id") -join "`n") -or
            [string]$source.kind -cne "form" -or
            [string]::IsNullOrWhiteSpace([string]$source.alias) -or
            [string]$source.form_id -cne $formId -or
            [string]$source.form_version_id -cne $formVersionId) {
            throw "Core Dataset source '$key' does not bind its exact Form/FormVersion identity."
        }
        $canonicalSource = [ordered]@{
            kind = "form"
            alias = [string]$source.alias
            form_id = $formId
            form_version_id = $formVersionId
        } | ConvertTo-Json -Depth 10 -Compress
        if ($sourceJson -cne $canonicalSource) {
            throw "Core Dataset source '$key' is not the exact canonical JSON encoding."
        }
        $schemaJson = Get-Sprint8BReceiptResourceValue -ResourceIds $CoreReceipt.resource_ids `
            -ResourceKey "$key.schema"
        try { $schema = $schemaJson | ConvertFrom-Json -Depth 100 } catch {
            throw "Core FormVersion schema '$key' is not valid JSON."
        }
        $validatedSchema = Assert-Sprint8BFormSchema -Schema $schema -SchemaJson $schemaJson `
            -FormDefinition $definition[0] -FormId $formId -FormVersionId $formVersionId `
            -Scopes ([pscustomobject]$scopes)
        $forms[$key] = [pscustomobject][ordered]@{
            form_id = $formId
            form_version_id = $formVersionId
            dataset_source = $source
            dataset_source_json = $sourceJson
            schema = $validatedSchema.schema
            schema_json = $validatedSchema.schema_json
            source_scope_digest = $validatedSchema.source_scope_digest
            content_digest = $validatedSchema.content_digest
            exact_field_type_option_layout = $validatedSchema.exact_field_type_option_layout
        }
    }
    $responses = [ordered]@{}
    foreach ($key in $preInitialResponseKeys) {
        $responses[$key] = [pscustomobject][ordered]@{
            response_id = Assert-Sprint8BCanonicalUuid `
                -Value (Get-Sprint8BReceiptResourceValue -ResourceIds $CoreReceipt.resource_ids `
                    -ResourceKey $key) -Label "Core pre-initial Response '$key'"
            provenance = "signed-core-bootstrap-receipt"
        }
    }
    [pscustomobject][ordered]@{
        scopes = [pscustomobject]$scopes
        actors = [pscustomobject]$actors
        forms = [pscustomobject]$forms
        responses = [pscustomobject]$responses
    }
}

function Assert-Sprint8BBootstrapReceiptEnvelope {
    param(
        [Parameter(Mandatory)]$ApplyResponse,
        [Parameter(Mandatory)]$ReferenceFixture,
        [Parameter(Mandatory)]$Blueprint
    )

    if ($null -eq $ApplyResponse.operation -or $null -eq $ApplyResponse.receipt -or
        [string]$ApplyResponse.operation.state -cne "succeeded" -or
        [bool]$ApplyResponse.receipt.no_op) {
        throw "Fixture preparation requires one successful non-no-op from-empty apply response."
    }
    $ownerReceipts = @($ApplyResponse.receipt.bootstrap_receipts)
    $requiredOwners = @(
        "core",
        "tessara.datasets",
        "tessara.components",
        "tessara.dashboards",
        "tessara.reference.scoped-records"
    )
    if (@($ownerReceipts).Count -ne $requiredOwners.Count -or
        ((@($ownerReceipts.owner | Sort-Object) -join "`n") -cne
            (@($requiredOwners | Sort-Object) -join "`n"))) {
        throw "Fixture preparation did not receive the exact Sprint 8B owner receipt set."
    }
    $expectedOwnerSchemas = [ordered]@{
        "core" = "tessara.io/core-bootstrap/v1"
        "tessara.datasets" = "tessara.io/dataset-bootstrap/v1"
        "tessara.components" = "tessara.io/component-bootstrap/v1"
        "tessara.dashboards" = "tessara.io/dashboard-bootstrap/v2"
        "tessara.reference.scoped-records" = "tessara.io/scoped-records-bootstrap/v1"
    }
    foreach ($owner in $requiredOwners) {
        $ownerReceipt = Get-Sprint8BOwnerReceipt -Receipts $ownerReceipts -Owner $owner
        if ([bool]$ownerReceipt.changed -ne $true -or
            [string]$ownerReceipt.schema_version -cne [string]$expectedOwnerSchemas[$owner] -or
            [string]$ownerReceipt.input_digest -cnotmatch '^sha256:[0-9a-f]{64}$' -or
            [string]$ownerReceipt.result_digest -cnotmatch '^sha256:[0-9a-f]{64}$') {
            throw "Owner receipt '$owner' is not an authenticated changed first-apply receipt."
        }
    }

    if ([int]$ReferenceFixture.schema_version -ne 1 -or
        [string]$ReferenceFixture.contract -cne "tessara.sprint-8b.reference-fixture" -or
        [string]$ReferenceFixture.identity_policy -cne
            "logical-keys-resolve-only-from-signed-owner-receipts-and-typed-read-back") {
        throw "Sprint 8B reference fixture identity contract is invalid."
    }

    $coreReceipt = Get-Sprint8BOwnerReceipt -Receipts $ownerReceipts -Owner "core"
    $coreIdentities = Get-Sprint8BCoreFixtureIdentities -CoreReceipt $coreReceipt `
        -ReferenceFixture $ReferenceFixture -Blueprint $Blueprint

    $datasetReceipt = Get-Sprint8BOwnerReceipt -Receipts $ownerReceipts -Owner "tessara.datasets"
    $expectedDatasets = @($ReferenceFixture.datasets | Where-Object {
        [string]$_.expected -notmatch 'rejected'
    } | ForEach-Object { [string]$_.key })
    $expectedDatasetKeys = @($expectedDatasets | ForEach-Object { $_; "$_.read_back" } | Sort-Object)
    $actualDatasetKeys = @($datasetReceipt.resource_ids.PSObject.Properties.Name | Sort-Object)
    if (($actualDatasetKeys -join "`n") -cne ($expectedDatasetKeys -join "`n")) {
        throw "Dataset owner receipt keys are not set-equal to the canonical accepted Dataset fixture keys."
    }

    $datasetIdentities = [ordered]@{}
    $installationIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($datasetKey in $expectedDatasets) {
        $majorLineJson = Get-Sprint8BReceiptResourceValue `
            -ResourceIds $datasetReceipt.resource_ids -ResourceKey $datasetKey
        $majorLine = $majorLineJson | ConvertFrom-Json -Depth 30
        $readBackProperty = "$datasetKey.read_back"
        $readBackJson = Get-Sprint8BReceiptResourceValue `
            -ResourceIds $datasetReceipt.resource_ids -ResourceKey $readBackProperty
        $readBack = $readBackJson | ConvertFrom-Json -Depth 30
        if ($null -eq $majorLine.PSObject.Properties['reference'] -or
            $null -eq $readBack.PSObject.Properties['dataset'] -or
            $null -eq $readBack.PSObject.Properties['revision'] -or
            $null -eq $readBack.PSObject.Properties['major_line'] -or
            $null -eq $readBack.dataset.PSObject.Properties['reference'] -or
            $null -eq $readBack.revision.PSObject.Properties['reference'] -or
            $null -eq $readBack.major_line.PSObject.Properties['reference']) {
            throw "Dataset logical key '$datasetKey' lacks the required typed read-back object shape (major=$majorLineJson; read_back=$readBackJson)."
        }
        if ([int]$readBack.schema_version -ne 2 -or
            [string]$majorLine.reference.resource_type -cne "tessara.datasets.dataset_major_line" -or
            [string]$majorLine.reference.resource_id -cnotmatch
                '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}@[1-9][0-9]*$' -or
            [string]$readBack.dataset.reference.resource_type -cne "tessara.datasets.dataset" -or
            [string]$readBack.dataset.reference.resource_id -cnotmatch
                '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$' -or
            [string]$readBack.revision.reference.resource_type -cne "tessara.datasets.dataset_revision" -or
            [string]$readBack.revision.reference.resource_id -cnotmatch
                '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$' -or
            [string]$readBack.major_line.reference.resource_type -cne "tessara.datasets.dataset_major_line" -or
            (($majorLine | ConvertTo-Json -Depth 30 -Compress) -cne
                ($readBack.major_line | ConvertTo-Json -Depth 30 -Compress))) {
            throw "Dataset logical key '$datasetKey' lacks exact typed Dataset v2 read-back."
        }
        foreach ($reference in @(
            $readBack.dataset.reference,
            $readBack.revision.reference,
            $readBack.major_line.reference
        )) {
            if ([string]$reference.owner.kind -cne "module_instance" -or
                [string]$reference.installation_id -cne [string]$reference.owner.installation_id -or
                [string]::IsNullOrWhiteSpace([string]$reference.owner.module_instance_id)) {
                throw "Dataset logical key '$datasetKey' is not owned by one Module Instance."
            }
            [void]$installationIds.Add([string]$reference.installation_id)
        }
        $datasetIdentities[$datasetKey] = $readBack
    }

    $componentReceipt = Get-Sprint8BOwnerReceipt -Receipts $ownerReceipts -Owner "tessara.components"
    $expectedComponents = @($ReferenceFixture.downstream.components | Where-Object {
        [string]$_.expected -cne 'incompatible'
    } | ForEach-Object { [string]$_.key } | Sort-Object)
    $actualComponentKeys = @($componentReceipt.resource_ids.PSObject.Properties.Name | Sort-Object)
    if (($actualComponentKeys -join "`n") -cne ($expectedComponents -join "`n")) {
        throw "Component owner receipt keys are not set-equal to the canonical compatible fixture keys."
    }
    $componentIdentities = [ordered]@{}
    foreach ($componentKey in $expectedComponents) {
        $reference = (Get-Sprint8BReceiptResourceValue `
            -ResourceIds $componentReceipt.resource_ids -ResourceKey $componentKey) | ConvertFrom-Json -Depth 30
        if ([string]$reference.reference.resource_type -cne "tessara.components.component_version" -or
            [string]$reference.reference.resource_id -cnotmatch
                '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$' -or
            [string]$reference.reference.owner.kind -cne "module_instance" -or
            [string]$reference.reference.installation_id -cne
                [string]$reference.reference.owner.installation_id) {
            throw "Component logical key '$componentKey' lacks exact typed owner read-back."
        }
        [void]$installationIds.Add([string]$reference.reference.installation_id)
        $componentIdentities[$componentKey] = $reference
    }

    $dashboardReceipt = Get-Sprint8BOwnerReceipt -Receipts $ownerReceipts -Owner "tessara.dashboards"
    $dashboardKeys = @($dashboardReceipt.resource_ids.PSObject.Properties.Name | Sort-Object)
    $expectedPlacementReceiptKeys = @($ReferenceFixture.downstream.dashboard.placements |
        ForEach-Object { "placement.$([string]$_.key)" })
    $expectedDashboardKeys = @("dashboard", "external_key") + $expectedPlacementReceiptKeys |
        Sort-Object
    if (($dashboardKeys -join "`n") -cne ($expectedDashboardKeys -join "`n") -or
        [string]$dashboardReceipt.resource_ids.external_key -cne
            [string]$ReferenceFixture.downstream.dashboard.key -or
        [string]$dashboardReceipt.resource_ids.dashboard -cnotmatch
            '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$') {
        throw "Dashboard owner receipt lacks the exact logical-key/physical-ID read-back pair."
    }
    $dashboardPlacements = [ordered]@{}
    foreach ($placement in @($ReferenceFixture.downstream.dashboard.placements)) {
        $receiptKey = "placement.$([string]$placement.key)"
        $dashboardPlacements[[string]$placement.key] = Assert-Sprint8BCanonicalUuid `
            -Value (Get-Sprint8BReceiptResourceValue -ResourceIds $dashboardReceipt.resource_ids `
                -ResourceKey $receiptKey) -Label "Dashboard placement '$([string]$placement.key)'"
    }
    if ($installationIds.Count -ne 1) {
        throw "Typed Dataset and Component read-backs do not share exactly one installation identity."
    }

    [pscustomobject][ordered]@{
        owner_receipts = $ownerReceipts
        installation_id = @($installationIds)[0]
        logical_identities = [pscustomobject][ordered]@{
            core = $coreIdentities
            datasets = [pscustomobject]$datasetIdentities
            components = [pscustomobject]$componentIdentities
            dashboard = [pscustomobject][ordered]@{
                key = [string]$dashboardReceipt.resource_ids.external_key
                id = [string]$dashboardReceipt.resource_ids.dashboard
                placements = [pscustomobject]$dashboardPlacements
            }
        }
    }
}

function New-Sprint8BFixtureReceipt {
    param(
        [Parameter(Mandatory)]$ApplyResponse,
        [Parameter(Mandatory)]$ReferenceFixture,
        [Parameter(Mandatory)]$Blueprint,
        [string]$ResolvedComposeProject,
        [Parameter(Mandatory)][string]$ApplyResponseSha256
    )

    $validated = Assert-Sprint8BBootstrapReceiptEnvelope `
        -ApplyResponse $ApplyResponse -ReferenceFixture $ReferenceFixture -Blueprint $Blueprint
    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "owner-controlled-uat-fixture-preparation"
        state = "passed"
        compose_project = if ([string]::IsNullOrWhiteSpace($ResolvedComposeProject)) { $null } else { $ResolvedComposeProject }
        apply_response_sha256 = $ApplyResponseSha256
        operation_id = [string]$ApplyResponse.operation.operation_id
        receipt_digest = [string]$ApplyResponse.operation.receipt_digest
        installation_id = [string]$validated.installation_id
        mutation_policy = [string]$ReferenceFixture.mutation_policy
        identity_policy = [string]$ReferenceFixture.identity_policy
        logical_identities = $validated.logical_identities
        owner_receipt_digests = @($validated.owner_receipts | ForEach-Object {
            [pscustomobject][ordered]@{
                owner = [string]$_.owner
                input_digest = [string]$_.input_digest
                result_digest = [string]$_.result_digest
            }
        })
        forbidden_proofs = [pscustomobject][ordered]@{
            predicted_uuid = "not_used"
            foreign_owner_database_write = "not_used"
            response_sql_mutation = "not_used"
            copied_count_as_identity = "not_used"
        }
        restoration = [pscustomobject][ordered]@{
            state = "passed"
            basis = "successful_from_empty_owner_receipts"
        }
    }
}

function Add-Sprint8BPostBootstrapResponseFixtures {
    param(
        [Parameter(Mandatory)]$FixtureReceipt,
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Email,
        [Parameter(Mandatory)][string]$Password,
        [Parameter(Mandatory)][string]$PublicKey
    )

    $core = $FixtureReceipt.logical_identities.core
    $form = $core.forms.'form.primary/v1'
    $nodeId = [string]$core.scopes.'scope.full'
    $actorId = [string]$core.actors.'actor.admin'
    $token = Get-Sprint8BFixtureOwnerToken -BaseUrl $BaseUrl -Email $Email -Password $Password
    $receipts = [Collections.Generic.List[object]]::new()
    $scenarioReceipts = [ordered]@{}
    $invoke = {
        param(
            [string]$LogicalKey,
            [string]$Action,
            [AllowNull()][string]$ResponseId,
            [AllowNull()]$Values,
            [string]$ChangeKind,
            [AllowNull()][string]$Reason
        )
        $request = [ordered]@{ schema_version = 1; action = $Action; logical_key = $LogicalKey }
        if ($Action -ceq "create") {
            $request.form_version_id = [string]$form.form_version_id
            $request.node_id = $nodeId
            $request.values = $Values
        } else {
            $request.response_id = $ResponseId
            if ($Action -ceq "correct") { $request.values = $Values }
        }
        $ordinal = $receipts.Count + 1
        $idempotencyKey = "sprint-8b:$($LogicalKey.Replace('.', '-')):${Action}:$ordinal"
        $receipt = Invoke-Sprint8BResponseOwnerAction -BaseUrl $BaseUrl -Token $token `
            -Request ([pscustomobject]$request) -IdempotencyKey $idempotencyKey `
            -ExpectedActorId $actorId -ExpectedFormId ([string]$form.form_id) `
            -ExpectedFormVersionId ([string]$form.form_version_id) -ExpectedNodeId $nodeId `
            -ExpectedResponseId $ResponseId -ExpectedChangeKind $ChangeKind `
            -ExpectedTombstoneReason $Reason -PublicKey $PublicKey
        if ($receipts.Count -gt 0 -and
            [uint64]$receipt.change_sequence -le [uint64]$receipts[$receipts.Count - 1].change_sequence) {
            throw "Response owner action change sequences are not strictly monotonic."
        }
        $receipts.Add($receipt)
        if (-not $scenarioReceipts.Contains($LogicalKey)) {
            $scenarioReceipts[$LogicalKey] = [Collections.Generic.List[int]]::new()
        }
        $scenarioReceipts[$LogicalKey].Add($receipts.Count - 1)
        $receipt
    }

    $values = {
        param([string]$Label, [int]$Amount, [string]$Category)
        [ordered]@{ label = $Label; amount = $Amount; category = $Category }
    }
    $new = & $invoke "response.new" "create" $null `
        (& $values "New after initial sync" 40 "C") "upsert" $null

    $correctedCreate = & $invoke "response.corrected" "create" $null `
        (& $values "Before correction" 50 "C") "upsert" $null
    & $invoke "response.corrected" "correct" $correctedCreate.response_id `
        (& $values "Corrected complete values" 55 "D") "upsert" $null | Out-Null

    $statusOutCreate = & $invoke "response.status-out" "create" $null `
        (& $values "Status out" 60 "D") "upsert" $null
    & $invoke "response.status-out" "status_out" $statusOutCreate.response_id `
        $null "tombstone" "status_excluded" | Out-Null

    $statusInCreate = & $invoke "response.status-in" "create" $null `
        (& $values "Status in" 70 "E") "upsert" $null
    & $invoke "response.status-in" "status_out" $statusInCreate.response_id `
        $null "tombstone" "status_excluded" | Out-Null
    & $invoke "response.status-in" "status_in" $statusInCreate.response_id `
        $null "upsert" $null | Out-Null

    $redactedCreate = & $invoke "response.redacted" "create" $null `
        (& $values "Redact me" 80 "E") "upsert" $null
    & $invoke "response.redacted" "redact" $redactedCreate.response_id `
        $null "tombstone" "redacted" | Out-Null

    $deletedCreate = & $invoke "response.deleted" "create" $null `
        (& $values "Delete me" 90 "F") "upsert" $null
    & $invoke "response.deleted" "delete" $deletedCreate.response_id `
        $null "tombstone" "deleted" | Out-Null

    foreach ($logicalKey in @(
        "response.new", "response.corrected", "response.status-out", "response.status-in",
        "response.redacted", "response.deleted"
    )) {
        $indices = @($scenarioReceipts[$logicalKey])
        $scenarioActions = @($indices | ForEach-Object { $receipts[$_] })
        $responseIds = @($scenarioActions.response_id | Sort-Object -Unique)
        if ($responseIds.Count -ne 1) {
            throw "Response owner scenario '$logicalKey' did not retain one exact Response identity."
        }
        $core.responses | Add-Member -NotePropertyName $logicalKey -NotePropertyValue `
            ([pscustomobject][ordered]@{
                response_id = [string]$responseIds[0]
                provenance = "signed-core-response-owner-action-receipt"
                receipt_indices = @($indices)
                terminal_action = [string]$scenarioActions[-1].action
                terminal_change_kind = [string]$scenarioActions[-1].change_kind
                terminal_tombstone_reason = $scenarioActions[-1].tombstone_reason
            })
    }
    $expectedResponseKeys = @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.new",
        "response.corrected", "response.status-out", "response.status-in", "response.redacted",
        "response.deleted", "response.outside-scope"
    ) | Sort-Object
    $actualResponseKeys = @($core.responses.PSObject.Properties.Name | Sort-Object)
    if (($expectedResponseKeys -join "`n") -cne ($actualResponseKeys -join "`n")) {
        throw "Prepared Response logical identities are not set-equal to the Sprint 8B fixture contract."
    }
    $actionNames = @($receipts.action | Sort-Object -Unique)
    if (($actionNames -join "`n") -cne
        (@("correct", "create", "delete", "redact", "status_in", "status_out") -join "`n")) {
        throw "Prepared Response fixtures did not exercise every exact owner action."
    }
    $FixtureReceipt | Add-Member -NotePropertyName response_owner_actions -NotePropertyValue `
        ([pscustomobject][ordered]@{
            schema_version = 1
            endpoint = $responseOwnerActionPath
            media_type = $responseOwnerActionMediaType
            authentication = "bearer-owner-capability"
            signature = "ed25519-purpose-bound"
            replay = "exact-signed-receipt"
            action_count = $receipts.Count
            strictly_monotonic_change_sequence = "passed"
            receipts = @($receipts)
        })
    $FixtureReceipt.restoration.basis =
        "successful-from-empty-owner-receipts-plus-authenticated-post-bootstrap-response-actions"
    $FixtureReceipt
}

function New-Sprint8BFixtureSelfTestSchema {
    param(
        [Parameter(Mandatory)]$FormDefinition,
        [Parameter(Mandatory)][string]$FormId,
        [Parameter(Mandatory)][string]$FormVersionId,
        [Parameter(Mandatory)]$CoreResources,
        [Parameter(Mandatory)][int]$Ordinal
    )

    $sectionId = "01980000-0015-7000-8000-{0:d12}" -f $Ordinal
    $sections = @([ordered]@{
        section_id = $sectionId
        key = $sectionId
        label = "Response"
        position = 0
    })
    $fieldOrdinal = ($Ordinal * 100)
    $fields = @($FormDefinition.fields | Sort-Object `
        @{ Expression = { [int]$_.position } },
        @{ Expression = { [int]$_.grid_row } },
        @{ Expression = { [int]$_.grid_column } },
        @{ Expression = { [string]$_.label } },
        @{ Expression = { [string]$_.key } } | ForEach-Object {
            $fieldOrdinal++
            [ordered]@{
                field_id = "01980000-0016-7000-8000-{0:d12}" -f $fieldOrdinal
                key = [string]$_.key
                label = ([string]$_.label).Trim()
                field_type = [string]$_.field_type
                required = [bool]$_.required
                options = @()
                section_id = $sectionId
                position = [int]$_.position
                grid_row = [int]$_.grid_row
                grid_column = [int]$_.grid_column
            }
        })
    $scopeIds = [string[]]@($FormDefinition.scope_node_keys | ForEach-Object {
        [string]$CoreResources[[string]$_]
    })
    [Array]::Sort($scopeIds, [StringComparer]::Ordinal)
    $scopeDigest = Get-Sprint8BJcsDigest -Value @($scopeIds)
    $major = [int](([string]$FormDefinition.version_label).Split('.')[0])
    $schema = [ordered]@{
        schema_version = 1
        form_id = $FormId
        form_version_id = $FormVersionId
        form_name = ([string]$FormDefinition.name).Trim()
        form_slug = [string]$FormDefinition.slug
        version_label = [string]$FormDefinition.version_label
        version_major = $major
        source_scope_node_ids = @($scopeIds)
        source_scope_revision = $scopeDigest
        source_scope_digest = $scopeDigest
        content_revision = ""
        content_digest = ""
        sections = $sections
        fields = $fields
    }
    $content = [ordered]@{
        schema_version = $schema.schema_version
        form_id = $schema.form_id
        form_version_id = $schema.form_version_id
        form_name = $schema.form_name
        form_slug = $schema.form_slug
        version_label = $schema.version_label
        version_major = $schema.version_major
        source_scope_node_ids = $schema.source_scope_node_ids
        source_scope_revision = $schema.source_scope_revision
        source_scope_digest = $schema.source_scope_digest
        sections = $schema.sections
        fields = $schema.fields
    }
    $contentDigest = Get-Sprint8BJcsDigest -Value $content
    $schema.content_revision = $contentDigest
    $schema.content_digest = $contentDigest
    $schema
}

function New-Sprint8BFixtureSelfTestInput {
    param(
        [Parameter(Mandatory)]$ReferenceFixture,
        [Parameter(Mandatory)]$Blueprint
    )

    $installationId = "01980000-0000-7000-8000-00000000008b"
    $datasetOwner = "01980000-0000-7000-8000-000000000081"
    $componentOwner = "01980000-0000-7000-8000-000000000082"
    $digest = "sha256:$('a' * 64)"
    $coreResources = [ordered]@{}
    $coreIndex = 1
    foreach ($scope in @("scope.full", "scope.restricted", "scope.confidential", "scope.disjoint")) {
        $coreResources[$scope] = "01980000-0010-7000-8000-{0:d12}" -f $coreIndex
        $coreIndex++
    }
    foreach ($actor in @($ReferenceFixture.actors)) {
        $coreResources[[string]$actor.key] = "01980000-0011-7000-8000-{0:d12}" -f $coreIndex
        $coreIndex++
    }
    foreach ($form in @($ReferenceFixture.form_versions)) {
        $formKey = [string]$form.key
        $formDefinition = @($Blueprint.core.bootstrap.value.forms | Where-Object {
            [string]$_.resource_key -ceq $formKey
        })[0]
        $formId = "01980000-0012-7000-8000-{0:d12}" -f $coreIndex
        $formVersionId = "01980000-0013-7000-8000-{0:d12}" -f $coreIndex
        $coreResources["$formKey.form_id"] = $formId
        $coreResources["$formKey.form_version_id"] = $formVersionId
        $coreResources["$formKey.dataset_source"] = ([ordered]@{
            kind = "form"
            alias = "source-$coreIndex"
            form_id = $formId
            form_version_id = $formVersionId
        } | ConvertTo-Json -Depth 10 -Compress)
        $coreResources["$formKey.schema"] = (New-Sprint8BFixtureSelfTestSchema `
            -FormDefinition $formDefinition -FormId $formId -FormVersionId $formVersionId `
            -CoreResources $coreResources -Ordinal $coreIndex) |
            ConvertTo-Json -Depth 100 -Compress
        $coreIndex++
    }
    foreach ($responseKey in @(
        "response.initial", "response.same-time-a", "response.same-time-b", "response.outside-scope"
    )) {
        $coreResources[$responseKey] = "01980000-0014-7000-8000-{0:d12}" -f $coreIndex
        $coreIndex++
    }
    $datasetResources = [ordered]@{}
    $index = 1
    foreach ($dataset in @($ReferenceFixture.datasets | Where-Object {
        [string]$_.expected -notmatch 'rejected'
    })) {
        $datasetId = "01980000-0001-7000-8000-{0:d12}" -f $index
        $revisionId = "01980000-0002-7000-8000-{0:d12}" -f $index
        $newReference = {
            param([string]$ResourceType, [string]$ResourceId)
            [ordered]@{
                installation_id = $installationId
                owner = [ordered]@{
                    kind = "module_instance"
                    installation_id = $installationId
                    module_instance_id = $datasetOwner
                }
                resource_type = $ResourceType
                resource_id = $ResourceId
            }
        }
        $major = [ordered]@{
            reference = & $newReference "tessara.datasets.dataset_major_line" "${datasetId}@1"
        }
        $readBack = [ordered]@{
            schema_version = 2
            dataset = [ordered]@{ reference = & $newReference "tessara.datasets.dataset" $datasetId }
            revision = [ordered]@{ reference = & $newReference "tessara.datasets.dataset_revision" $revisionId }
            major_line = $major
            materialized_row_count = 1
        }
        $datasetResources[[string]$dataset.key] = ($major | ConvertTo-Json -Depth 20 -Compress)
        $datasetResources["$($dataset.key).read_back"] = ($readBack | ConvertTo-Json -Depth 20 -Compress)
        $index++
    }
    $componentResources = [ordered]@{}
    $index = 1
    foreach ($component in @($ReferenceFixture.downstream.components | Where-Object {
        [string]$_.expected -cne 'incompatible'
    })) {
        $componentResources[[string]$component.key] = ([ordered]@{
            reference = [ordered]@{
                installation_id = $installationId
                owner = [ordered]@{
                    kind = "module_instance"
                    installation_id = $installationId
                    module_instance_id = $componentOwner
                }
                resource_type = "tessara.components.component_version"
                resource_id = "01980000-0003-7000-8000-{0:d12}" -f $index
            }
        } | ConvertTo-Json -Depth 20 -Compress)
        $index++
    }
    $receipt = {
        param($Owner, $Schema, $Resources)
        [pscustomobject][ordered]@{
            owner = $Owner
            schema_version = $Schema
            input_digest = $digest
            result_digest = $digest
            changed = $true
            resource_ids = [pscustomobject]$Resources
        }
    }
    [pscustomobject][ordered]@{
        operation = [pscustomobject][ordered]@{
            operation_id = "01980000-0004-7000-8000-000000000001"
            state = "succeeded"
            receipt_digest = $digest
        }
        receipt = [pscustomobject][ordered]@{
            no_op = $false
            bootstrap_receipts = @(
                (& $receipt "core" "tessara.io/core-bootstrap/v1" $coreResources)
                (& $receipt "tessara.datasets" "tessara.io/dataset-bootstrap/v1" $datasetResources)
                (& $receipt "tessara.components" "tessara.io/component-bootstrap/v1" $componentResources)
                (& $receipt "tessara.dashboards" "tessara.io/dashboard-bootstrap/v2" ([ordered]@{
                    dashboard = "01980000-0006-7000-8000-000000000001"
                    external_key = [string]$ReferenceFixture.downstream.dashboard.key
                    "placement.dataset-stat" = "01980000-0006-7000-8000-000000000002"
                    "placement.dataset-table" = "01980000-0006-7000-8000-000000000003"
                    "placement.dataset-chart" = "01980000-0006-7000-8000-000000000004"
                    "placement.dataset-disjoint" = "01980000-0006-7000-8000-000000000005"
                }))
                (& $receipt "tessara.reference.scoped-records" "tessara.io/scoped-records-bootstrap/v1" ([ordered]@{
                    reference = "01980000-0007-7000-8000-000000000001"
                }))
            )
        }
    }
}

function Test-Sprint8BFixturePreparation {
    $vendorJson = '{"schema_version":1,"state":"passed"}'
    $vendorJsonBytes = [System.Text.Encoding]::UTF8.GetBytes($vendorJson)
    if ((ConvertFrom-Sprint8BHttpContent -Content $vendorJsonBytes) -cne $vendorJson -or
        (ConvertFrom-Sprint8BHttpContent -Content $vendorJson) -cne $vendorJson) {
        throw "Sprint 8B fixture HTTP content decoding did not preserve vendor JSON bytes."
    }
    $timestampJson = '{"committed_at":"2026-08-14T15:13:32.123456Z"}'
    $timestampDocument = $timestampJson | ConvertFrom-Json -Depth 10 -DateKind String
    if ($timestampDocument.committed_at -isnot [string] -or
        [string]$timestampDocument.committed_at -cne "2026-08-14T15:13:32.123456Z") {
        throw "Sprint 8B fixture JSON parsing did not preserve canonical signed timestamps."
    }

    $referenceFixture = Read-Sprint8BFixtureJson -Path $ReferenceFixturePath -Label "Reference fixture contract"
    $blueprint = Read-Sprint8BFixtureJson -Path $BlueprintPath -Label "Reference Blueprint"
    $mock = New-Sprint8BFixtureSelfTestInput -ReferenceFixture $referenceFixture -Blueprint $blueprint
    $receipt = New-Sprint8BFixtureReceipt -ApplyResponse $mock -ReferenceFixture $referenceFixture `
        -Blueprint $blueprint `
        -ResolvedComposeProject "tessara-s8b-fixture-selftest" -ApplyResponseSha256 ("b" * 64)
    if ($receipt.state -cne "passed" -or
        @($receipt.logical_identities.core.scopes.PSObject.Properties).Count -ne 4 -or
        @($receipt.logical_identities.core.actors.PSObject.Properties).Count -ne 7 -or
        @($receipt.logical_identities.core.forms.PSObject.Properties).Count -ne 3 -or
        @($receipt.logical_identities.core.forms.PSObject.Properties.Value | Where-Object {
            [string]$_.exact_field_type_option_layout -ceq "passed"
        }).Count -ne 3 -or
        @($receipt.logical_identities.core.responses.PSObject.Properties).Count -ne 4 -or
        @($receipt.logical_identities.datasets.PSObject.Properties).Count -ne 5 -or
        @($receipt.logical_identities.components.PSObject.Properties).Count -ne 4 -or
        @($receipt.logical_identities.dashboard.placements.PSObject.Properties).Count -ne 4 -or
        $receipt.forbidden_proofs.predicted_uuid -cne "not_used") {
        throw "Sprint 8B fixture receipt self-test did not preserve exact owner read-back."
    }

    $tampered = $mock | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $tamperedDataset = @($tampered.receipt.bootstrap_receipts | Where-Object owner -CEQ "tessara.datasets")[0]
    $tamperedDataset.resource_ids.'dataset.base' = '"01980000-0001-7000-8000-000000000001"'
    try {
        New-Sprint8BFixtureReceipt -ApplyResponse $tampered -ReferenceFixture $referenceFixture `
            -Blueprint $blueprint `
            -ResolvedComposeProject "tessara-s8b-fixture-selftest" -ApplyResponseSha256 ("c" * 64) | Out-Null
        throw "Sprint 8B fixture preparation accepted a predicted/copyable Dataset UUID."
    } catch {
        if ($_.Exception.Message -notmatch "typed .*read-back|required typed read-back") { throw }
    }

    $tamperedCore = $mock | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $coreReceipt = @($tamperedCore.receipt.bootstrap_receipts | Where-Object owner -CEQ "core")[0]
    $coreReceipt.resource_ids.'form.primary/v1.dataset_source' =
        '{"kind":"form","alias":"substituted","form_id":"01980000-ffff-7000-8000-000000000001","form_version_id":"01980000-0013-7000-8000-00000000000c"}'
    try {
        New-Sprint8BFixtureReceipt -ApplyResponse $tamperedCore -ReferenceFixture $referenceFixture `
            -Blueprint $blueprint `
            -ResolvedComposeProject "tessara-s8b-fixture-selftest" `
            -ApplyResponseSha256 ("d" * 64) | Out-Null
        throw "Sprint 8B fixture preparation accepted a substituted Core Dataset source."
    } catch {
        if ($_.Exception.Message -notmatch "does not bind its exact Form/FormVersion") { throw }
    }

    $tamperedSchema = $mock | ConvertTo-Json -Depth 100 | ConvertFrom-Json -Depth 100
    $schemaCoreReceipt = @($tamperedSchema.receipt.bootstrap_receipts | Where-Object owner -CEQ "core")[0]
    $schema = $schemaCoreReceipt.resource_ids.'form.primary/v1.schema' |
        ConvertFrom-Json -Depth 100
    $schema.fields[0].grid_column = 99
    $schemaCoreReceipt.resource_ids.'form.primary/v1.schema' =
        $schema | ConvertTo-Json -Depth 100 -Compress
    try {
        New-Sprint8BFixtureReceipt -ApplyResponse $tamperedSchema `
            -ReferenceFixture $referenceFixture -Blueprint $blueprint `
            -ResolvedComposeProject "tessara-s8b-fixture-selftest" `
            -ApplyResponseSha256 ("e" * 64) | Out-Null
        throw "Sprint 8B fixture preparation accepted FormVersion layout/digest tampering."
    } catch {
        if ($_.Exception.Message -notmatch "field/type/options/layout|content digest") { throw }
    }

    $tamperedActorBlueprint = $blueprint | ConvertTo-Json -Depth 100 |
        ConvertFrom-Json -Depth 100
    $restrictedActor = @($tamperedActorBlueprint.core.bootstrap.value.actors | Where-Object {
        [string]$_.resource_key -ceq "actor.restricted"
    })[0]
    $restrictedActor.capabilities = @("datasets:read")
    try {
        New-Sprint8BFixtureReceipt -ApplyResponse $mock `
            -ReferenceFixture $referenceFixture -Blueprint $tamperedActorBlueprint `
            -ResolvedComposeProject "tessara-s8b-fixture-selftest" `
            -ApplyResponseSha256 ("f" * 64) | Out-Null
        throw "Sprint 8B fixture preparation accepted actor capability/scope drift."
    } catch {
        if ($_.Exception.Message -notmatch "capability/scope tuple") { throw }
    }

    $verifierPath = Join-Path $PSScriptRoot "verify-sprint-8b-response-owner-receipt.mjs"
    $verifierOutput = @(& node $verifierPath --self-test 2>&1 | ForEach-Object { [string]$_ })
    if ($LASTEXITCODE -ne 0) {
        throw "Response owner receipt verifier self-test failed: $($verifierOutput -join ' ')"
    }
    try { $verifierProof = ($verifierOutput -join "`n") | ConvertFrom-Json -Depth 20 } catch {
        throw "Response owner receipt verifier self-test emitted invalid JSON."
    }
    if ([string]$verifierProof.state -cne "passed" -or
        -not [bool]$verifierProof.tamper_rejected) {
        throw "Response owner receipt verifier did not reject signed payload tampering."
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        sprint = "sprint-8b"
        proof = "owner-controlled-uat-fixture-preparation-self-test"
        state = "passed"
        database_free = $true
        scenario_contract_only = $true
        response_owner_action_contract = [pscustomobject][ordered]@{
            path = $responseOwnerActionPath
            media_type = $responseOwnerActionMediaType
            actions = @("create", "correct", "status_out", "status_in", "redact", "delete")
            replay = "exact-signed-receipt"
            ed25519_tamper_rejection = "passed"
        }
    }
}

if ($SelfTest) {
    Test-Sprint8BFixturePreparation | ConvertTo-Json -Depth 20
    return
}

if ([string]::IsNullOrWhiteSpace($ApplyResponsePath)) {
    throw "Live Sprint 8B fixture preparation requires -ApplyResponsePath."
}
if (-not [string]::IsNullOrWhiteSpace($ComposeProject)) {
    Assert-Sprint8BComposeProject -ComposeProject $ComposeProject | Out-Null
}
$resolvedApplyPath = Resolve-Sprint8BRepositoryPath -Path $ApplyResponsePath
$applyResponse = Read-Sprint8BFixtureJson -Path $resolvedApplyPath -Label "Apply response"
$referenceFixture = Read-Sprint8BFixtureJson -Path $ReferenceFixturePath -Label "Reference fixture contract"
$blueprint = Read-Sprint8BFixtureJson -Path $BlueprintPath -Label "Reference Blueprint"
$applyHash = (Get-FileHash -LiteralPath $resolvedApplyPath -Algorithm SHA256).Hash.ToLowerInvariant()
$fixtureReceipt = New-Sprint8BFixtureReceipt -ApplyResponse $applyResponse `
    -ReferenceFixture $referenceFixture -Blueprint $blueprint -ResolvedComposeProject $ComposeProject `
    -ApplyResponseSha256 $applyHash
$resolvedCoreBaseUrl = if (-not [string]::IsNullOrWhiteSpace($CoreBaseUrl)) {
    $CoreBaseUrl
} elseif (-not [string]::IsNullOrWhiteSpace([string]$env:TESSARA_CORE_CONTROL_PORT)) {
    "http://127.0.0.1:$([int]$env:TESSARA_CORE_CONTROL_PORT)"
} else {
    throw "Live fixture preparation requires -CoreBaseUrl or TESSARA_CORE_CONTROL_PORT."
}
$resolvedOwnerActionPublicKey = if (-not [string]::IsNullOrWhiteSpace($OwnerActionPublicKey)) {
    $OwnerActionPublicKey
} elseif (-not [string]::IsNullOrWhiteSpace([string]$env:TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY)) {
    [string]$env:TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY
} else {
    $defaultCoreAuthorizationPublicKey
}
$fixtureReceipt = Add-Sprint8BPostBootstrapResponseFixtures -FixtureReceipt $fixtureReceipt `
    -BaseUrl $resolvedCoreBaseUrl -Email $OwnerActionEmail -Password $OwnerActionPassword `
    -PublicKey $resolvedOwnerActionPublicKey
$published = Publish-Sprint8BHarnessEvidence -Document $fixtureReceipt -OutputPath $OutputPath
$fixtureReceipt | Add-Member -NotePropertyName evidence -NotePropertyValue ([pscustomobject][ordered]@{
    path = $published.path
    sha256 = $published.sha256
})
$fixtureReceipt | ConvertTo-Json -Depth 100

Set-StrictMode -Version Latest

function New-Sprint8AHealthContractException {
    param(
        [Parameter(Mandatory)][string]$Message,
        [ValidateSet("product", "harness", "environment")][string]$Classification = "harness"
    )

    $exception = [IO.InvalidDataException]::new($Message)
    $exception.Data["TessaraFailureClassification"] = $Classification
    $exception
}

function Get-Sprint8AHealthMemberValue {
    param(
        [AllowNull()]$InputObject,
        [Parameter(Mandatory)][string]$Name,
        [switch]$Required
    )

    if ($null -eq $InputObject) {
        if ($Required) {
            throw (New-Sprint8AHealthContractException -Message "Sprint 8A health evidence is missing required member '$Name'.")
        }
        return $null
    }
    if ($InputObject -is [Collections.IDictionary]) {
        if ($InputObject.Contains($Name)) {
            return $InputObject[$Name]
        }
    } else {
        $property = $InputObject.PSObject.Properties[$Name]
        if ($null -ne $property) {
            return $property.Value
        }
    }
    if ($Required) {
        throw (New-Sprint8AHealthContractException -Message "Sprint 8A health evidence is missing required member '$Name'.")
    }
    $null
}

function Get-Sprint8AHealthContract {
    param(
        [Parameter(Mandatory)]
        [ValidateSet("gateway_core", "core_control", "supervisor")]
        [string]$Target
    )

    if ($Target -in @("gateway_core", "core_control")) {
        return [pscustomobject][ordered]@{
            target = $Target
            endpoint_kind = "core"
            method = "GET"
            path = "/health"
            status = 200
            media_type = "text/plain"
            allowed_charsets = @($null, "", "utf-8")
            body_utf8_length = 2L
            body_sha256 = "2689367b205c16ce32ed4200942b8b8b1e262dfc70d9bc9fbc77c49699a4f1df"
            redirects_allowed = $false
        }
    }
    [pscustomobject][ordered]@{
        target = $Target
        endpoint_kind = "supervisor"
        method = "GET"
        path = "/health/ready"
        status = 204
        media_type = $null
        allowed_charsets = @()
        body_utf8_length = 0L
        body_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        redirects_allowed = $false
    }
}

function Get-Sprint8AHealthBytesSha256 {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)

    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        ([BitConverter]::ToString($algorithm.ComputeHash($Bytes)) -replace "-", "").ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
    }
}

function Add-Sprint8AHealthFailureCode {
    param(
        [Parameter(Mandatory)][AllowEmptyCollection()][Collections.Generic.List[string]]$Failures,
        [Parameter(Mandatory)][string]$Code
    )

    if (-not $Failures.Contains($Code)) {
        $Failures.Add($Code)
    }
}

function Test-Sprint8AHealthObservation {
    param([Parameter(Mandatory)]$Observation)

    $target = [string](Get-Sprint8AHealthMemberValue -InputObject $Observation -Name "target" -Required)
    $expectation = Get-Sprint8AHealthContract -Target $target
    $request = Get-Sprint8AHealthMemberValue -InputObject $Observation -Name "request" -Required
    $response = Get-Sprint8AHealthMemberValue -InputObject $Observation -Name "response" -Required
    $transportError = Get-Sprint8AHealthMemberValue -InputObject $Observation -Name "transport_error"
    $failures = [Collections.Generic.List[string]]::new()

    $method = [string](Get-Sprint8AHealthMemberValue -InputObject $request -Name "method" -Required)
    $requestUri = [string](Get-Sprint8AHealthMemberValue -InputObject $request -Name "uri" -Required)
    $requestPath = [string](Get-Sprint8AHealthMemberValue -InputObject $request -Name "path" -Required)
    $timeoutSeconds = [int](Get-Sprint8AHealthMemberValue -InputObject $request -Name "timeout_seconds" -Required)
    $redirectPolicy = [string](Get-Sprint8AHealthMemberValue -InputObject $request -Name "redirect_policy" -Required)
    if ($method -cne [string]$expectation.method) {
        Add-Sprint8AHealthFailureCode -Failures $failures -Code "unexpected_request_method"
    }
    if ($requestPath -cne [string]$expectation.path) {
        Add-Sprint8AHealthFailureCode -Failures $failures -Code "unexpected_request_path"
    }
    if ($redirectPolicy -cne "forbid") {
        Add-Sprint8AHealthFailureCode -Failures $failures -Code "redirect_policy_not_forbid"
    }
    if ($timeoutSeconds -lt 1) {
        Add-Sprint8AHealthFailureCode -Failures $failures -Code "invalid_timeout"
    }

    $receivedValue = Get-Sprint8AHealthMemberValue -InputObject $response -Name "received" -Required
    if ($receivedValue -isnot [bool]) {
        throw (New-Sprint8AHealthContractException -Message "Sprint 8A health response 'received' must be Boolean.")
    }
    $received = [bool]$receivedValue
    if (-not $received) {
        Add-Sprint8AHealthFailureCode -Failures $failures -Code "transport_error"
    } else {
        $responseUri = Get-Sprint8AHealthMemberValue -InputObject $response -Name "uri"
        if ([string]::IsNullOrWhiteSpace([string]$responseUri)) {
            Add-Sprint8AHealthFailureCode -Failures $failures -Code "response_uri_missing"
        } elseif (-not [string]::Equals($requestUri, [string]$responseUri, [StringComparison]::OrdinalIgnoreCase)) {
            Add-Sprint8AHealthFailureCode -Failures $failures -Code "response_uri_mismatch"
        }

        $redirectsFollowed = Get-Sprint8AHealthMemberValue -InputObject $response -Name "redirects_followed"
        if ($null -eq $redirectsFollowed) {
            Add-Sprint8AHealthFailureCode -Failures $failures -Code "redirect_accounting_missing"
        } elseif ([int]$redirectsFollowed -ne 0) {
            Add-Sprint8AHealthFailureCode -Failures $failures -Code "redirect_followed"
        }

        $statusValue = Get-Sprint8AHealthMemberValue -InputObject $response -Name "status"
        if ($null -eq $statusValue) {
            Add-Sprint8AHealthFailureCode -Failures $failures -Code "unexpected_status"
        } else {
            $status = [int]$statusValue
            if ($status -ge 300 -and $status -le 399) {
                Add-Sprint8AHealthFailureCode -Failures $failures -Code "redirect_response"
            }
            if ($status -ne [int]$expectation.status) {
                Add-Sprint8AHealthFailureCode -Failures $failures -Code "unexpected_status"
            }
        }

        $mediaType = [string](Get-Sprint8AHealthMemberValue -InputObject $response -Name "media_type")
        $charset = [string](Get-Sprint8AHealthMemberValue -InputObject $response -Name "charset")
        if ([string]$expectation.endpoint_kind -ceq "core") {
            if (-not [string]::Equals($mediaType, [string]$expectation.media_type, [StringComparison]::OrdinalIgnoreCase)) {
                Add-Sprint8AHealthFailureCode -Failures $failures -Code "unexpected_media_type"
            }
            if (-not [string]::IsNullOrWhiteSpace($charset) -and $charset -cne "utf-8") {
                Add-Sprint8AHealthFailureCode -Failures $failures -Code "unexpected_charset"
            }
        }

        $bodyLength = Get-Sprint8AHealthMemberValue -InputObject $response -Name "body_utf8_length"
        $bodySha256 = [string](Get-Sprint8AHealthMemberValue -InputObject $response -Name "body_sha256")
        if ($null -eq $bodyLength -or [long]$bodyLength -ne [long]$expectation.body_utf8_length -or
            $bodySha256 -cne [string]$expectation.body_sha256) {
            Add-Sprint8AHealthFailureCode -Failures $failures -Code "unexpected_body"
        }
    }

    $codes = @($failures)
    $passed = $codes.Count -eq 0
    $failure = if ($passed) {
        $null
    } else {
        $exceptionType = Get-Sprint8AHealthMemberValue -InputObject $transportError -Name "exception_type"
        $message = Get-Sprint8AHealthMemberValue -InputObject $transportError -Name "sanitized_message"
        [ordered]@{
            kind = if (-not $received) {
                "transport"
            } elseif ($null -ne $transportError) {
                "evidence_capture"
            } else {
                "contract"
            }
            codes = $codes
            exception_type = if ($null -eq $exceptionType) { $null } else { [string]$exceptionType }
            sanitized_message = if ($null -eq $message) { $null } else { [string]$message }
        }
    }

    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.health-observation"
        target = $target
        expectation = [ordered]@{
            endpoint_kind = [string]$expectation.endpoint_kind
            method = [string]$expectation.method
            path = [string]$expectation.path
            status = [int]$expectation.status
            media_type = $expectation.media_type
            allowed_charsets = @($expectation.allowed_charsets)
            body_utf8_length = [long]$expectation.body_utf8_length
            body_sha256 = [string]$expectation.body_sha256
            redirects_allowed = $false
        }
        request = [ordered]@{
            method = $method
            uri = $requestUri
            path = $requestPath
            timeout_seconds = $timeoutSeconds
            redirect_policy = $redirectPolicy
        }
        response = [ordered]@{
            received = $received
            uri = Get-Sprint8AHealthMemberValue -InputObject $response -Name "uri"
            status = Get-Sprint8AHealthMemberValue -InputObject $response -Name "status"
            content_type = Get-Sprint8AHealthMemberValue -InputObject $response -Name "content_type"
            media_type = Get-Sprint8AHealthMemberValue -InputObject $response -Name "media_type"
            charset = Get-Sprint8AHealthMemberValue -InputObject $response -Name "charset"
            location = Get-Sprint8AHealthMemberValue -InputObject $response -Name "location"
            body_utf8_length = Get-Sprint8AHealthMemberValue -InputObject $response -Name "body_utf8_length"
            body_sha256 = Get-Sprint8AHealthMemberValue -InputObject $response -Name "body_sha256"
            redirects_followed = Get-Sprint8AHealthMemberValue -InputObject $response -Name "redirects_followed"
        }
        failure = $failure
        passed = $passed
    }
}

function Resolve-Sprint8AHealthUri {
    param(
        [Parameter(Mandatory)][string]$BaseUrl,
        [Parameter(Mandatory)][string]$Path
    )

    $baseUri = $null
    if (-not [Uri]::TryCreate($BaseUrl, [UriKind]::Absolute, [ref]$baseUri) -or
        $baseUri.Scheme -notin @([Uri]::UriSchemeHttp, [Uri]::UriSchemeHttps) -or
        -not [string]::IsNullOrEmpty($baseUri.UserInfo) -or
        -not [string]::IsNullOrEmpty($baseUri.Query) -or
        -not [string]::IsNullOrEmpty($baseUri.Fragment) -or
        $baseUri.AbsolutePath -cne "/") {
        throw (New-Sprint8AHealthContractException -Message "Sprint 8A health BaseUrl must be an absolute HTTP(S) root with no credentials, query, or fragment.")
    }
    [Uri]::new($baseUri, $Path)
}

function Invoke-Sprint8AHealthProbe {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [ValidateSet("gateway_core", "core_control", "supervisor")]
        [string]$Target,
        [Parameter(Mandatory)][string]$BaseUrl,
        [ValidateRange(1, 300)][int]$TimeoutSeconds = 5
    )

    $expectation = Get-Sprint8AHealthContract -Target $Target
    $uri = Resolve-Sprint8AHealthUri -BaseUrl $BaseUrl -Path ([string]$expectation.path)
    $request = [ordered]@{
        method = "GET"
        uri = $uri.AbsoluteUri
        path = [string]$expectation.path
        timeout_seconds = $TimeoutSeconds
        redirect_policy = "forbid"
    }
    $responseEvidence = [ordered]@{
        received = $false
        uri = $null
        status = $null
        content_type = $null
        media_type = $null
        charset = $null
        location = $null
        body_utf8_length = $null
        body_sha256 = $null
        redirects_followed = $null
    }
    $transportError = $null
    $handler = [Net.Http.HttpClientHandler]::new()
    $handler.AllowAutoRedirect = $false
    $client = [Net.Http.HttpClient]::new($handler)
    $client.Timeout = [TimeSpan]::FromSeconds($TimeoutSeconds)
    try {
        $httpResponse = $client.GetAsync($uri).GetAwaiter().GetResult()
        try {
            $contentTypeHeader = if ($null -eq $httpResponse.Content) { $null } else { $httpResponse.Content.Headers.ContentType }
            $responseEvidence.received = $true
            $responseEvidence.uri = [string]$httpResponse.RequestMessage.RequestUri.AbsoluteUri
            $responseEvidence.status = [int]$httpResponse.StatusCode
            $responseEvidence.content_type = if ($null -eq $contentTypeHeader) { $null } else { [string]$contentTypeHeader }
            $responseEvidence.media_type = if ($null -eq $contentTypeHeader -or [string]::IsNullOrWhiteSpace([string]$contentTypeHeader.MediaType)) {
                $null
            } else {
                ([string]$contentTypeHeader.MediaType).ToLowerInvariant()
            }
            $responseEvidence.charset = if ($null -eq $contentTypeHeader -or [string]::IsNullOrWhiteSpace([string]$contentTypeHeader.CharSet)) {
                $null
            } else {
                ([string]$contentTypeHeader.CharSet).Trim('"').ToLowerInvariant()
            }
            $responseEvidence.location = if ($null -eq $httpResponse.Headers.Location) { $null } else { [string]$httpResponse.Headers.Location }
            $responseEvidence.redirects_followed = 0
            [byte[]]$bytes = [byte[]]::new(0)
            if ($null -ne $httpResponse.Content) {
                $bytes = $httpResponse.Content.ReadAsByteArrayAsync().GetAwaiter().GetResult()
            }
            $responseEvidence.body_utf8_length = [long]$bytes.LongLength
            $responseEvidence.body_sha256 = Get-Sprint8AHealthBytesSha256 -Bytes $bytes
        } finally {
            $httpResponse.Dispose()
        }
    } catch {
        $message = ([string]$_.Exception.Message -replace '[\r\n]+', ' ').Trim()
        if ($message.Length -gt 500) {
            $message = $message.Substring(0, 500)
        }
        $transportError = [ordered]@{
            exception_type = $_.Exception.GetType().FullName
            sanitized_message = $message
        }
    } finally {
        $client.Dispose()
    }

    Test-Sprint8AHealthObservation -Observation ([ordered]@{
        target = $Target
        request = $request
        response = $responseEvidence
        transport_error = $transportError
    })
}

function Wait-Sprint8AHealthProbe {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [ValidateSet("gateway_core", "core_control", "supervisor")]
        [string]$Target,
        [Parameter(Mandatory)][string]$BaseUrl,
        [ValidateRange(1, 300)][int]$TimeoutSeconds = 5,
        [ValidateRange(1, 600)][int]$MaximumAttempts = 60,
        [ValidateRange(0, 60)][int]$DelaySeconds = 1
    )

    $observation = $null
    for ($attempt = 1; $attempt -le $MaximumAttempts; $attempt++) {
        $observation = Invoke-Sprint8AHealthProbe `
            -Target $Target `
            -BaseUrl $BaseUrl `
            -TimeoutSeconds $TimeoutSeconds
        if ([bool]$observation.passed) {
            return [pscustomobject][ordered]@{
                schema_version = 1
                contract = "tessara.sprint-8a.health-wait"
                target = $Target
                attempts = $attempt
                maximum_attempts = $MaximumAttempts
                delay_seconds = $DelaySeconds
                observation = $observation
                passed = $true
            }
        }
        if ($attempt -lt $MaximumAttempts -and $DelaySeconds -gt 0) {
            Start-Sleep -Seconds $DelaySeconds
        }
    }
    [pscustomobject][ordered]@{
        schema_version = 1
        contract = "tessara.sprint-8a.health-wait"
        target = $Target
        attempts = $MaximumAttempts
        maximum_attempts = $MaximumAttempts
        delay_seconds = $DelaySeconds
        observation = $observation
        passed = $false
    }
}

function Assert-Sprint8AHealthPassed {
    param(
        [Parameter(Mandatory)]$Observation,
        [string]$Context = "Sprint 8A health probe"
    )

    if ([bool](Get-Sprint8AHealthMemberValue -InputObject $Observation -Name "passed" -Required)) {
        return
    }
    $failure = Get-Sprint8AHealthMemberValue -InputObject $Observation -Name "failure" -Required
    $codes = @((Get-Sprint8AHealthMemberValue -InputObject $failure -Name "codes" -Required) | ForEach-Object { [string]$_ })
    $kind = [string](Get-Sprint8AHealthMemberValue -InputObject $failure -Name "kind" -Required)
    $transportMessage = [string](Get-Sprint8AHealthMemberValue -InputObject $failure -Name "sanitized_message")
    $suffix = if ([string]::IsNullOrWhiteSpace($transportMessage)) { "" } else { " ($transportMessage)" }
    $exception = [InvalidOperationException]::new("$Context failed: $($codes -join ', ').$suffix")
    $exception.Data["TessaraFailureClassification"] = switch ($kind) {
        "transport" { "environment" }
        "evidence_capture" { "harness" }
        default { "product" }
    }
    throw $exception
}

function Test-Sprint8AHealthContract {
    $coreHash = "2689367b205c16ce32ed4200942b8b8b1e262dfc70d9bc9fbc77c49699a4f1df"
    $emptyHash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    $validCore = Test-Sprint8AHealthObservation -Observation ([ordered]@{
        target = "gateway_core"
        request = [ordered]@{
            method = "GET"; uri = "http://127.0.0.1:8088/health"; path = "/health"
            timeout_seconds = 5; redirect_policy = "forbid"
        }
        response = [ordered]@{
            received = $true; uri = "http://127.0.0.1:8088/health"; status = 200
            content_type = "text/plain; charset=utf-8"; media_type = "text/plain"; charset = "utf-8"
            location = $null; body_utf8_length = 2; body_sha256 = $coreHash; redirects_followed = 0
        }
    })
    $validSupervisor = Test-Sprint8AHealthObservation -Observation ([ordered]@{
        target = "supervisor"
        request = [ordered]@{
            method = "GET"; uri = "http://127.0.0.1:8098/health/ready"; path = "/health/ready"
            timeout_seconds = 5; redirect_policy = "forbid"
        }
        response = [ordered]@{
            received = $true; uri = "http://127.0.0.1:8098/health/ready"; status = 204
            content_type = $null; media_type = $null; charset = $null; location = $null
            body_utf8_length = 0; body_sha256 = $emptyHash; redirects_followed = 0
        }
    })
    if (-not [bool]$validCore.passed -or -not [bool]$validSupervisor.passed) {
        throw "Sprint 8A health self-test rejected a canonical Core or Supervisor response."
    }
    [byte[]]$emptyBody = [byte[]]::new(0)
    if ($emptyBody.LongLength -ne 0 -or
        (Get-Sprint8AHealthBytesSha256 -Bytes $emptyBody) -cne $emptyHash) {
        throw "Sprint 8A health self-test could not preserve an exact empty response body."
    }

    $redirect = Test-Sprint8AHealthObservation -Observation ([ordered]@{
        target = "gateway_core"
        request = $validCore.request
        response = [ordered]@{
            received = $true; uri = "http://127.0.0.1:8088/health"; status = 303
            content_type = "text/html; charset=utf-8"; media_type = "text/html"; charset = "utf-8"
            location = "/login"; body_utf8_length = 0; body_sha256 = $emptyHash; redirects_followed = 0
        }
    })
    $wrongSupervisorStatus = Test-Sprint8AHealthObservation -Observation ([ordered]@{
        target = "supervisor"
        request = $validSupervisor.request
        response = [ordered]@{
            received = $true; uri = "http://127.0.0.1:8098/health/ready"; status = 200
            content_type = $null; media_type = $null; charset = $null; location = $null
            body_utf8_length = 0; body_sha256 = $emptyHash; redirects_followed = 0
        }
    })
    if ([bool]$redirect.passed -or @($redirect.failure.codes) -cnotcontains "redirect_response" -or
        @($redirect.failure.codes) -cnotcontains "unexpected_status" -or
        [bool]$wrongSupervisorStatus.passed -or @($wrongSupervisorStatus.failure.codes) -cnotcontains "unexpected_status") {
        throw "Sprint 8A health self-test accepted a redirect or the wrong Supervisor status."
    }
    $captureFailure = Test-Sprint8AHealthObservation -Observation ([ordered]@{
        target = "supervisor"
        request = $validSupervisor.request
        response = [ordered]@{
            received = $true; uri = "http://127.0.0.1:8098/health/ready"; status = 204
            content_type = $null; media_type = $null; charset = $null; location = $null
            body_utf8_length = $null; body_sha256 = $null; redirects_followed = 0
        }
        transport_error = [ordered]@{
            exception_type = "System.Management.Automation.PropertyNotFoundException"
            sanitized_message = "synthetic response evidence capture failure"
        }
    })
    if ([bool]$captureFailure.passed -or [string]$captureFailure.failure.kind -cne "evidence_capture") {
        throw "Sprint 8A health self-test did not distinguish response evidence capture from a product contract failure."
    }
    try {
        Assert-Sprint8AHealthPassed -Observation $captureFailure -Context "synthetic health capture"
        throw "Sprint 8A health self-test accepted incomplete response evidence."
    } catch {
        if ([string]$_.Exception.Data["TessaraFailureClassification"] -cne "harness") {
            throw "Sprint 8A health self-test did not classify response evidence capture as harness."
        }
    }

    $fixturePath = Join-Path $PSScriptRoot "fixtures/sprint-8a-health-contract-regressions.json"
    if (-not (Test-Path -LiteralPath $fixturePath -PathType Leaf)) {
        throw "Sprint 8A health regression fixture is missing."
    }
    $fixture = Get-Content -LiteralPath $fixturePath -Raw | ConvertFrom-Json
    $expectedReceiptHashes = @(
        "e3aaca8880875a5ff49afada3c7d2beaecbb0fccae9bbd0af7f6a065a7e6aa40",
        "2c08914fba0839635289a3fabee90420d8685c5ae9e213891ccb9d9d6119c886"
    )
    if ([int]$fixture.schema_version -ne 1 -or
        [string]$fixture.fixture_kind -cne "tessara.sprint-8a.health-contract-regressions" -or
        [string]$fixture.authority -cne "diagnostic_history_only" -or
        [int]$fixture.source_attempt -ne 32 -or
        @($fixture.cases).Count -ne 2 -or
        ((@($fixture.cases.source_receipt.sha256) | Sort-Object) -join "`n") -cne (($expectedReceiptHashes | Sort-Object) -join "`n")) {
        throw "Sprint 8A health regression fixture identity drifted from retained Rehearsal 32 evidence."
    }
    foreach ($case in @($fixture.cases)) {
        if ([string]$case.observation.response.content_type -cne "text/html; charset=utf-8" -or
            [long]$case.observation.response.body_utf8_length -ne 4710 -or
            [string]$case.observation.response.body_sha256 -cne "f00cefaa866bef8a84a0dff0ecaa73c6437ab295e961c20779d5b31a9cc4a574") {
            throw "Sprint 8A health regression fixture '$($case.name)' changed the retained HTML-response metadata."
        }
        $evaluated = Test-Sprint8AHealthObservation -Observation $case.observation
        if ([bool]$evaluated.passed) {
            throw "Sprint 8A health regression fixture '$($case.name)' was incorrectly accepted."
        }
        foreach ($requiredCode in @($case.required_failure_codes)) {
            if (@($evaluated.failure.codes) -cnotcontains [string]$requiredCode) {
                throw "Sprint 8A health regression fixture '$($case.name)' omitted expected rejection '$requiredCode'."
            }
        }
    }

    $invokeDefinition = (Get-Command Invoke-Sprint8AHealthProbe).Definition
    if (-not $invokeDefinition.Contains('$handler.AllowAutoRedirect = $false')) {
        throw "Sprint 8A health transport no longer explicitly disables redirects."
    }
    Write-Host "Sprint 8A exact no-redirect health contract self-test passed."
    [pscustomobject][ordered]@{
        contract = "tessara.sprint-8a.health-contract-self-test"
        passed = $true
    }
}

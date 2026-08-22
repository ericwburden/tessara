[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$containerName = "tessara-s8b-response-contract-$([guid]::NewGuid().ToString('N').Substring(0, 12))"
$databaseName = "tessara_s8b_response_contract_test"
$firstMutationJob = $null
. (Join-Path $PSScriptRoot "sprint-8b-cargo-test-integrity.ps1")

try {
    $containerId = docker run --detach --rm --name $containerName `
        -e POSTGRES_PASSWORD=tessara `
        -e POSTGRES_DB=$databaseName `
        --publish 127.0.0.1::5432 `
        postgres:16-alpine
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($containerId)) {
        throw "Unable to start isolated Response export contract database."
    }
    $ready = $false
    $stableReadyChecks = 0
    for ($attempt = 0; $attempt -lt 120; $attempt++) {
        $readyOutput = docker exec $containerName psql -At -U postgres -d $databaseName -c "SELECT 1;" 2>$null
        if ($LASTEXITCODE -eq 0 -and ([string]$readyOutput).Trim() -eq "1") {
            $stableReadyChecks++
            if ($stableReadyChecks -ge 3) { $ready = $true; break }
        }
        else {
            $stableReadyChecks = 0
        }
        Start-Sleep -Milliseconds 250
    }
    if (-not $ready) { throw "Isolated Response export contract database did not become ready." }
    $portOutput = docker port $containerName 5432/tcp
    if ($LASTEXITCODE -ne 0 -or ([string]$portOutput).Trim() -notmatch ':(?<port>[0-9]+)$') {
        throw "Unable to resolve the isolated Response export contract database port."
    }
    $databasePort = $Matches.port

    $migrationOutput = Get-Content -Raw -LiteralPath (Join-Path $repoRoot "crates/tessara-api/migrations/001_baseline.sql") |
        docker exec -i $containerName psql -v ON_ERROR_STOP=1 -U postgres -d $databaseName 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Core baseline migration failed in the isolated contract database: $($migrationOutput -join [Environment]::NewLine)"
    }

    $assertions = @'
BEGIN;
SELECT append_response_export_change(
  '00000000-0000-0000-0000-000000000001',
  '00000000-0000-0000-0000-000000000002',
  '00000000-0000-0000-0000-000000000003',
  'upsert',
  '{"response_id":"00000000-0000-0000-0000-000000000001"}'::jsonb
);
ROLLBACK;
DO $$
BEGIN
  IF (SELECT next_sequence FROM response_export_state WHERE singleton) <> 0 THEN
    RAISE EXCEPTION 'rolled-back mutation advanced the committed source cursor';
  END IF;
END $$;
SELECT append_response_export_change(
  '00000000-0000-0000-0000-000000000001',
  '00000000-0000-0000-0000-000000000002',
  '00000000-0000-0000-0000-000000000003',
  'upsert',
  '{"response_id":"00000000-0000-0000-0000-000000000001"}'::jsonb
);
SELECT append_response_export_change(
  '00000000-0000-0000-0000-000000000001',
  '00000000-0000-0000-0000-000000000002',
  '00000000-0000-0000-0000-000000000003',
  'tombstone',
  '{"response_id":"00000000-0000-0000-0000-000000000001","reason":"deleted"}'::jsonb
);
DO $$
BEGIN
  IF (SELECT array_agg(change_sequence ORDER BY change_sequence) FROM response_export_changes)
       <> ARRAY[1::bigint,2::bigint] THEN
    RAISE EXCEPTION 'Response export sequence is not gap-free after rollback and commit';
  END IF;
  IF EXISTS (
      SELECT 1 FROM response_export_changes
       WHERE content_digest <> 'sha256:' || encode(
           digest(convert_to(payload::text, 'UTF8'), 'sha256'), 'hex'
       )
  ) THEN
    RAISE EXCEPTION 'Response export payload digest does not bind exact jsonb::text storage bytes';
  END IF;
  IF (SELECT array_agg(change_kind ORDER BY change_sequence) FROM response_export_changes)
       <> ARRAY['upsert'::text,'tombstone'::text] THEN
    RAISE EXCEPTION 'Response export upsert/tombstone order is incorrect';
  END IF;
END $$;

-- Build one minimal owner graph and exercise every persisted Response shape
-- through the tables guarded by the Response export triggers.
INSERT INTO accounts (id,email,display_name)
VALUES ('10000000-0000-0000-0000-000000000001','export-owner@example.test','Export Owner');
INSERT INTO node_types (id,name,slug)
VALUES ('10000000-0000-0000-0000-000000000010','Export Node Type','export-node-type');
INSERT INTO nodes (id,node_type_id,name)
VALUES ('10000000-0000-0000-0000-000000000011','10000000-0000-0000-0000-000000000010','Export Node');
INSERT INTO forms (id,name,slug,scope_node_type_id)
VALUES ('10000000-0000-0000-0000-000000000020','Export Form','export-form','10000000-0000-0000-0000-000000000010');
INSERT INTO form_versions (id,form_id,status,version_label)
VALUES ('10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000020','published','1.0.0');
INSERT INTO form_sections (id,form_version_id,title)
VALUES ('10000000-0000-0000-0000-000000000022','10000000-0000-0000-0000-000000000021','Export Section');
INSERT INTO form_fields
  (field_id,form_version_id,section_id,key,label,field_type,position)
VALUES
  ('10000000-0000-0000-0000-000000000023','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000022','answer','Answer','text',0),
  ('10000000-0000-0000-0000-000000000024','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000022','labels','Labels','multi_choice',1),
  ('10000000-0000-0000-0000-000000000025','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000022','optional','Optional','text',2);
INSERT INTO workflows (id,workflow_node_type_id,name,slug)
VALUES ('10000000-0000-0000-0000-000000000030','10000000-0000-0000-0000-000000000010','Export Workflow','export-workflow');
INSERT INTO workflow_versions (id,workflow_id,status,version_label)
VALUES ('10000000-0000-0000-0000-000000000031','10000000-0000-0000-0000-000000000030','published','1.0.0');
INSERT INTO workflow_steps (id,workflow_version_id,form_version_id,title,position)
VALUES ('10000000-0000-0000-0000-000000000032','10000000-0000-0000-0000-000000000031','10000000-0000-0000-0000-000000000021','Export Step',0);
INSERT INTO workflow_assignments
  (id,workflow_version_id,workflow_step_id,node_id,account_id)
VALUES
  ('10000000-0000-0000-0000-000000000033','10000000-0000-0000-0000-000000000031','10000000-0000-0000-0000-000000000032','10000000-0000-0000-0000-000000000011','10000000-0000-0000-0000-000000000001');
INSERT INTO submissions
  (id,form_version_id,node_id,workflow_assignment_id,status,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000040','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000011','10000000-0000-0000-0000-000000000033','draft','2026-08-13T12:00:00Z');
INSERT INTO submission_values (submission_id,form_version_id,field_id,value)
VALUES
  ('10000000-0000-0000-0000-000000000040','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000023','"initial"'::jsonb),
  ('10000000-0000-0000-0000-000000000040','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000024','["a","b"]'::jsonb),
  ('10000000-0000-0000-0000-000000000040','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000025','null'::jsonb);
UPDATE submissions
   SET status='submitted', submitted_at='2026-08-13T12:05:00Z'
 WHERE id='10000000-0000-0000-0000-000000000040';
INSERT INTO submission_audit_events
  (id,submission_id,event_type,account_id,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000041','10000000-0000-0000-0000-000000000040','submit','10000000-0000-0000-0000-000000000001','2026-08-13T12:06:00Z');
DO $$
DECLARE latest jsonb;
BEGIN
  SELECT payload INTO latest
    FROM response_export_changes
   WHERE response_id='10000000-0000-0000-0000-000000000040'
   ORDER BY change_sequence DESC LIMIT 1;
  IF latest->>'node_name' <> 'Export Node'
     OR latest->>'restriction_tier' <> 'public'
     OR (latest->>'form_id')::uuid <> '10000000-0000-0000-0000-000000000020'
     OR (latest->>'last_modified_at')::timestamptz <> '2026-08-13T12:06:00Z'::timestamptz
     OR latest->>'last_modified_by_user_name' <> 'Export Owner'
     OR latest#>>'{values,answer,field_id}' <> '10000000-0000-0000-0000-000000000023'
     OR latest#>>'{values,answer,value_text}' <> 'initial'
     OR latest#>>'{values,labels,value_text}' <> '["a", "b"]'
     OR latest#>>'{values,optional,value_text}' <> 'null' THEN
    RAISE EXCEPTION 'submitted aggregate omitted or altered an authoring/materialization source fact: %', latest;
  END IF;
END $$;

UPDATE submission_values
   SET value='"corrected"'::jsonb
 WHERE submission_id='10000000-0000-0000-0000-000000000040'
   AND field_id='10000000-0000-0000-0000-000000000023';
INSERT INTO submission_audit_events
  (id,submission_id,event_type,account_id,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000042','10000000-0000-0000-0000-000000000040','correct','10000000-0000-0000-0000-000000000001','2026-08-13T12:10:00Z');
DO $$
DECLARE latest jsonb;
BEGIN
  SELECT payload INTO latest FROM response_export_changes
   WHERE response_id='10000000-0000-0000-0000-000000000040'
   ORDER BY change_sequence DESC LIMIT 1;
  IF latest#>>'{values,answer,value_text}' <> 'corrected'
     OR (latest->>'last_modified_at')::timestamptz <> '2026-08-13T12:10:00Z'::timestamptz THEN
    RAISE EXCEPTION 'audit-after-correction did not publish the final aggregate: %', latest;
  END IF;
END $$;

DELETE FROM submission_values
 WHERE submission_id='10000000-0000-0000-0000-000000000040'
   AND field_id='10000000-0000-0000-0000-000000000024';
INSERT INTO submission_audit_events
  (id,submission_id,event_type,account_id,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000043','10000000-0000-0000-0000-000000000040','delete_value','10000000-0000-0000-0000-000000000001','2026-08-13T12:11:00Z');
DO $$
DECLARE latest jsonb;
BEGIN
  SELECT payload INTO latest FROM response_export_changes
   WHERE response_id='10000000-0000-0000-0000-000000000040'
   ORDER BY change_sequence DESC LIMIT 1;
  IF (latest->'values') ? 'labels'
     OR (latest->>'last_modified_at')::timestamptz <> '2026-08-13T12:11:00Z'::timestamptz THEN
    RAISE EXCEPTION 'value deletion was not represented by the final aggregate: %', latest;
  END IF;
END $$;

UPDATE submissions SET status='draft', submitted_at=NULL
 WHERE id='10000000-0000-0000-0000-000000000040';
DO $$
DECLARE latest response_export_changes%ROWTYPE;
BEGIN
  SELECT * INTO latest FROM response_export_changes
   WHERE response_id='10000000-0000-0000-0000-000000000040'
   ORDER BY change_sequence DESC LIMIT 1;
  IF latest.change_kind <> 'tombstone' OR latest.payload->>'reason' <> 'status_excluded' THEN
    RAISE EXCEPTION 'submitted-to-draft transition did not emit status tombstone: %', latest.payload;
  END IF;
END $$;

UPDATE submissions SET status='submitted', submitted_at='2026-08-13T12:12:00Z'
 WHERE id='10000000-0000-0000-0000-000000000040';
INSERT INTO submission_audit_events
  (id,submission_id,event_type,account_id,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000044','10000000-0000-0000-0000-000000000040','resubmit','10000000-0000-0000-0000-000000000001','2026-08-13T12:12:01Z');
DO $$
DECLARE latest response_export_changes%ROWTYPE;
BEGIN
  SELECT * INTO latest FROM response_export_changes
   WHERE response_id='10000000-0000-0000-0000-000000000040'
   ORDER BY change_sequence DESC LIMIT 1;
  IF latest.change_kind <> 'upsert'
     OR latest.payload#>>'{values,answer,value_text}' <> 'corrected'
     OR (latest.payload->>'last_modified_at')::timestamptz <> '2026-08-13T12:12:01Z'::timestamptz THEN
    RAISE EXCEPTION 'status re-entry did not emit the current complete aggregate: %', latest.payload;
  END IF;
END $$;

SELECT append_response_export_change(
  '10000000-0000-0000-0000-000000000040',
  '10000000-0000-0000-0000-000000000021',
  '10000000-0000-0000-0000-000000000011',
  'tombstone',
  '{"response_id":"10000000-0000-0000-0000-000000000040","reason":"redacted"}'::jsonb
);
DO $$
BEGIN
  IF (SELECT payload->>'reason' FROM response_export_changes
       WHERE response_id='10000000-0000-0000-0000-000000000040'
       ORDER BY change_sequence DESC LIMIT 1) <> 'redacted' THEN
    RAISE EXCEPTION 'explicit owner redaction did not emit its tombstone';
  END IF;
END $$;

DELETE FROM submissions WHERE id='10000000-0000-0000-0000-000000000040';
DO $$
DECLARE latest response_export_changes%ROWTYPE;
BEGIN
  SELECT * INTO latest FROM response_export_changes
   WHERE response_id='10000000-0000-0000-0000-000000000040'
   ORDER BY change_sequence DESC LIMIT 1;
  IF latest.change_kind <> 'tombstone' OR latest.payload->>'reason' <> 'deleted' THEN
    RAISE EXCEPTION 'delete and cascading child triggers did not finish on the delete tombstone: %', latest.payload;
  END IF;
  IF (SELECT next_sequence FROM response_export_state WHERE singleton)
       <> (SELECT count(*) FROM response_export_changes) THEN
    RAISE EXCEPTION 'committed Response export sequence contains a gap';
  END IF;
END $$;

INSERT INTO submissions
  (id,form_version_id,node_id,workflow_assignment_id,status,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000050','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000011','10000000-0000-0000-0000-000000000033','draft','2026-08-13T13:00:00Z');
INSERT INTO submission_values (submission_id,form_version_id,field_id,value)
VALUES
  ('10000000-0000-0000-0000-000000000050','10000000-0000-0000-0000-000000000021','10000000-0000-0000-0000-000000000023','"before-race"'::jsonb);
UPDATE submissions SET status='submitted', submitted_at='2026-08-13T13:01:00Z'
 WHERE id='10000000-0000-0000-0000-000000000050';
INSERT INTO submission_audit_events
  (id,submission_id,event_type,account_id,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000051','10000000-0000-0000-0000-000000000050','submit','10000000-0000-0000-0000-000000000001','2026-08-13T13:02:00Z');
'@
    $assertionOutput = $assertions | docker exec -i $containerName psql -v ON_ERROR_STOP=1 -U postgres -d $databaseName 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Response export transactional assertions failed: $($assertionOutput -join [Environment]::NewLine)"
    }

    $sequenceBeforeConcurrencyOutput = docker exec $containerName psql -At -U postgres -d $databaseName `
        -c "SELECT next_sequence FROM response_export_state WHERE singleton;"
    if ($LASTEXITCODE -ne 0) { throw "Could not read the pre-concurrency Response cursor." }
    $sequenceBeforeConcurrency = [long]([string]$sequenceBeforeConcurrencyOutput).Trim()
    $firstMutation = @'
BEGIN;
UPDATE submission_values
   SET value='"after-race"'::jsonb
 WHERE submission_id='10000000-0000-0000-0000-000000000050'
   AND field_id='10000000-0000-0000-0000-000000000023';
SELECT pg_sleep(4);
COMMIT;
'@
    $firstMutationJob = Start-Job -ScriptBlock {
        param($ContainerName, $DatabaseName, $Sql)
        $output = $Sql | docker exec -i -e PGAPPNAME=response-export-value-mutation `
            $ContainerName psql -v ON_ERROR_STOP=1 -U postgres -d $DatabaseName 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Concurrent value mutation failed: $($output -join [Environment]::NewLine)"
        }
        $output
    } -ArgumentList $containerName, $databaseName, $firstMutation

    $firstMutationIsHoldingSequenceLock = $false
    for ($attempt = 0; $attempt -lt 40; $attempt++) {
        if ($firstMutationJob.State -eq "Failed") {
            Receive-Job -Job $firstMutationJob -ErrorAction Stop | Out-Null
        }
        $activity = docker exec $containerName psql -At -U postgres -d $databaseName `
            -c "SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE application_name='response-export-value-mutation' AND wait_event='PgSleep');"
        if ($LASTEXITCODE -ne 0) { throw "Could not observe the concurrent Response mutation." }
        if (([string]$activity).Trim() -eq "t") {
            $firstMutationIsHoldingSequenceLock = $true
            break
        }
        Start-Sleep -Milliseconds 100
    }
    if (-not $firstMutationIsHoldingSequenceLock) {
        throw "The first Response mutation did not reach its lock-holding checkpoint."
    }

    $secondMutation = @'
INSERT INTO submission_audit_events
  (id,submission_id,event_type,account_id,created_at)
VALUES
  ('10000000-0000-0000-0000-000000000052','10000000-0000-0000-0000-000000000050','concurrent_audit','10000000-0000-0000-0000-000000000001','2026-08-13T13:04:00Z');
'@
    $secondMutationOutput = $secondMutation | docker exec -i -e PGAPPNAME=response-export-audit-mutation `
        $containerName psql -v ON_ERROR_STOP=1 -U postgres -d $databaseName 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Concurrent audit mutation failed: $($secondMutationOutput -join [Environment]::NewLine)"
    }
    Wait-Job -Job $firstMutationJob -Timeout 15 | Out-Null
    if ($firstMutationJob.State -ne "Completed") {
        throw "The serialized Response value mutation did not complete."
    }
    Receive-Job -Job $firstMutationJob -ErrorAction Stop | Out-Null

    $concurrencyAssertions = @'
DO $$
DECLARE
  expected_sequences bigint[] := ARRAY[(__BEFORE__ + 1)::bigint,(__BEFORE__ + 2)::bigint];
  actual_sequences bigint[];
  projected_values text[];
  latest jsonb;
BEGIN
  SELECT array_agg(change_sequence ORDER BY change_sequence),
         array_agg(payload#>>'{values,answer,value_text}' ORDER BY change_sequence)
    INTO actual_sequences, projected_values
    FROM response_export_changes
   WHERE change_sequence > __BEFORE__;
  SELECT payload INTO latest
    FROM response_export_changes
   WHERE change_sequence > __BEFORE__
   ORDER BY change_sequence DESC LIMIT 1;
  IF actual_sequences <> expected_sequences THEN
    RAISE EXCEPTION 'simultaneous Response mutations did not allocate adjacent commit-ordered positions: %', actual_sequences;
  END IF;
  IF projected_values <> ARRAY['after-race'::text,'after-race'::text]
     OR (latest->>'last_modified_at')::timestamptz <> '2026-08-13T13:04:00Z'::timestamptz
     OR latest->>'last_modified_by_user_name' <> 'Export Owner' THEN
    RAISE EXCEPTION 'audit mutation published a stale aggregate after waiting for the value mutation: %, %', projected_values, latest;
  END IF;
  IF (SELECT next_sequence FROM response_export_state WHERE singleton)
       <> (SELECT count(*) FROM response_export_changes) THEN
    RAISE EXCEPTION 'concurrent committed Response export sequence contains a gap';
  END IF;
  IF EXISTS (
      SELECT 1 FROM response_export_changes
       WHERE content_digest <> 'sha256:' || encode(
           digest(convert_to(payload::text, 'UTF8'), 'sha256'), 'hex'
       )
  ) THEN
    RAISE EXCEPTION 'a trigger-produced Response export row lost exact raw-storage integrity';
  END IF;
END $$;
'@.Replace("__BEFORE__", $sequenceBeforeConcurrency.ToString([Globalization.CultureInfo]::InvariantCulture))
    $concurrencyAssertionOutput = $concurrencyAssertions | docker exec -i $containerName `
        psql -v ON_ERROR_STOP=1 -U postgres -d $databaseName 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Response export concurrency assertions failed: $($concurrencyAssertionOutput -join [Environment]::NewLine)"
    }

    $previousTestDatabaseUrl = [Environment]::GetEnvironmentVariable("TEST_API_DATABASE_URL", "Process")
    [Environment]::SetEnvironmentVariable(
        "TEST_API_DATABASE_URL",
        "postgres://postgres:tessara@127.0.0.1:$databasePort/$databaseName",
        "Process"
    )
    Push-Location $repoRoot
    try {
        Invoke-Sprint8BCheckedCargoTest -Arguments @(
            "test", "--locked", "--color", "never", "-p", "tessara-api",
            "--test", "response_owner_actions", "--", "--test-threads=1"
        ) -ExpectedExecutedTestCount 4 | Out-Null
    }
    finally {
        Pop-Location
        [Environment]::SetEnvironmentVariable(
            "TEST_API_DATABASE_URL",
            $previousTestDatabaseUrl,
            "Process"
        )
    }

    [pscustomobject]@{
        sprint = "sprint-8b"
        contract = "tessara.responses.submitted-response-export"
        contract_version = "1.0.0"
        migration = "crates/tessara-api/migrations/001_baseline.sql"
        assertions = @(
            "rollback_does_not_advance",
            "commit_order_is_monotonic",
            "complete_authoring_envelope",
            "audit_after_correction_is_final",
            "value_deletion_is_final",
            "status_exit_and_reentry",
            "redaction_and_delete_tombstones",
            "simultaneous_allocation_is_gap_free",
            "waiting_audit_cannot_publish_stale_values",
            "exact_raw_storage_digest",
            "canonical_contract_digest",
            "response_owner_atomic_replay_rollback_ordering",
            "response_owner_scope_reparent_serialization",
            "trigger_and_owner_tamper_rejection",
            "zero_test_guard"
        )
        status = "passed"
    } | ConvertTo-Json
} finally {
    if ($null -ne $firstMutationJob) {
        if ($firstMutationJob.State -eq "Running") { Stop-Job -Job $firstMutationJob }
        Remove-Job -Job $firstMutationJob -Force
    }
    docker stop $containerName *> $null
}

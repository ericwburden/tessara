Set-StrictMode -Version Latest

function Get-Sprint8CResponseOwnerTestInventory {
    [pscustomobject][ordered]@{
        package = "tessara-response-module"
        kind = "integration"
        binary = "owner_persistence"
        identities = @(
            "concurrent_different_save_input_with_one_key_applies_once_and_conflicts",
            "concurrent_identical_delete_is_one_apply_and_one_replay",
            "concurrent_identical_save_is_one_apply_and_one_replay",
            "concurrent_identical_start_is_one_apply_and_one_replay",
            "concurrent_identical_submit_is_one_apply_and_one_replay",
            "create_is_atomic_audited_evented_and_idempotent",
            "create_requires_the_exact_live_start_claim",
            "delete_authority_separates_respond_ownership_from_manage_scope",
            "expired_start_claim_cannot_create_a_response",
            "fresh_gateway_grant_replays_stable_request_and_preserves_initial_binding",
            "inaccessible_response_is_nondisclosing",
            "pinned_draft_saves_submits_and_exports_without_live_providers"
        )
    }
}

function Get-Sprint8CWorkflowEventTestInventory {
    @(
        [pscustomobject][ordered]@{
            label = "Core autonomous consumer"
            package = "tessara-api"
            test_binary = "lib"
            arguments = @(
                "test", "-p", "tessara-api", "--lib", "workflow_response_consumer::tests::",
                "--locked", "--offline", "--jobs", "1"
            )
            identities = @(
                "workflow_response_consumer::tests::autonomous_consumer_recovers_owner_start_save_submit_backlog_while_unready",
                "workflow_response_consumer::tests::background_consumer_cancels_cleanly_without_losing_the_durable_cursor",
                "workflow_response_consumer::tests::projection_revision_policy_is_stale_safe_gap_intolerant_and_reconciliation_aware"
            )
        },
        [pscustomobject][ordered]@{
            label = "Response owner outbox"
            package = "tessara-response-module"
            test_binary = "owner_persistence"
            arguments = @(
                "test", "-p", "tessara-response-module", "--test", "owner_persistence",
                "--locked", "--offline", "--jobs", "1",
                "pinned_draft_saves_submits_and_exports_without_live_providers"
            )
            identities = @("pinned_draft_saves_submits_and_exports_without_live_providers")
        },
        [pscustomobject][ordered]@{
            label = "Response consumer ACK"
            package = "tessara-response-module"
            test_binary = "lib"
            arguments = @(
                "test", "-p", "tessara-response-module", "--lib",
                "--locked", "--offline", "--jobs", "1",
                "event_provider::tests::workflow_consumer_checkpoint_ack_is_monotonic_and_rejects_unpublished_heads"
            )
            identities = @(
                "event_provider::tests::workflow_consumer_checkpoint_ack_is_monotonic_and_rejects_unpublished_heads"
            )
        }
    )
}

function Assert-Sprint8CUatPredicateInventory {
    $owner = Get-Sprint8CResponseOwnerTestInventory
    $workflow = @(Get-Sprint8CWorkflowEventTestInventory)
    $ownerIdentities = @($owner.identities | ForEach-Object { [string]$_ })
    $workflowIdentities = @($workflow | ForEach-Object { @($_.identities) })
    if ([string]$owner.package -cne "tessara-response-module" -or
        [string]$owner.binary -cne "owner_persistence" -or
        $ownerIdentities.Count -ne 12 -or
        @($ownerIdentities | Sort-Object -Unique).Count -ne $ownerIdentities.Count -or
        $workflow.Count -ne 3 -or
        @($workflow.label | Sort-Object -Unique).Count -ne $workflow.Count -or
        $workflowIdentities.Count -ne 5 -or
        @($workflowIdentities | Sort-Object -Unique).Count -ne $workflowIdentities.Count -or
        @($workflow | Where-Object {
            [string]::IsNullOrWhiteSpace([string]$_.package) -or
            [string]::IsNullOrWhiteSpace([string]$_.test_binary) -or
            @($_.arguments).Count -eq 0 -or
            @($_.identities).Count -eq 0
        }).Count -ne 0) {
        throw "Sprint 8C canonical UAT predicate inventory is invalid."
    }
}

Assert-Sprint8CUatPredicateInventory

# Sprint 8A disposable deployment profile

This profile materializes Core, Component, Dashboard, and other selected
owners from empty databases. It is intentionally destructive and must use the
exact Compose project `tessara-sprint-8a`; it does not upgrade or preserve a
retained application stack.

Render the topology without mutating it:

```powershell
docker compose -f .\deploy\sprint-8a\compose.yaml --profile reference config
```

Fresh-materialize only after confirming the target is disposable:

```powershell
.\scripts\materialize-sprint-8a.ps1 -AuthorizeDisposableReset -Confirm
```

Add `-VerifyNoOp` to repeat the resolved composition after the first healthy
apply. The command refuses any Compose project or named-volume namespace other
than `tessara-sprint-8a`, retains failure evidence under
`target/sprint-8a-bootstrap/reference`, and verifies Component owner read-back
before accepting the Dashboard seed. Because this is a pre-production rebuild,
the signed owner bootstraps create the complete canonical acceptance seed:
seven Component shells, eight ComponentVersions (including one
superseded/inactive predecessor with a published/active successor), and seven
Dashboard placements (including three independent lifecycle-action fixtures).
The legacy fixture helper is limited to account/security setup and does not
write Component or Dashboard product tables.

The checked-in signing keys are disposable local acceptance fixtures only.
Deployments must supply distinct keys through their secret store.

After a new Validation Readiness result authorizes Candidate Rehearsal, its
Component release lane uses the current wrapper interface:

```powershell
.\scripts\run-sprint-8a-component-upgrade.ps1 `
  -ComposeFile "deploy/sprint-8a/compose.yaml" `
  -BaseUrl "http://127.0.0.1:8088" `
  -BaselineTag "tessara-sprint-8a-components-rehearsal-baseline:latest" `
  -OutputPath "artifacts/sprint-8a-closeout/rehearsal/component-upgrade-rollback.json"
```

The wrapper compiles a distinct compatible Component `0.9.0` release from the
same clean source; it does not relabel the `1.0.0` candidate. It records the
baseline executable, immutable image, Manifest, commit, and tree identities,
then resolves exact Component-only Blueprint deltas and asks the out-of-process
Supervisor to apply them through the Compose deployment adapter. The sequence
establishes `0.9.0`, upgrades to `1.0.0`, rolls back to `0.9.0`, and restores
intended `1.0.0`. Each stage verifies release/image read-back and Component
data, instance, configuration, routes, and behavior while asserting that
unrelated image IDs, container IDs, restart counts, semantic data, navigation,
and availability remain unchanged.

Formal validation is currently exited. This command documents the checked-in
interface; it is not a passing rehearsal receipt and must not be run as a
substitute for the complete readiness and candidate-rehearsal gates.

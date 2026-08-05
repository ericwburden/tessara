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
before accepting the Dashboard seed.

The checked-in signing keys are disposable local acceptance fixtures only.
Deployments must supply distinct keys through their secret store.

Validation rehearses a Component-only immutable-image upgrade, rollback, and
current-release restoration with:

```powershell
$candidate = "tessara-sprint-8a-components@sha256:<digest>"
$baseline = .\scripts\build-sprint-8a-component-rehearsal-baseline.ps1 `
  -CurrentImage $candidate
.\scripts\verify-sprint-8a-component-upgrade.ps1 `
  -BaselineImage $baseline `
  -CandidateImage $candidate `
  -CurrentImage $candidate
```

The local baseline is a source-preserving derivative of the current compatible
Component image with a distinct OCI configuration digest. The runner rejects
identical baseline/candidate digests, verifies Component resource identity, and
asserts that unrelated service image IDs, container IDs, and restart counts
remain unchanged.

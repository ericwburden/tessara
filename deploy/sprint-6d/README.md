# Sprint 6D Deployment

This topology includes Sprint 6C and adds the non-product
`tessara.reference.module-sdk` process. The reference process owns the
`reference-module-state` volume and is reachable only through the service
network and Core's manifest-driven same-origin document proxy.

From a clean committed tree, set `TESSARA_SOURCE_COMMIT`,
`TESSARA_SOURCE_TREE`, and `TESSARA_SOURCE_DIRTY=false`, then run:

```powershell
docker compose -f deploy/sprint-6d/compose.yaml config
docker compose -f deploy/sprint-6d/compose.yaml up --build -d
.\scripts\bootstrap-sprint-6d-deployment.ps1
.\scripts\bootstrap-sprint-6d-deployment.ps1
```

The second bootstrap must report an exact no-op.

For the retained closeout run, publish the canonical evidence in this order:

```powershell
.\scripts\verify-module-sdk-boundaries.ps1 `
  -NativeEvidencePath artifacts/sprint-6d-closeout/package-boundaries-native.json `
  -WasmEvidencePath artifacts/sprint-6d-closeout/package-boundaries-wasm.json
.\scripts\verify-module-sdk-compatibility.ps1 `
  -EvidencePath artifacts/sprint-6d-closeout/compatibility-inventory.json
.\scripts\run-module-sdk-conformance.ps1 `
  -EvidencePath artifacts/sprint-6d-closeout/reference-conformance.json
.\scripts\capture-sprint-6d-closeout-evidence.ps1 -Mode Static
.\scripts\capture-sprint-6d-closeout-evidence.ps1 -Mode RuntimeResilience
```

After smoke, UAT, Playwright, the Scoped Records regression capture, and the
manual matrix are complete, run `capture-sprint-6d-closeout-evidence.ps1
-Mode Digests`. It rejects an incomplete 18-file closeout inventory and writes
the required SHA-256 sidecars.

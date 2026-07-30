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

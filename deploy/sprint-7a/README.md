# Sprint 7A deployment profile

This fresh profile layers Sprint 7A over the retained Sprint 6F Supervisor and
isolated-module baseline. It builds source-exact product images and configures the
per-instance Dashboard Ed25519 service identity used for catalog, metadata, and
placement rendering.

Validate the materialized topology with:

```powershell
docker compose -f .\deploy\sprint-7a\compose.yaml --profile reference config
```

The checked-in key pair is for disposable local acceptance only. Deployed
environments must supply both service-key variables through their secret store.
The disposable defaults expose the public gateway through `127.0.0.1:8086`
and Supervisor through `127.0.0.1:8096` so a retained Sprint 6F stack can
remain available. Gateway discovery is constrained to the Sprint 7A Compose
project so retained stacks cannot contribute colliding router identities.

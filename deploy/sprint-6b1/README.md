# Sprint 6B1 single-host topology

This Compose project is the supported Sprint 6B1 topology. Traefik is the only
published HTTP entrypoint and uses Docker discovery with
`exposedByDefault=false`. Core and Scoped Records share the browser origin but
do not share a database or database role.

The checked-in fixture is a curated Tessara development release. Its exact
digest fields support deterministic planning and readback but do not claim
cryptographic publisher verification. Verified third-party module distribution
and production lifecycle management are future platform work. The checked-in
password defaults are for isolated local acceptance only.

```powershell
docker compose -f deploy/sprint-6b1/compose.yaml up --build --wait
```

The deterministic contract path can be checked without Docker:

```powershell
.\scripts\test-sprint-6b1-contract.ps1
```

For an applied host, pass the local Core URL and `TESSARA_DEPLOY_TOKEN` as the
last two `tessara-deploy apply` arguments. The CLI publishes the sanitized
curated-release receipt only after its local plan-binding checks succeed.

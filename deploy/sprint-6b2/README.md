# Sprint 6B2 secure local deployment

This topology exposes only Traefik and routes every public request to Core.
Installation control and Scoped Records have private service-network identities
and separate deployment/module databases. Core cannot connect to either
database, and Scoped Records cannot connect to Core or installation-control
databases.

The defaults are deterministic development trust fixtures. Set all
`TESSARA_*_SIGNING_KEY`, `TESSARA_*_PUBLIC_KEY`, shared-key, and database
password variables from Compose secrets or Kubernetes Secrets outside local
development. Modules receive Core public verification keys only.

Start the slice with:

```powershell
docker compose -f deploy/sprint-6b2/compose.yaml up -d --build
```

## Guided administrator enrollment

Run the repository wrapper from the workspace root. It validates the local
stack, asks installation control to issue the claim, creates a short-lived
one-time browser handoff, and prints the claim secret exactly once:

```powershell
.\scripts\tessara.ps1 enrollment issue -Open
```

The browser handoff pre-fills the installation, claim, generation, and flow
kind. Paste the printed secret into the enrollment page and choose the new
administrator's identity and password. The browser URL does not contain the
claim secret.

Initial enrollment is unavailable while the installation already has a viable
administrator. When the installation has previously enrolled an administrator
but no viable administrator remains, issue a recovery claim instead:

```powershell
.\scripts\tessara.ps1 enrollment recover `
  -Reason "Restore administration after the only administrator was disabled" `
  -Operator "local-console" `
  -Open
```

Recovery records the signed local operator authorization before issuing the
claim. The development signing keys in `compose.yaml` are fixtures only and
must be replaced through the deployment secret mechanism outside local
development.

If a browser cannot be opened automatically, omit `-Open` and use the
`enrollment_url` printed by the command.

## Useful checks

```powershell
docker compose -f deploy/sprint-6b2/compose.yaml ps
docker compose -f deploy/sprint-6b2/compose.yaml logs --tail 100 core
```

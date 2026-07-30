# Sprint 6D UAT Scripts

Run these scripts in order against the dedicated Sprint 6D stack at
`http://127.0.0.1:8080`. Each file covers one user-observable scenario and
uses the source-exact deployment created from the commit that contains these
scripts.

1. [Fresh deployment and idempotent bootstrap](./uat-6d-01-deployment.md)
2. [Reference module navigation and SSR](./uat-6d-02-reference-experience.md)
3. [Accessible and responsive presentation](./uat-6d-03-presentation.md)
4. [Configuration and operational state](./uat-6d-04-configuration-operations.md)
5. [Authorization and nondisclosure](./uat-6d-05-authorization.md)
6. [Outage, recovery, and retained-product regression](./uat-6d-06-outage-regression.md)

Automated UAT is retained separately at
`artifacts/sprint-6d-closeout/uat-fresh.json`. Completed manual results are
retained at `artifacts/sprint-6d-closeout/manual-uat.md`.

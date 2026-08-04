# Sprint 7B Manual UAT

Run these nine scenarios only after scripted UAT passes for the exact
SIT-authorized frozen candidate. Retain one JSON receipt and non-secret evidence
paths per scenario under `artifacts/sprint-7b-closeout/uat/manual/`. Restore the
reference composition after every mutating scenario. Never retain credentials,
session material, or bearer grants.

1. [Lifecycle observation](./uat-7b-01.md)
2. [Deferral and recovery](./uat-7b-02.md)
3. [In-place update](./uat-7b-03.md)
4. [Successor upgrade](./uat-7b-04.md)
5. [Replace, remove, and conflict](./uat-7b-05.md)
6. [Archive and tombstone](./uat-7b-06.md)
7. [Resolution and outage matrix](./uat-7b-07.md)
8. [Dataset equivalence](./uat-7b-08.md)
9. [Approved visual contract](./uat-7b-09.md)

Each receipt records candidate commit/tree/image/config/catalog/database and
installation fingerprints, actor class, preconditions, exact actions, expected
and actual outcomes, restoration result, evidence digests, and pass/fail.

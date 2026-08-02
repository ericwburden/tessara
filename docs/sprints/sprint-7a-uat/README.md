# Sprint 7A Manual UAT

Run these scenarios only after the authoritative Sprint 7A SIT receipt passes.
Use the exact candidate and environment fingerprints from that receipt. Record
one result and non-secret evidence path per scenario; do not retain passwords,
cookies, bearer grants, or signing material.

1. [Scoped Dataset catalog](./uat-7a-01.md)
2. [Dataset tier rows](./uat-7a-02.md)
3. [Component execution](./uat-7a-03.md)
4. [Dashboard viewing](./uat-7a-04.md)
5. [Cross-boundary recovery](./uat-7a-05.md)
6. [Disjoint and known/random negatives](./uat-7a-06.md)
7. [Service misuse](./uat-7a-07.md)
8. [Freshness](./uat-7a-08.md)
9. [Compatibility](./uat-7a-09.md)
10. [Safe states and responsiveness](./uat-7a-10.md)
11. [Administrator control](./uat-7a-11.md)

Restore the reference composition, provider health, role/scope assignments,
and visibility state after every scenario that mutates them.

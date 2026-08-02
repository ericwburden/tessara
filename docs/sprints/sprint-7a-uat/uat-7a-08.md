# UAT-7A-08 — Freshness

- Issue an authorized request, then retain only its non-secret correlation ID.
- Through normal administration paths, change role/scope, visibility, provider
  authority, Organization, and delegation state one dimension at a time.
- Pass when the retained request becomes stale after each change and a fresh
  request immediately reflects current authority. Restore every mutation.

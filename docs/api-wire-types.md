# API Wire Types

Tessara currently keeps request and response DTOs in the API crate and mirrors the same JSON shapes in the Leptos frontend near the screens that consume them. That is acceptable while contracts are still moving quickly, but new stable API surfaces should be designed so the wire type has one durable home.

## Direction

- Keep backend endpoint modules split into route, handler/service, repo, and DTO concerns.
- Keep frontend transport behavior in `crates/tessara-web/src/infra/http.rs`.
- When an API contract stabilizes, move shared request/response shapes into a crate that both `tessara-api` and `tessara-web` can depend on.
- Preserve frontend-local view models when they are truly presentation-specific; only share JSON wire contracts.
- Update Playwright permission scenarios whenever a shared wire contract adds or changes a permission-controlled surface.

## Migration Rule

Do not introduce another ad hoc browser request helper for mutations or authenticated JSON parsing. Add typed client functions over `infra::http` instead, then migrate existing screens incrementally as their route families are touched.

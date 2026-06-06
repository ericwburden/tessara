# ORPHEUM

- Project root: repository root
- Project state: `initialized`
- Local skill: `.codex\skills\orpheum\SKILL.md`
- Local catalog config: `.codex\orpheum\config.json` if present
- Catalog root: `embedded-catalog`
- Catalog source: `embedded`
- Active session present: `false`

## Next Commands

- `orpheum scenario list --json`
- `orpheum scenario show <id> --json`
- `orpheum scenario apply <id> --json`

## Notes

- `orpheum init` makes the project Orpheum-capable, but does not apply a scenario.
- Catalog-dependent commands prefer explicit `--catalog`, then repo-local config, then `ORPHEUM_CATALOG`, then the embedded catalog.
- No external catalog override is tracked by default; Orpheum will use the embedded catalog unless `.codex\orpheum\config.json` or `ORPHEUM_CATALOG` is configured locally.

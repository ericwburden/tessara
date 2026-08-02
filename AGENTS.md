# Tessara Repository Instructions

- Use the repository-local `tessara-implementation` skill for every implementation, refactor, migration, fixture change, and implementation review before editing source.
- Treat Tessara as pre-production until the user explicitly states that it is post-production. Never infer post-production status from versions, releases, deployments, documentation, or repository state.
- Apply the skill's forward-only, canonical-ownership, touched-cone cleanup, test-integrity, and zero-warning rules by default. A conflicting compatibility requirement must be explicitly approved in the governing user request, requirement, or sprint plan.
- Use the specialized Tessara kickoff, validation, UAT, and closeout skills for their respective lifecycle phases; do not duplicate those phases inside implementation work.

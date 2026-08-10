These Tessara-specific Codex skills are the repo-tracked source of truth.

Installed copies live under the user's global skills directory. Sync them with:

```powershell
.\scripts\sync-codex-skills.ps1
```

The sync includes implementation, kickoff, validation coordination, Preflight,
SIT, UAT, and closeout so one installed lifecycle cannot retain stale policy
instructions from another phase.

Phase 8 Core-to-module work pairs kickoff and implementation with
`docs/architecture/module-extraction-playbook.md`. Its
`phase8-module-extraction` validation-contract profile makes Sprint 8A's
repeatable delivery lessons executable before formal validation.

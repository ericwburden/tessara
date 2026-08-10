These Tessara-specific Codex skills are the repo-tracked source of truth.

Installed copies live under the user's global skills directory. Sync them with:

```powershell
.\scripts\sync-codex-skills.ps1
```

The sync includes implementation, kickoff, validation coordination, Preflight,
SIT, UAT, and closeout so one installed lifecycle cannot retain stale policy
instructions from another phase.

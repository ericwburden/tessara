# <Sprint> Validation Record

## Scope and acceptance inventory

| Roadmap clause | Risk/contract | Automated proof | Smoke proof | Manual UAT proof |
|---|---|---|---|---|
| <clause> | <risk> | <assertion> | <assertion or N/A with reason> | <scenario> |

## Candidate identity

- Implementation commit:
- Tree:
- Dirty state:
- Image digest(s):
- Source-provenance labels:
- Deployment profile/configuration digest:
- Migration/baseline identity:
- Acceptance-manifest identity:

## Preflight

- Status: Not Run
- Environment and reset authorization:
- Test databases:
- Bootstrap/materialization and no-op proof:
- Harness/inventory reconciliation:
- Migration from-scratch proof:
- Evidence paths:

## SIT

| Lane | Command/evidence | Result | Duration |
|---|---|---|---|
| Static and boundaries | | Not Run | |
| Rust workspace | `cargo test --workspace --locked` | Not Run | |
| Playwright | `npm --prefix .\end2end test` | Not Run | |
| Deployed acceptance smoke | `.\scripts\smoke.ps1` | Not Run | |

## UAT

### Scripted UAT

- Command:
- Result: Not Run
- Evidence:

### Manual UAT

| Scenario | Role/start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| | | | | Not Run | |

## Failure and restart chronology

| Time | Gate/lane | Candidate | Classification | Correction | Narrow proof | SIT restart |
|---|---|---|---|---|---|---|

## Closeout authorization

- Status: Not Authorized
- Authorized candidate:
- SIT passed:
- UAT passed:
- Acceptance mapping complete:
- Unresolved product decisions:
- Intended active route/slot:
- Application health:
- Evidence source commit:
- Authorization timestamp:

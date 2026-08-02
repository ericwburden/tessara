# <Sprint> Validation Record

## Scope and acceptance inventory

| Roadmap clause | Risk/contract | Automated proof | Smoke proof | Manual UAT proof |
|---|---|---|---|---|
| <clause> | <risk> | <assertion> | <assertion or N/A with reason> | <scenario> |

## Required evidence inventory

| Artifact | Producer | Required before | Status |
|---|---|---|---|
| `preflight-result.json` | Preflight | Candidate freeze | Not Run |
| `candidate.json` | Preflight | SIT | Not Run |
| `sit-result.json` | SIT | UAT | Not Run |
| `uat-result.json` | UAT | Authorization | Not Run |
| `evidence-manifest.json` and sidecar | Validation phases | Authorization | Not Run |
| `closeout-authorization.json` | Coordinator | Closeout | Not Run |

## Candidate identity

- Implementation commit:
- Tree:
- Dirty state:
- Candidate fingerprint:
- Acceptance-inventory identity:
- Deployment profile/configuration digest:
- Migration/baseline identity:
- Expected provenance labels:
- Observed image digest(s):

## Environment contract

- Environment fingerprint:
- Tool versions:
- Test database identities and reset authorization:
- Compose project/profile and ports:
- Account/role fixture identities:
- Evidence root and output-path mode:

## Preflight

- Status: Not Run
- Receipt:
- Environment and reset authorization:
- Harness/inventory reconciliation:
- Bootstrap/no-op/restoration commands:
- Evidence paths and required artifact audit:

## SIT

| Lane | Prepare receipt | Command/evidence | Assertions | Result | Duration |
|---|---|---|---|---|---|
| Static and boundaries | | | | Not Run | |
| Rust workspace | | `cargo test --workspace --locked` | | Not Run | |
| Playwright | | `npm --prefix .\end2end test` | | Not Run | |
| Deployed acceptance smoke | | `.\scripts\smoke.ps1` | | Not Run | |

- SIT result receipt:
- Canonical topology restoration:

## UAT

### Scripted UAT

- Command:
- Result: Not Run
- Evidence:

### Manual UAT

| Scenario | Role/start state | Actions | Expected | Result | Evidence |
|---|---|---|---|---|---|
| | | | | Not Run | |

- UAT result receipt:
- Final topology restoration:

## Failure and invalidation chronology

| Time | Phase/lane/stage | Assertions started | Candidate | Classification | Correction/narrow proof | Invalidation scope | Authoritative replacement |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

## Evidence integrity

- Required files complete:
- Structured artifacts parse:
- Markdown links pass:
- Authoritative/superseded attempts distinguished:
- Manifest file count:
- Manifest SHA-256:

## Closeout authorization

- Status: Not Authorized
- Authorization receipt:
- Authorized candidate/fingerprint:
- SIT passed:
- UAT passed:
- Acceptance mapping complete:
- Invalidation decisions satisfied:
- Unresolved product decisions:
- Intended active route/slot:
- Application health:
- Evidence source commit:
- Documentation commit:
- Authorization timestamp:

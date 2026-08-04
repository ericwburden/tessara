# <Sprint> Validation Record

## Scope and acceptance inventory

| Roadmap clause | Risk/contract | Automated proof | Smoke proof | Manual UAT proof |
|---|---|---|---|---|
| <clause> | <risk> | <assertion> | <assertion or N/A with reason> | <scenario> |

## Required evidence inventory

| Artifact | Producer | Required before | Status |
|---|---|---|---|
| `validation-readiness-result.json` | Validation coordinator | Rehearsal | Not Run |
| `candidate-rehearsal-result.json` | Validation coordinator | Candidate freeze | Not Run |
| `preflight-result.json` | Preflight | Candidate freeze | Not Run |
| `candidate.json` | Preflight | SIT | Not Run |
| `sit-result.json` | SIT | UAT | Not Run |
| `uat-result.json` | UAT | Authorization | Not Run |
| `uat-defect-harvest.json` | UAT/coordinator | Correction batch, when triggered | Planned / Conditional |
| `defect-batch.json` | Coordinator | Impact assessment, when triggered | Planned / Conditional |
| `correction-impact-assessment.json` | Coordinator | Focused repair validation, when triggered | Planned / Conditional |
| `focused-repair-validation/attempt-<n>.json` | SIT/UAT/coordinator | Convergence, when triggered | Planned / Conditional |
| `canonical-restoration.json` | Coordinator | Convergence/final certification, when triggered | Planned / Conditional |
| `final-certification-entry.json` | Coordinator | Final readiness/rehearsal, when triggered | Planned / Conditional |
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

## Validation Readiness

- Derived executable checklist:
- Environment variables and reset acknowledgements:
- Supported tools, shells, and runtimes:
- Ports, Compose, databases, topology, health, and provenance:
- Semantic fixture and idempotence audit:
- Runner, output, receipt, hash, and finalization self-tests:
- Acceptance-clause evidence mapping:
- Clean repository and source-exact inputs:
- Result receipt:

## Candidate Rehearsal

| Diagnostic lane | Command/evidence | Assertions | Result | Defect batch |
|---|---|---|---|---|
| Static and boundaries | | | Not Run | |
| Full Rust | | | Not Run | |
| Source-exact deployment/materialization | | | Not Run | |
| Playwright | | | Not Run | |
| Conformance and nondisclosure | | | Not Run | |
| Deployed smoke | | | Not Run | |
| Recovery/restoration | | | Not Run | |
| Automated UAT diagnostics | | | Not Run | |

- Mutable source/environment identity:
- Passing readiness prerequisite:
- Consolidated defects and correction batch:
- Complete-cycle repetitions:
- Result receipt:

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

## Post-SIT Defect Convergence (conditional)

### UAT defect harvest

- Invalidated candidate/fingerprint:
- First invalidating failure and classification:
- Passing `uat-result.json` forbidden/absent:
- Harvest receipt:

| Scenario | Disposition | Authoritative | Prerequisites reconfirmed | Blocked dependency | Defect IDs | Evidence |
|---|---|---|---|---|---|---|
| | `<passed|failed|blocked|superseded>` | false | | | | |

### Consolidated correction batch

| Defect ID | Classification | Status | Discovery evidence | Correction | Supersedes |
|---|---|---|---|---|---|
| | | `<open|corrected|passed|blocked|superseded>` | | | |

- Batch receipt:
- Canonical restoration/prerequisite reconfirmation:

### Correction-impact assessment

| Changed file/contract/input/behavior | Affected checks and scenarios | Explicit exclusions | Evidence/rationale | Broad-cone trigger |
|---|---|---|---|---|
| | | | | |

- Impact assessment receipt:
- Coordinator authorization:
- Escalation when not confidently bounded:

### Focused repair validation

| Attempt | Mutable source/environment | Declared SIT/UAT cone | Passed | Blocked | New defects | Result receipt |
|---|---|---|---|---|---|---|
| | | | | | | |

- All defects passed/superseded with no open or blocked item:
- Canonical restoration passed:
- Decision to enter complete final readiness/rehearsal:
- Final-certification-entry receipt:
- Successor full readiness/rehearsal/SIT/UAT chain:

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

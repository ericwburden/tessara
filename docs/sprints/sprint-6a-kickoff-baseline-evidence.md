# Sprint 6A Retrospective Clean-Base Evidence

## Evidence status

This is a **retrospective** validation run performed on 2026-07-14. It does not
claim that these commands ran during Sprint 6A kickoff. The run used the exact
kickoff commit requested by the Sprint plan:

- commit: `3625d4de52c5856e4ac3bc642a9422a029e9f375`
- commit date: `2026-07-13T23:27:03-04:00`
- subject: `docs: define modular application platform roadmap`
- checkout: detached, clean temporary worktree at
  `C:\Users\eric-dev\Projects\tessara-sprint-6a-baseline-evidence`
- build isolation: `CARGO_TARGET_DIR=C:\Users\eric-dev\Projects\tessara-sprint-6a-baseline-target`
- database isolation: disposable database `tessara_sprint6a_baseline_test` in
  the existing `tessara-sprint6a-test-postgres` container, exposed on local
  port `55432`

The temporary source worktree was removed after this report was written. The
disposable database was deliberately retained for Sprint closeout inspection.
No developer stack was launched, stopped, rebuilt, or reset.

## Environment

| Tool | Version |
| --- | --- |
| Operating system | Microsoft Windows 11 Pro, version `10.0.26200`, 64-bit |
| Git | `2.53.0.windows.3` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)`; LLVM `22.1.2` |
| Cargo | `1.95.0 (f2d3ce0bd 2026-03-21)` |
| rustfmt | `1.9.0-stable (59807616e1 2026-04-14)` |
| Docker | `29.4.3`, build `055a478` |
| PostgreSQL client/container | `16.14` |
| Node.js | `v24.15.0` |
| npm | `11.15.0` |
| Playwright used for discovery | `1.59.1` |

The database began empty. The workspace test run applied and retained exactly
the two base migrations:

| Version | Description | Success |
| --- | --- | --- |
| 1 | `baseline` | true |
| 2 | `dashboard placement capacity` | true |

## Commands and results

Durations are measured end-to-end wall-clock times around each command. Test
counts are reported only when the runner emitted them; format and compile
checks do not have pass/skip/filter counts.

| Command | Exit | Duration | Tests/results |
| --- | ---: | ---: | --- |
| `cargo fmt --all -- --check` | 0 | 6.477 s | Pass; test counts not applicable |
| `cargo check --workspace --all-features --locked` | 0 | 352.811 s | Pass; zero compiler warning/error diagnostics; test counts not applicable |
| `cargo test --workspace --all-features --locked -j 2` | 0 | 1,955.618 s (32m 35.618s) | 360 passed, 0 failed, 0 ignored/skipped, 0 measured, 0 filtered across 54 runner result blocks |
| `npm ci` from `end2end` | 0 | 4.251 s | 3 packages installed, 0 vulnerabilities; test counts not applicable |
| `npx playwright test --list` from `end2end` | 0 | 3.732 s | 50 tests discovered in 6 files; no tests executed, so pass/fail/skip/filter counts are not applicable |

The Cargo test command reported a 20m 05s test-profile build inside the overall
wall-clock duration. It also emitted 34 `has been running for over 60 seconds`
diagnostics while database-backed integration tests ran; those tests completed
successfully. There were no retries, test edits, skipped tests, filtered tests,
or reruns used to obtain the passing result.

The exact validation environment and commands were:

```powershell
$env:CARGO_TARGET_DIR = 'C:\Users\eric-dev\Projects\tessara-sprint-6a-baseline-target'
cargo fmt --all -- --check
cargo check --workspace --all-features --locked

$env:CARGO_BUILD_JOBS = '2'
$env:TEST_DATABASE_URL = 'postgres://tessara:tessara_test_password@127.0.0.1:55432/tessara_sprint6a_baseline_test'
cargo test --workspace --all-features --locked -j 2

Set-Location end2end
npm ci
npx playwright test --list
```

Playwright was discovery-only. Its successful exit proves that the base suite
could be loaded and enumerated; it is not browser execution evidence and does
not prove those 50 scenarios passed.

## Base characterization limits and known red route inventory

The green base workspace suite is evidence only for tests that existed at
commit `3625d4d`. Sprint 6A characterization artifacts and tests did not yet
exist there, including the Sprint 6A regression matrix, module-control-plane
tests, semantic-destination/reference tests, dynamic-navigation actor matrix,
populated upgrade proof, and `end2end/tests/modules.spec.ts`. They therefore
could not truthfully be run as part of this clean-base command.

The detached source inventory also confirms the pre-existing native-route gap
recorded by the Sprint plan:

- Leptos declared `/datasets/:dataset_id/preview` and
  `/datasets/:dataset_id/revisions/:revision_id/edit`.
- The base Axum document router registered Dataset directory, create, detail,
  edit, revisions, and revision-detail routes, but not those preview and
  revision-edit direct-load routes.
- No existing base test failed for that mismatch, so the 360-test green result
  must not be cited as proof that those two direct loads worked. The missing
  route registrations remain a known red pre-Sprint defect for the new
  characterization coverage to expose and the Sprint implementation to fix.

Sprint 6A-only routes such as `/administration/modules`, module platform APIs,
and their authorization behavior were absent by design at the base commit.
Their absence is not a base regression and their later tests are closing-build
evidence, not retrospective kickoff evidence.

## Interpretation

This run establishes a reproducible clean-source baseline for the broad Rust
workspace at the requested commit while preserving two important limits:

1. passing legacy tests are durable compatibility evidence and must not be
   weakened or casually rewritten during Sprint 6A; and
2. legacy-suite green status cannot substitute for characterization that was
   missing at the base commit, especially the two known Dataset direct-load
   route gaps and the Sprint 6A-specific contracts and actor matrices.

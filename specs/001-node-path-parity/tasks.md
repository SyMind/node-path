# Tasks: Node.js Path Behavioral Parity

**Input**: Design documents from `specs/001-node-path-parity/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, and
`quickstart.md`

**Tests**: Required. The feature explicitly requires direct upstream test porting, oracle-backed
behavior, differential/property coverage, and CodSpeed validation. Within every story, test tasks
precede implementation tasks and must demonstrate failure before the corresponding behavior is
implemented.

**Organization**: Tasks are grouped by user story. IDs are globally ordered; `[P]` marks tasks that
can proceed concurrently without editing the same files or depending on unfinished work.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize a reproducible Rust 2024 library and repository layout.

- [X] T001 Initialize the Rust 2024 `node-path` library, resolver 3, MSRV 1.88, exact `regress = 0.11.1` features, dev dependencies, package notice inclusion, and `path_ops` bench target in `Cargo.toml`
- [X] T002 [P] Pin the reference toolchain to Rust 1.97.1 and ignore generated build/report output in `rust-toolchain.toml` and `.gitignore`
- [X] T003 Create the planned module, integration-test, oracle, fixture, benchmark, and workflow directory skeleton with compileable module declarations in `src/lib.rs`, `src/glob/mod.rs`, `tests/node_path.rs`, and `benches/path_ops.rs`
- [X] T004 [P] Add the pinned Node license and derived-material provenance, including minimatch/brace-expansion notice requirements, in `UPSTREAM_LICENSE_NODE.txt` and `THIRD_PARTY_NOTICES.md`
- [X] T005 [P] Configure MSRV and Rust 1.97.1 Linux/macOS/Windows formatting, lint, build, and test jobs in `.github/workflows/ci.yml`

**Checkpoint**: Cargo metadata resolves, the planned file tree exists, and notices/CI policy are
visible before feature behavior is added.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish shared types, deterministic environment handling, evidence models, and the
test runner required by every user story.

**⚠️ CRITICAL**: No user story implementation begins until this phase compiles on Rust 1.88.

- [X] T006 Write failing contract tests for Node host detection, empty cwd, Unicode conversion errors, ASCII-insensitive drive lookup, duplicate devices, mismatched drive fallback, and immutable environment snapshots in `tests/context_contract.rs`
- [X] T007 Define dependency-independent `ContextError` and `GlobError` variants with `Display`, `Error`, and equality behavior required by the public contract in `src/error.rs`
- [X] T008 Implement `NodeHost`, `DriveCwd`, `PathContext::new`, `PathContext::from_env`, hidden Windows drive-cwd enumeration, validation, and read-only accessors to pass T006 in `src/context.rs`
- [X] T009 [P] Define generic owned/borrowed `PathObject<S>` and `ParsedPath<'a>` records with empty defaults and public field semantics in `src/path_object.rs`
- [X] T010 Add failing unit tests for ASCII separators, drive roots, dot-segment classification, UTF-8-safe slicing, and output-capacity helpers in `src/shared.rs`
- [X] T011 Implement only the safe namespace-neutral string primitives required to pass T010, without using `std::path` or adding allocation-heavy abstractions, in `src/shared.rs`
- [X] T012 [P] Implement Serde models for baselines, source anchors, contexts, case disposition/verification, approvals, summaries, stable IDs, and hashes from the two JSON contracts in `tests/node_path/support.rs`
- [X] T013 Create the compileable aggregate conformance-runner skeleton, module registry, report path, and final gate entry point in `tests/node_path.rs`
- [X] T014 [P] Generate and check in the initial Node commit, version, implementation/minimatch blobs, 17 test blob OIDs, license references, and provisional 17/301/1,406 counts in `tests/fixtures/node-path/baseline.json`
- [X] T015 Wire public modules/types/errors into `src/lib.rs`, generate and commit `Cargo.lock`, and make `cargo +1.88.0 test --all-targets --all-features --no-run` succeed without weakening required dependency features

**Checkpoint**: Context and evidence foundations compile on the MSRV; no global state mutation or
filesystem-path semantics are present.

---

## Phase 3: User Story 1 - Use Node-Compatible Path Operations (Priority: P1) 🎯 MVP

**Goal**: Deliver the complete public POSIX, Windows, and host-default API with representative
pinned-Node behavior across every advertised operation.

**Independent Test**: Run the five US1 contract suites on any supported host. Every public operation,
constant, namespace, ownership mapping, context input, error, and representative edge case must
match the pinned Node result; the exhaustive 17-file audit is not yet required for this checkpoint.

### Tests for User Story 1

> Write these tests first and confirm they fail for unimplemented behavior.

- [X] T016 [P] [US1] Write failing public namespace, constant, snake-case mapping, borrowed/owned return, deprecated alias, and host-default selection contract tests in `tests/api_contract.rs`
- [X] T017 [P] [US1] Write failing representative POSIX tests for all context-free operations, parse/format precedence, empty input, roots, repeated separators, dot segments, backslash-as-data, suffix/dot, and Unicode cases in `tests/posix_contract.rs`
- [X] T018 [P] [US1] Write failing representative Windows tests for all context-free operations, drives, UNC, device/namespaced paths, reserved names, mixed separators, streams, suffix/dot, and Unicode cases in `tests/win32_contract.rs`
- [X] T019 [P] [US1] Write failing deterministic tests for POSIX/Windows resolve, relative, to-namespaced-path, drive cwd fallback, empty cwd, and environment-backed facade equivalence in `tests/contextual_contract.rs`
- [X] T020 [P] [US1] Write failing glob contract tests for pinned Node options, Win32/Darwin/OtherPosix case modes, separators, classes, globstar, braces, extglobs, malformed patterns, 65,536 UTF-16-unit limits, and astral Unicode in `tests/glob_contract.rs`

### Implementation for User Story 1

- [X] T021 [US1] Implement allocation-conscious shared normalization scanning, component accumulation, root handling, and capacity planning needed by both namespaces in `src/shared.rs`
- [X] T022 [P] [US1] Implement POSIX `SEP`, `DELIMITER`, normalize, is-absolute, join, dirname, basename, and extname behavior to pass T017 in `src/posix.rs`
- [X] T023 [P] [US1] Implement Windows `SEP`, `DELIMITER`, normalize, is-absolute, join, dirname, basename, and extname behavior to pass T018 in `src/win32.rs`
- [X] T024 [US1] Implement borrowed parse results and Node-compatible format precedence/extension behavior for both namespaces using `PathObject` in `src/path_object.rs`, `src/posix.rs`, and `src/win32.rs`
- [X] T025 [P] [US1] Implement POSIX resolve, relative, and identity to-namespaced-path with explicit `PathContext`, plus environment-backed adapters, in `src/posix.rs`
- [X] T026 [P] [US1] Implement Windows resolve, relative, to-namespaced-path, drive-cwd fallback, UNC/device handling, and `_make_long` alias with explicit `PathContext` in `src/win32.rs`
- [X] T027 [US1] Implement crate-root target-selected constants/functions, context snapshot delegation, public error/type exports, and deprecated alias mapping in `src/lib.rs`
- [X] T028 [P] [US1] Port pinned minimatch 10.2.5 brace expansion, expansion limits, UTF-16 length accounting, and required upstream notices in `src/glob/brace.rs`
- [X] T029 [US1] Port pinned minimatch parsing/optimization/regex-source generation, extglob/globstar recursion limits, separator rules, and Node option handling in `src/glob/minimatch.rs`
- [X] T030 [US1] Integrate exact `regress` 0.11.1 UTF-16/UCS-2 execution, `NodeHost` case behavior, pattern resource errors, and no-cache matching in `src/glob/mod.rs`
- [X] T031 [US1] Expose deterministic and environment-backed `matches_glob` functions through explicit namespaces and the crate root without leaking dependency types in `src/posix.rs`, `src/win32.rs`, and `src/lib.rs`
- [X] T032 [US1] Add deterministic proptest invariants for normalization idempotence, parse/format consistency, namespace host-independence under identical context, and glob replay seeds in `tests/generated.rs`

**Checkpoint**: All US1 contract and generated tests pass on every supported host. This is a usable
MVP demonstration, but it is not constitutionally release-ready until US2 completes the upstream
corpus.

---

## Phase 4: User Story 2 - Verify Parity from Upstream Tests (Priority: P2)

**Goal**: Directly port, classify, run, and audit every assertion from the 17 pinned Node path-test
files, producing a release-gating machine-readable parity report.

**Independent Test**: `cargo +1.97.1 test --test node_path -- --nocapture` must write a schema-valid
`target/parity-summary.json` with commit `3f42cfac…`, inventory 17/301/1,406, zero missing/orphan/
unapproved/failing/pending/stale cases, and `releasable: true`.

### Ported Tests for User Story 2

> Port these modules before fixing newly exposed implementation gaps. Each task preserves upstream
> inputs, comparator metadata, line/vector anchors, case hashes, and applicable notices.

- [X] T033 [P] [US2] Port `test-path.js` constants, expanded invalid-type boundaries, and host-default identity replacements as 180 classified cases with the original Joyent/Node header in `tests/node_path/test_path.rs`
- [X] T034 [P] [US2] Port all 61 POSIX/Windows basename and suffix/control-character cases from `test-path-basename.js` in `tests/node_path/test_path_basename.rs`
- [X] T035 [P] [US2] Port all 45 root, drive, UNC, stream, separator, and deterministic `__filename` dirname cases from `test-path-dirname.js` in `tests/node_path/test_path_dirname.rs`
- [X] T036 [P] [US2] Expand and port all 145 dot, trailing-separator, and namespace-specific extension outcomes from `test-path-extname.js` in `tests/node_path/test_path_extname.rs`
- [X] T037 [P] [US2] Port all 22 `matchesGlob` outcomes and typed non-string boundaries from `test-path-glob.js` in `tests/node_path/test_path_glob.rs`
- [X] T038 [P] [US2] Port all 22 POSIX root and Windows drive/UNC absolute-path outcomes from `test-path-isabsolute.js` in `tests/node_path/test_path_isabsolute.rs`
- [X] T039 [P] [US2] Expand and port all 183 common/Windows join outcomes, retaining upstream alternative-comparator provenance while asserting the exact Node result, from `test-path-join.js` in `tests/node_path/test_path_join.rs`
- [X] T040 [P] [US2] Port all 34 deterministic fixture, namespace, device/UNC, and non-string boundary cases from `test-path-makelong.js`, preserving its full Joyent/Node header, in `tests/node_path/test_path_makelong.rs`
- [X] T041 [P] [US2] Port all 66 POSIX/Windows normalization outcomes, including CVE-2024-36139 and traversal regressions, from `test-path-normalize.js` in `tests/node_path/test_path_normalize.rs`
- [X] T042 [P] [US2] Expand and port all 486 parse/format field, round-trip, precedence, trailing-separator, and typed boundary outcomes from `test-path-parse-format.js`, preserving its full Joyent/Node header, in `tests/node_path/test_path_parse_format.rs`
- [X] T043 [P] [US2] Classify the CommonJS identity assertion from `test-path-posix-exists.js` and add the equivalent Rust POSIX namespace accessibility check in `tests/node_path/test_path_posix_exists.rs`
- [X] T044 [P] [US2] Port the POSIX-relative-on-Windows case with deterministic cwd and retained regexp-comparator provenance from `test-path-posix-relative-on-windows.js` in `tests/node_path/test_path_posix_relative_on_windows.rs`
- [X] T045 [P] [US2] Expand and port all 40 POSIX/Windows relative outcomes, including drive, UNC, Turkish `İ`, combining-dot, and `ß` behavior, from `test-path-relative.js` in `tests/node_path/test_path_relative.rs`
- [X] T046 [P] [US2] Expand and port all 25 resolve outcomes using deterministic cwd, fixture, empty-cwd, and hidden drive-cwd contexts from `test-path-resolve.js` in `tests/node_path/test_path_resolve.rs`
- [X] T047 [P] [US2] Classify the CommonJS identity assertion from `test-path-win32-exists.js` and add the equivalent Rust Windows namespace accessibility check in `tests/node_path/test_path_win32_exists.rs`
- [X] T048 [P] [US2] Remove the upstream host skip and port all 78 reserved-device, UNC, traversal, and stream normalization outcomes from `test-path-win32-normalize-device-names.js` in `tests/node_path/test_path_win32_normalize_device_names.rs`
- [X] T049 [P] [US2] Port all 16 empty-string join, normalize, is-absolute, resolve, and relative outcomes with deterministic cwd from `test-path-zero-length-strings.js` in `tests/node_path/test_path_zero_length_strings.rs`

### Evidence and Parity Completion for User Story 2

- [X] T050 [US2] Write failing schema/completeness tests for unique IDs/hashes, blob provenance, dual 301/1,406 coverage, legal lifecycle combinations, approvals, and zero orphan/missing cases in `tests/parity_schema.rs`
- [X] T051 [US2] Populate stable source/vector anchors, canonical inputs/context/expectations, dispositions, verification state, replacement checks, blob OIDs, hashes, and license references for all 1,406 cases in `tests/fixtures/node-path/parity-ledger.json`
- [X] T052 [US2] Complete module discovery, ledger reconciliation, case execution, failure aggregation, schema-shaped JSON output, and release-gate assertions in `tests/node_path.rs` and `tests/node_path/support.rs`
- [X] T053 [P] [US2] Implement the test-only line-delimited request/response adapter for pinned POSIX, Windows, contextual, structured, and glob oracle calls in `tests/oracle/path-oracle.mjs`
- [X] T054 [US2] Implement ignored-by-default pinned-binary verification and replayable randomized differential suites for normalization, parsing, resolving, and globbing in `tests/differential.rs`
- [X] T055 [US2] Resolve every newly exposed POSIX semantic failure without changing expected vectors or approved boundaries in `src/posix.rs` and `src/shared.rs`
- [X] T056 [US2] Resolve every newly exposed Windows semantic failure, including Unicode case handling and exact separator output, without weakening comparators in `src/win32.rs` and `src/shared.rs`
- [X] T057 [US2] Resolve cwd, drive-cwd, host-default, empty-cwd, and environment-adapter parity failures without process mutation in `src/context.rs`, `src/posix.rs`, and `src/win32.rs`
- [X] T058 [US2] Resolve every pinned and differential glob mismatch while preserving exact minimatch limits/options and the safe regex dependency configuration in `src/glob/brace.rs`, `src/glob/minimatch.rs`, and `src/glob/mod.rs`
- [X] T059 [US2] Review the provisional 29-site/420-expanded JavaScript-only boundary set, add precise reasons/replacement evidence, obtain maintainer approvals, and reject any missing-implementation exclusion in `tests/fixtures/node-path/parity-ledger.json`
- [X] T060 [US2] Run the aggregate and differential acceptance commands, fix only source/ledger defects, and produce a schema-valid release-ready report at `target/parity-summary.json`

**Checkpoint**: All 17 upstream files, 301 assertion sites, and 1,406 expanded cases are classified;
every representable case passes and every true representation boundary is approved.

---

## Phase 5: User Story 3 - Protect Path-Processing Performance (Priority: P3)

**Goal**: Establish a reproducible, compatibility-gated CodSpeed benchmark suite that detects
operation-specific regressions.

**Independent Test**: With `target/parity-summary.json` release-ready, local Divan and
`cargo codspeed build/run -m simulation --bench path_ops` discover every stable matrix ID; the hosted
`CodSpeed Performance Analysis` check compares a pull request against the pinned default-branch
baseline and reports no unexplained regression over its configured threshold.

### Tests for User Story 3

- [X] T061 [P] [US3] Write a failing benchmark-contract test for unique stable IDs, full operation/namespace coverage, required short/long/clean/dirty/structural/Unicode categories, fixed contexts, and `_vN` fixture versioning in `tests/benchmark_contract.rs`

### Implementation for User Story 3

- [X] T062 [US3] Define immutable at-most-64-byte short, approximately-1-KiB long, POSIX root/backslash, Windows drive/UNC/namespaced, Unicode, suffix/dot, and glob hit/miss fixtures with fixed contexts in `benches/cases.rs`
- [X] T063 [US3] Implement the Divan-compatible `path_ops/{posix|win32}/{operation}/{semantic_case}_v1` matrix with setup outside closures and measured output allocation in `benches/path_ops.rs`
- [X] T064 [P] [US3] Configure Ubuntu 24.04, Rust 1.97.1, `cargo-codspeed = 5.0.1`, OIDC, CPU Simulation, default-branch, pull-request, and workflow-dispatch execution in `.github/workflows/codspeed.yml`
- [X] T065 [US3] Run ordinary Divan plus local CodSpeed build/discovery, correct only unstable IDs or invalid measurement boundaries, and make `tests/benchmark_contract.rs` pass against `benches/cases.rs` and `benches/path_ops.rs`
- [X] T066 [US3] Document compiler/runner pinning, fixture immutability, initial 10% global threshold, per-benchmark tuning, rename/version rules, and compatibility-required slowdown evidence in `benches/README.md`
- [ ] T067 [US3] Configure the CodSpeed project threshold and required `CodSpeed Performance Analysis` branch check, recording the resulting project/check evidence in `benches/README.md`
- [ ] T068 [US3] Establish the first default-branch baseline only after verifying `.releasable == true`, then confirm a pull-request comparison contains every expected benchmark ID with no unexplained regression in `target/parity-summary.json` and `.github/workflows/codspeed.yml`

**Checkpoint**: Performance comparisons are stable, required, operation-specific, and unable to
override a compatibility failure.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize consumer documentation, packaging, governance metadata, and release evidence.

- [X] T069 [P] Document installation, namespace/API mapping, supported string domain, context usage, Node baseline, parity-report command, and compatibility-before-performance policy in `README.md`
- [X] T070 Add public rustdoc examples, ownership/error guarantees, deterministic-context guidance, and pinned `matchesGlob` semantics without duplicating implementation details in `src/lib.rs`, `src/context.rs`, `src/path_object.rs`, `src/posix.rs`, `src/win32.rs`, and `src/glob/mod.rs`
- [X] T071 Verify `cargo package --list` includes project/upstream licenses and provenance but excludes generated reports/oracle build artifacts, correcting package metadata in `Cargo.toml`, `UPSTREAM_LICENSE_NODE.txt`, and `THIRD_PARTY_NOTICES.md`
- [X] T072 Audit direct/transitive licenses, exact dependency versions, Rust 1.88 compatibility, `regress` safety features, and all project `unsafe` occurrences; fix any unexplained result in `Cargo.toml`, `Cargo.lock`, and `src/`
- [X] T073 Resolve the constitution Sync Impact Report/footer version mismatch through `$speckit-constitution` without changing feature principles in `.specify/memory/constitution.md`
- [ ] T074 Execute every MSRV, fmt, clippy, cross-platform conformance, parity-summary, differential, and local benchmark command from `specs/001-node-path-parity/quickstart.md`, updating that guide only when the implemented command contract differs intentionally
- [ ] T075 Run the final release gate with zero known parity failures, a release-ready `target/parity-summary.json`, complete package notices, and a passing required CodSpeed check using `.github/workflows/ci.yml` and `.github/workflows/codspeed.yml`

---

## Dependencies & Execution Order

### Phase Dependencies

| Phase | Depends on | Blocking outcome |
|-------|------------|------------------|
| Phase 1: Setup | None | Reproducible crate and repository skeleton |
| Phase 2: Foundational | Phase 1 | MSRV-compiling context/evidence foundation; blocks every story |
| Phase 3: US1 | Phase 2 | Complete representative public API MVP |
| Phase 4: US2 | Phase 3 for passing results; port modules may be authored after Phase 2 | Exhaustive release-ready Node parity evidence |
| Phase 5: US3 | Phase 4 and `.releasable == true` | Accepted performance baseline and regression gate |
| Phase 6: Polish | All selected stories | Package/release readiness |

### User Story Completion Order

```text
Setup -> Foundation -> US1 public compatibility -> US2 exhaustive parity -> US3 performance gate
```

- **US1 (P1)**: No story dependency after Foundation. Delivers all advertised APIs under a
  representative oracle-backed corpus.
- **US2 (P2)**: Test modules can be authored from the contracts after Foundation, but story
  acceptance depends on US1 behavior because every representable port must pass.
- **US3 (P3)**: Strictly depends on US2's release-ready report. No benchmark baseline is accepted
  earlier.

### Within Each Story

- Test tasks execute first and must fail for missing behavior or evidence.
- Shared models/context precede namespace services.
- Explicit namespace behavior precedes host-default facade integration.
- Ported source modules precede ledger finalization and parity fixes.
- Conformance becomes release-ready before benchmark fixtures or baselines are accepted.

## Parallel Opportunities

- Setup metadata, notices, and normal CI (`T002`, `T004`, `T005`) can proceed concurrently.
- Foundational path-object, evidence-model, and baseline work (`T009`, `T012`, `T014`) can proceed
  alongside the ordered context/shared work.
- US1's five failing contract suites (`T016`–`T020`) can be authored concurrently; POSIX/Windows
  implementations (`T022`/`T023`, then `T025`/`T026`) can be paired across different files.
- All 17 one-to-one upstream port modules (`T033`–`T049`) can be authored concurrently after their
  shared harness contract is stable.
- The oracle adapter (`T053`) can proceed while ledger/report integration is underway.
- The benchmark contract (`T061`) and hosted workflow (`T064`) can be prepared concurrently after
  US2 is release-ready.
- README work (`T069`) can proceed alongside final audits that do not edit `README.md`.

## Parallel Example: User Story 1

```text
Run together: T016, T017, T018, T019, T020
After T021/T024, run together: T022 with T023, then T025 with T026
T028 can proceed independently before T029 and T030
```

## Parallel Example: User Story 2

```text
Run together: T033 through T049 (17 distinct Rust counterpart files)
Run T053 while T050 through T052 assemble ledger/report evidence
Then run T055 through T058 in order when they touch shared source files
```

## Parallel Example: User Story 3

```text
Run together after US2: T061 and T064
Then run T062 -> T063 -> T065 -> T066 -> T067 -> T068
```

---

## Implementation Strategy

### MVP First: User Story 1

1. Complete Setup and Foundational phases.
2. Write all US1 contract tests and confirm missing behavior fails.
3. Implement shared, explicit namespace, context, parse/format, glob, and root-facade behavior.
4. Stop and validate US1 independently on all supported hosts.
5. Treat this as a demonstration/MVP only; it is not releasable until US2 completes 17/301/1,406
   evidence.

### Incremental Delivery

1. **Foundation**: Reproducible crate, safe context, and evidence model.
2. **US1**: Complete public API under representative pinned cases.
3. **US2**: Exhaustive direct upstream ports, approved boundaries, differential evidence, and
   release-ready parity summary.
4. **US3**: Stable CodSpeed matrix, accepted baseline, and required performance check.
5. **Polish**: Documentation, packaging, audit, governance metadata, and final release gate.

### Parallel Team Strategy

After Foundation:

- API workers split POSIX, Windows, and glob implementations under US1.
- Test workers can begin distinct US2 counterpart modules against the frozen API contract while
  implementation progresses; ported tests remain red until US1 completes.
- Performance work starts only after the aggregate US2 report is release-ready.

## Notes

- `[P]` means different files and no unmet dependency; do not parallelize tasks that both edit
  `src/shared.rs`, namespace files, the central ledger, or the report runner.
- Every upstream port preserves source/vector anchors and expected semantics; harness adaptation is
  not permission to weaken a comparator.
- A non-representable entry means the Rust type/API cannot express a JavaScript runtime property; it
  never means missing implementation.
- Keep normal tests independent of Node. Use the pinned executable only for explicit differential or
  baseline-refresh work.
- Re-run parity before interpreting any performance result.

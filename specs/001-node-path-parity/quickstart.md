# Quickstart: Validate Node.js Path Parity End to End

This guide describes the commands and expected evidence once implementation tasks are complete. It
does not require Node.js for the normal conformance run because the pinned expectations and
provenance are checked into the project.

## 1. Prerequisites

- Git
- Rust 1.88.0 for MSRV validation
- Rust 1.97.1 for the initial reference and CodSpeed environment
- `jq` for inspecting the generated parity summary
- Optional: a Node executable built from commit
  `3f42cfacf27e348297a52d89b4cdc48b35cb7559` for oracle refresh/differential runs

Install the two Rust toolchains:

```bash
rustup toolchain install 1.88.0 1.97.1
```

The public API to exercise is defined in [contracts/rust-api.md](contracts/rust-api.md).

## 2. Validate Toolchain and Static Quality Gates

From the repository root:

```bash
cargo +1.88.0 test --all-targets --all-features --no-run --locked
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy --all-targets --all-features --locked -- -D warnings
```

Expected outcome:

- The entire crate, test corpus, reporter, and benchmark targets compile on the declared MSRV.
- Formatting and linting complete with no diagnostics.
- The exact `regress` dependency builds with `std`, `utf16`, and `prohibit-unsafe` on Rust 1.88.

An MSRV build failure is a planning-gate failure; do not silently raise `rust-version` or remove
UTF-16/safety features. Rust 1.88 is the accepted minimum because exact `regress` 0.11.1 uses the
let-chain syntax stabilized in that release.

## 3. Run the Full Test Suite

```bash
cargo +1.97.1 test --all-targets --all-features --locked
```

Expected outcome:

- Unit tests for shared, POSIX, Windows, context, parse/format, and glob logic pass.
- Explicit POSIX and Windows behavior passes on the current host.
- Host-default adapter checks match the current target.
- No test mutates cwd or process environment.

CI repeats conformance on Linux, macOS, and Windows. CodSpeed uses a separate single Linux job and
does not replace the cross-platform test matrix.

## 4. Generate and Verify the Parity Summary

Run the central upstream-port harness:

```bash
cargo +1.97.1 test --test node_path -- --nocapture
```

The runner writes `target/parity-summary.json` before its final release-gate assertion. Validate the
initial inventory and release state:

```bash
jq -e '
  .baseline_commit == "3f42cfacf27e348297a52d89b4cdc48b35cb7559" and
  .inventory.source_files == 17 and
  .inventory.assertion_sites == 301 and
  .inventory.expanded_cases == 1406 and
  .results.failing == 0 and
  .results.pending == 0 and
  .results.stale == 0 and
  .results.unclassified == 0 and
  (.failures | length) == 0 and
  (.missing_case_ids | length) == 0 and
  (.orphan_local_test_ids | length) == 0 and
  (.unapproved_boundaries | length) == 0 and
  .releasable == true
' target/parity-summary.json
```

Expected outcome: `jq` exits with status 0. The summary conforms to
[contracts/parity-summary.schema.json](contracts/parity-summary.schema.json), and the checked-in
ledger conforms to [contracts/parity-ledger.schema.json](contracts/parity-ledger.schema.json).

If a case is non-representable, inspect its approval and replacement evidence. “Not implemented” is
never an acceptable boundary reason.

## 5. Exercise Representative Public Behavior

The conformance suite must demonstrate at least these end-to-end scenarios through the public API:

1. POSIX normalization and resolution with repeated separators, dot segments, and a supplied cwd.
2. Windows normalization and resolution for drive-relative, drive-absolute, UNC, device, and
   namespaced paths using supplied drive cwd mappings.
3. `parse` followed by `format`, including field precedence and extension-dot behavior.
4. `basename` suffix removal and `extname` handling for leading/trailing/multiple dots.
5. `matches_glob_with_context` under Win32, Darwin, and OtherPosix Node hosts for both namespaces.
6. Crate-root default selection on the current target.
7. Deprecated `_make_long` returning the same content as `to_namespaced_path`.

Each failure must report its stable upstream case ID, expected result, actual result, source file,
baseline commit, and local counterpart.

## 6. Optional Pinned-Oracle Differential Run

Differential tests are required when refreshing generated fixtures, changing high-risk parsing or
normalization logic, or modifying glob compilation. They are not a runtime dependency.

```bash
NODE_PATH_ORACLE_BIN=/absolute/path/to/pinned-node \
  cargo +1.97.1 test --test differential -- --ignored --nocapture
```

Expected outcome:

- The harness first verifies that the configured oracle corresponds to the recorded baseline.
- Generated POSIX, Windows, Unicode, cwd/drive-context, and glob cases match Node.
- Any mismatch prints a replayable seed/input and does not update fixtures automatically.

Fixture or ledger updates require an explicit review of changed Node blobs, case counts, hashes,
expectations, representation boundaries, and notices.

## 7. Validate Benchmarks Locally

Install the pinned cargo integration:

```bash
cargo install cargo-codspeed@5.0.1 --locked
```

Run ordinary local feedback and verify CodSpeed discovery:

```bash
cargo +1.97.1 bench --bench path_ops
cargo +1.97.1 codspeed build -m simulation --locked --bench path_ops
cargo +1.97.1 codspeed run -m simulation --bench path_ops
```

Expected outcome:

- Benchmark IDs follow `path_ops/{posix|win32}/{operation}/{semantic_case}_v1`.
- Every path-processing operation has the required applicable short, long, clean, dirty, structural,
  and Unicode cases.
- Contextual operations use fixed `PathContext` fixtures; environment capture and Node subprocesses
  are outside measured closures.
- Inputs are black-boxed, outputs are returned or black-boxed, fixture setup is outside the measured
  closure, and output allocation remains measured.

Local macOS/Windows timing is useful for development but is not compared with the hosted baseline.

## 8. Establish or Compare the Hosted Baseline

The GitHub workflow runs on Ubuntu 24.04 with Rust 1.97.1 using CPU Simulation and
`CodSpeedHQ/action@v5`. Before establishing the first baseline, verify:

```bash
jq -e '.releasable == true' target/parity-summary.json
```

Expected hosted outcome:

- Default-branch runs establish baseline values.
- Pull requests compare against the latest base-branch run.
- The required `CodSpeed Performance Analysis` check reports no benchmark over the initial global
  10% regression threshold.
- Compatibility-required slowdowns contain the constitution-mandated defect, cost, approval, and
  mitigation evidence.

Never reuse an existing benchmark ID after materially changing its fixture. Increment `_vN` and
establish a new baseline.

## 9. Release Readiness

A release candidate is ready only when all conditions hold:

- MSRV and current-stable builds pass.
- Linux, macOS, and Windows conformance jobs pass.
- The parity report is releasable with 17/301/1,406 complete coverage.
- Every non-representable case is explicitly approved and has replacement evidence where possible.
- Differential tests required by the change pass against the pinned oracle.
- The hosted CodSpeed check has no unexplained regression.
- The crate package contains the project license, pinned Node license, and all provenance notices.

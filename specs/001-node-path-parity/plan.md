# Implementation Plan: Node.js Path Behavioral Parity

**Branch**: `001-node-path-parity` (feature identifier; no Git branch hook ran) | **Date**:
2026-08-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-node-path-parity/spec.md`

**Note**: This plan completes Spec Kit Phase 0 research and Phase 1 design. Task decomposition and
implementation are performed by later commands.

## Summary

Build a Rust 2024 library whose POSIX, Windows, and host-default path operations reproduce Node.js
`lib/path.js` at commit `3f42cfacf27e348297a52d89b4cdc48b35cb7559`. Implement path algorithms over
Unicode strings rather than native filesystem paths, make cwd/drive/Node-host inputs explicit,
mirror all 17 upstream path-test files in Rust, and require complete assertion-level provenance
before release. Establish CodSpeed CPU Simulation benchmarks only after conformance is fully green.

The design keeps core parsing dependency-free and safe. The only planned runtime dependency is an
exactly pinned ECMAScript regex engine used to reproduce minimatch 10.2.5 for `matchesGlob`; generic
glob crates are not treated as compatible. Slice-only operations return borrowed data, transforming
operations return `Cow` or `String`, and no v1 operation performs filesystem I/O or mutates process
state.

## Technical Context

**Language/Version**: Rust edition 2024, Cargo resolver 3, MSRV 1.88.0; Rust 1.97.1 pinned for the
initial reference and CodSpeed environment

**Primary Dependencies**: Runtime `regress = 0.11.1` exactly pinned with `std`, `utf16`, and
`prohibit-unsafe`; path parsing/normalization otherwise dependency-free. Dev-only Serde/Serde JSON
for evidence, proptest for generated invariants, and `codspeed-divan-compat = 5.0.1` renamed to
`divan`; all resolved versions locked in `Cargo.lock`

**Storage**: No runtime persistence. Checked-in JSON baseline/ledger and upstream notices; generated
parity summaries under `target/`; benchmark history stored by CodSpeed

**Testing**: Rust unit tests; one-to-one ported integration modules for 17 Node files; central
ledger/report runner; generated/property cases; opt-in differential tests against the pinned Node
oracle; Linux/macOS/Windows CI

**Target Platform**: Cross-platform Rust library for Linux, macOS, and Windows with explicit POSIX
and Windows semantics available on every host

**Project Type**: Single Rust library crate with test-only oracle/report tooling and one benchmark
target

**Performance Goals**: Every path-processing operation covered under explicit POSIX and Windows
namespaces; stable short (up to 64 bytes), long (about 1 KiB), clean, dirty, structural, and Unicode
cases; zero unexplained CodSpeed regressions above the initial global 10% threshold

**Constraints**: Node behavior is authoritative; compatibility blocks release; initial input domain
is valid Unicode scalar strings; deterministic calls use explicit immutable context; safe Rust only;
no v1 global cache, filesystem I/O, process mutation, or Node runtime dependency

**Scale/Scope**: 12 public path operations, two explicit namespaces plus host-default selection,
two constants per namespace, 17 upstream files, 301 assertion sites, and 1,406 expanded cases

## Constitution Check

*GATE: Passed before Phase 0 and re-checked after Phase 1 design.*

### Pre-Research Gate

| Principle or standard | Planned evidence | Status |
|-----------------------|------------------|--------|
| I. Node.js Behavioral Parity | Pinned source revision, public API contract, complete ported corpus, zero known parity failures | PASS |
| II. Explicit POSIX and Windows Semantics | Separate namespace algorithms and explicit Node execution context | PASS |
| III. Oracle-Driven Conformance Testing | All upstream files inventoried; every assertion mapped in a machine-checkable ledger | PASS |
| IV. Measured Performance | CodSpeed baseline begins only after conformance and blocks unexplained regressions | PASS |
| V. Safe, Allocation-Conscious Rust | Safe baseline, borrowed outputs, minimal dependencies, no initial `unsafe` | PASS |
| Compatibility baseline | Node commit and source/test provenance recorded in design artifacts | PASS |
| Development workflow | Tests and parity matrix precede implementation; performance follows conformance | PASS |

### Post-Design Gate

| Principle or standard | Concrete Phase 1 evidence | Status |
|-----------------------|---------------------------|--------|
| I. Node.js Behavioral Parity | [Rust API contract](contracts/rust-api.md) enumerates every method, namespace, mapping, error, and boundary | PASS |
| II. Explicit POSIX and Windows Semantics | `PathContext` separates `NodeHost` from namespace; `_with_context` results never depend on actual Rust host | PASS |
| III. Oracle-Driven Conformance Testing | [Ledger schema](contracts/parity-ledger.schema.json) covers 17/301/1,406; summary schema makes failures machine-checkable | PASS |
| IV. Measured Performance | Divan/CodSpeed design pins runner, compiler, IDs, fixtures, comparison policy, and compatibility-first baseline gate | PASS |
| V. Safe, Allocation-Conscious Rust | Borrowed/Cow public results, `regress` `prohibit-unsafe`, no cache, and measured output allocation | PASS |
| Compatibility and performance standards | Data model pins commit/blob/hash provenance; quickstart verifies full parity before benchmark acceptance | PASS |
| Development workflow | Design orders baseline/tests before behavior, then differential verification, then CodSpeed | PASS |

No implementation exception or constitution violation is accepted.

**Governance metadata warning**: the existing constitution Sync Impact Report says `1.0.0 -> 1.0.1`
but its footer still declares version `1.0.0`. This does not change the normative principles used by
the plan, but it must be corrected separately through `$speckit-constitution` before release.

## Architecture

### 1. Public Namespace Layer

`src/lib.rs` exposes the crate-root host-default facade and shared types. `src/posix.rs` and
`src/win32.rs` expose explicit semantics on every target. The root selects Windows only on Windows
and POSIX elsewhere, matching pinned Node.

Context-free functions delegate directly to namespace algorithms. Environment-backed functions
snapshot a `PathContext` and then delegate to the same pure context-taking implementation used by
tests and benchmarks.

### 2. Shared String-Algorithm Layer

`src/shared.rs` contains ASCII separator, drive, dot-segment, and normalization helpers shared only
when sharing does not obscure namespace differences. Algorithms scan UTF-8 safely while treating
Node control characters as ASCII. No helper uses `std::path` semantics.

Return ownership follows [the public contract](contracts/rust-api.md): slice-only queries borrow,
identity-capable transforms use `Cow`, and compositional transforms allocate `String` with
capacity planned from input lengths.

### 3. Deterministic Context Layer

`src/context.rs` models Node host, cwd, and Windows drive-specific cwd. Pure algorithms receive an
immutable context. Environment adapters validate Unicode and snapshot hidden Windows `=C:` entries
without global mutation. Empty cwd remains representable for the pinned Node safety regression.

This layer also resolves pinned `matchesGlob` host sensitivity: namespace controls glob platform,
while explicit `NodeHost` controls case behavior. Identical input plus context is host-independent.

### 4. Parse/Format Layer

`src/path_object.rs` defines generic owned/borrowed `PathObject` values. `parse` returns input slices;
`format` accepts any string-like fields and applies Node's `dir`/`root` and `base`/`name+ext`
precedence plus extension-dot behavior.

### 5. Glob Compatibility Layer

`src/glob/` ports the pinned minimatch 10.2.5 and brace-expansion compilation pipeline and applies
the exact Node option set. Pattern length is counted in UTF-16 code units. Generated ECMAScript
regex is evaluated through exact `regress` 0.11.1 UTF-16/UCS-2 modes with unsafe prohibited.

The 20 pinned path-glob rows are the minimum corpus, not the language definition. Differential and
generated tests cover braces, extglobs, globstar, character classes, separator rules, case modes,
limits, malformed patterns, and astral Unicode.

### 6. Conformance Evidence Layer

Each upstream `test-path*.js` file has one Rust counterpart. Pure string/object cases preserve
inputs, expectations, and comparators. Harness adaptations are limited to deterministic filename,
cwd, drive cwd, fixtures, host branch, and table-loop setup. Windows-only explicit cases run on all
hosts.

The central runner validates the checked-in ledger and produces `target/parity-summary.json` before
its final assertion. Release requires 17 files, 301 static sites, 1,406 expanded cases, no missing or
orphan cases, all representable cases passing, and every non-representable boundary approved.

### 7. Performance Layer

One `path_ops` Divan target benchmarks explicit POSIX and Windows functions using fixed contexts.
Benchmark IDs follow `path_ops/{namespace}/{operation}/{semantic_case}_vN`; semantic fixture changes
create a new versioned ID. Setup is outside measured closures, while result allocation remains
inside. Node subprocesses and process-context capture are never benchmarked.

GitHub runs CPU Simulation on Ubuntu 24.04/Rust 1.97.1 through `CodSpeedHQ/action@v5` with OIDC.
Cross-platform correctness remains in the separate normal CI matrix.

## Project Structure

### Documentation (this feature)

```text
specs/001-node-path-parity/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/
│   └── requirements.md
├── contracts/
│   ├── rust-api.md
│   ├── parity-ledger.schema.json
│   └── parity-summary.schema.json
└── tasks.md                             # Created later by $speckit-tasks
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
LICENSE
UPSTREAM_LICENSE_NODE.txt
THIRD_PARTY_NOTICES.md

src/
├── lib.rs                               # Host-default facade and public exports
├── context.rs                           # NodeHost, cwd, and drive-cwd snapshots
├── error.rs                             # ContextError and GlobError
├── path_object.rs                       # Borrowed/owned parse and format records
├── posix.rs                             # POSIX public contract and algorithms
├── win32.rs                             # Windows public contract and algorithms
├── shared.rs                            # Safe allocation-conscious string helpers
└── glob/
    ├── mod.rs                           # matchesGlob facade and option mapping
    ├── brace.rs                         # Pinned brace-expansion behavior
    └── minimatch.rs                     # Pinned minimatch compilation behavior

tests/
├── node_path.rs                         # Ledger-aware aggregate runner/report writer
├── differential.rs                      # Opt-in pinned-Node oracle comparison
├── oracle/
│   └── path-oracle.mjs                  # Test-only Node request/response adapter
├── node_path/
│   ├── support.rs                       # Cases, deterministic context, ledger/report helpers
│   ├── test_path.rs
│   ├── test_path_basename.rs
│   ├── test_path_dirname.rs
│   ├── test_path_extname.rs
│   ├── test_path_glob.rs
│   ├── test_path_isabsolute.rs
│   ├── test_path_join.rs
│   ├── test_path_makelong.rs
│   ├── test_path_normalize.rs
│   ├── test_path_parse_format.rs
│   ├── test_path_posix_exists.rs
│   ├── test_path_posix_relative_on_windows.rs
│   ├── test_path_relative.rs
│   ├── test_path_resolve.rs
│   ├── test_path_win32_exists.rs
│   ├── test_path_win32_normalize_device_names.rs
│   └── test_path_zero_length_strings.rs
└── fixtures/
    └── node-path/
        ├── baseline.json                 # Commit, blob OIDs, inventory totals
        └── parity-ledger.json            # Expanded-case provenance and lifecycle

benches/
├── path_ops.rs                           # Divan/CodSpeed benchmark entry point
└── cases.rs                              # Stable fixed inputs and contexts

.github/workflows/
├── ci.yml                                # MSRV and Linux/macOS/Windows conformance
└── codspeed.yml                          # Single pinned CPU Simulation job
```

**Structure Decision**: Use one library crate with explicit semantic modules, one shared internal
core, a contained glob compatibility subsystem, tests mirroring upstream file boundaries, and one
benchmark target. A workspace, service layer, CLI, or runtime persistence would add complexity
without improving a standalone path library.

## Implementation Strategy

### Stage A: Reproducible Bootstrap and Evidence Skeleton

- Create crate metadata, MSRV/current toolchain checks, locked dependencies, notices, and CI shells.
- Materialize the baseline manifest from pinned Git objects and generate the 17/301/1,406 inventory.
- Implement ledger parsing/validation and report output before path cases are marked passing.
- Prove `regress` with required features builds on Rust 1.88; failure blocks later stages. The
  implementation gate raised the planned 1.85 MSRV after exact `regress` 0.11.1 was verified to use
  let chains, which became stable in Rust 1.88.

### Stage B: Context-Free POSIX Behavior

- Implement separators/constants, normalization, absoluteness, join, dirname, basename, extname,
  parse, and format.
- Port the corresponding POSIX vectors first and keep their ledger entries pending until passing.
- Add allocation assertions or benchmark probes only after semantic cases are green.

### Stage C: Context-Free Windows Behavior

- Implement drive, UNC, device, namespaced, mixed-separator, reserved-name, and traversal rules.
- Run all explicit Windows vectors on all hosts, including the 78 formerly Windows-only reserved
  name cases and CVE-2024-36139 normalization regressions.

### Stage D: Contextual Operations and Host-Default Facade

- Implement deterministic resolve/relative/toNamespacedPath using supplied cwd and drive mappings.
- Add environment snapshot adapters and target-selected root exports.
- Port cwd, drive-cwd, host-default, empty-cwd, fixture, and namespaced-path cases without global
  state mutation.

### Stage E: Exact Glob Compatibility

- Port pinned brace/minimatch compilation and Node option mapping.
- Integrate UTF-16/UCS-2 regex execution, limits, and errors.
- Port path-glob cases and add generated/differential coverage for the larger glob language.

### Stage F: Complete Corpus and Boundary Review

- Finish all one-to-one test modules and validate exact upstream comparators and Unicode cases.
- Review the provisional 29-site/420-expanded non-representable set; require explicit approval and
  replacement type/export checks.
- Reach a releasable parity report with zero failures, pending, stale, missing, or orphan entries.

### Stage G: Benchmark Baseline and Optimization

- Add the complete stable Divan matrix only after Stage F is green.
- Establish CodSpeed baseline on the pinned compiler/runner and make its check required.
- Optimize hot paths iteratively; each change reruns conformance before performance comparison.
- Any compatibility-required slowdown follows the constitution exception process.

## Phase 0 Output

[research.md](research.md) resolves all toolchain, API, context, glob, test-port, ledger, licensing,
and CodSpeed decisions. All research questions have concrete decisions.

## Phase 1 Outputs

- [data-model.md](data-model.md): runtime values, baseline/case evidence, state transitions, summary,
  and benchmark entities.
- [contracts/rust-api.md](contracts/rust-api.md): public namespace, ownership, context, errors, and
  `matchesGlob` contract.
- [contracts/parity-ledger.schema.json](contracts/parity-ledger.schema.json): checked-in source and
  case provenance contract.
- [contracts/parity-summary.schema.json](contracts/parity-summary.schema.json): generated run and
  release-readiness contract.
- [quickstart.md](quickstart.md): end-to-end MSRV, conformance, differential, and CodSpeed validation.

## Complexity Tracking

No constitution violation or unjustified project layer is planned. The internal minimatch compiler
and one exact ECMAScript regex dependency are necessary because `matchesGlob` is in the required
public surface and no generic Rust glob dialect matches the pinned Node contract.

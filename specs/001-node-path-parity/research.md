# Phase 0 Research: Node.js Path Behavioral Parity

## Baseline and Evidence

The compatibility baseline is Node.js commit
`3f42cfacf27e348297a52d89b4cdc48b35cb7559` (Node `27.0.0` development tree). The authoritative
implementation is [`lib/path.js`](https://github.com/nodejs/node/blob/3f42cfacf27e348297a52d89b4cdc48b35cb7559/lib/path.js).
Research inspected Git objects at that commit rather than relying on mutable working-tree content.

The pinned public-path test corpus is the 17 files matching `test/parallel/test-path*.js`. It has
1,302 source lines, 301 static assertion sites, and a provisional 1,406 executable case instances
after tables, loops, namespaces, and mutually exclusive host branches are expanded. These totals
become generated, blob-verified metadata during implementation rather than permanent hand-maintained
truth.

## Decision 1: Rust Toolchain and Dependency Policy

**Decision**: Use Rust edition 2024, Cargo resolver 3, and `rust-version = "1.88"`. Test both the
MSRV and current stable. Pin Rust `1.97.1` for the initial CodSpeed baseline. Treat an MSRV increase
as a minor-version compatibility change.

Keep parsing and normalization free of runtime dependencies. For `matchesGlob`, use the exact
dependency `regress = "=0.11.1"` with default features disabled and features `std`, `utf16`, and
`prohibit-unsafe`. Port the pinned minimatch/brace-expansion compilation rules internally; do not
delegate semantics to a generic glob crate. `regress` MUST pass the Rust 1.88 CI job before the
dependency is accepted.

Use dev-only dependencies for serialization/reporting, property or generated testing, and
benchmarking. Commit `Cargo.lock` because the benchmark compiler and dependency graph are part of
the reproducible performance baseline.

**Rationale**: Rust 1.85 is the first Rust 2024 release and the selected CodSpeed adapter supports
it, but implementation validation proved that exact `regress` 0.11.1 uses let-chain syntax, which
became stable in Rust 1.88. Rust 1.88 is therefore the lowest toolchain that can satisfy the exact
dependency and feature contract without maintaining a fork. A current pinned compiler keeps
CodSpeed comparisons stable without making it the consumer MSRV. Pinned Node minimatch 10.2.5
generates ECMAScript regex with UTF-16/UCS-2 semantics and lookarounds; ordinary Rust glob and regex
crates do not provide the same language.

**Alternatives considered**:

- Latest-stable-only MSRV: rejected because it needlessly narrows adoption.
- Rust edition 2021: rejected because no compatibility need justifies starting a new library on the
  previous edition.
- `globset`, `glob`, or `wax` as the semantic implementation: rejected because their dialect,
  separators, malformed-pattern handling, extglobs, braces, and Unicode behavior differ.
- Embedding or spawning Node at runtime: rejected because it breaks portability and performance.

Sources: [Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html),
[Cargo rust-version](https://doc.rust-lang.org/cargo/reference/rust-version.html),
[Rust 1.97.1](https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/), and
[`regress` 0.11.1](https://docs.rs/regress/0.11.1/regress/).

## Decision 2: Public API and Ownership

**Decision**: Expose `node_path::posix`, `node_path::win32`, and crate-root host-default functions.
Use Rust naming while documenting the Node mapping:

- `resolve`, `normalize`, `is_absolute`, `join`, `relative`
- `to_namespaced_path`, `dirname`, `basename`, `extname`
- `format`, `parse`, `matches_glob`
- deprecated `_make_long` for Node `_makeLong`
- `SEP` and `DELIMITER` as `&'static str`

Inputs are Unicode `&str`; variadic Node arguments map to `&[&str]`. Use borrowed or conditionally
borrowed results when Node semantics permit them:

- `normalize` and `to_namespaced_path`: `Cow<'a, str>`
- `dirname`, `basename`, and `extname`: `&'a str`
- `parse`: `PathObject<&'a str>`
- `join`, `resolve`, `relative`, and `format`: `String`
- `is_absolute`: `bool`
- `matches_glob`: `Result<bool, GlobError>`

Use one generic record:

```rust
pub struct PathObject<S = String> {
    pub root: S,
    pub dir: S,
    pub base: S,
    pub ext: S,
    pub name: S,
}
```

Empty values represent missing/falsey Node format fields within the declared string-only input
domain. Dynamic JavaScript values remain explicit representation-boundary cases; the Rust API does
not add an `Any`-like input solely to reproduce JavaScript `TypeError` checks.

**Rationale**: Modules and root selection closely map Node's namespace model. String types avoid
host-native filesystem behavior, while borrowed results remove allocations from slice-only
operations without changing returned text.

**Alternatives considered**:

- `Path`, `PathBuf`, or `OsStr`: rejected because they impose host-native encoding and semantics.
- Always returning `String`: rejected because basename/dirname/extname/parse can be borrowed safely.
- Separate owned and borrowed parse/format structures: rejected until usage demonstrates a need.

## Decision 3: Deterministic Node Execution Context

**Decision**: Make environment state explicit in the normative conformance API:

```rust
pub enum NodeHost { Win32, Darwin, OtherPosix }

pub struct DriveCwd<'a> {
    pub device: &'a str,
    pub cwd: &'a str,
}

pub struct PathContext<'a> {
    pub host: NodeHost,
    pub cwd: &'a str,
    pub drive_cwds: &'a [DriveCwd<'a>],
}
```

Provide pure context-taking variants for `resolve`, `relative`, Windows `to_namespaced_path`, and
`matches_glob`. Context-backed explicit namespace calls are the parity oracle. Crate-root convenience
functions snapshot the real process context and return `ContextError` when current-directory access
or Unicode conversion fails.

Windows drive lookup is ASCII-case-insensitive and follows Node's fallback behavior. Tests construct
contexts and MUST NOT mutate global cwd or environment. The Windows environment adapter enumerates
`vars_os()` to find hidden `=C:` entries because direct `var_os("=C:")` lookup is not portable.

**Rationale**: Node behavior depends on cwd, hidden Windows drive cwd values, and the Node execution
host. Explicit context makes those inputs reproducible and keeps POSIX/Windows algorithms independent
of the actual Rust test host.

This also resolves a subtle constitution gate: pinned `matchesGlob` sets case behavior from the Node
host even when the namespace is explicitly POSIX or Windows. With `NodeHost` supplied, identical
path inputs and identical context produce identical results on every Rust host.

**Alternatives considered**:

- Read process state inside core algorithms: rejected as nondeterministic and difficult to test.
- Mutate cwd/environment around tests: rejected because the state is process-global and race-prone.
- Ignore Windows drive-specific cwd: rejected as a known `resolve` parity failure.

Sources: [`current_dir`](https://doc.rust-lang.org/std/env/fn.current_dir.html),
[`vars_os`](https://doc.rust-lang.org/std/env/fn.vars_os.html), and
[`set_var` safety](https://doc.rust-lang.org/std/env/fn.set_var.html).

## Decision 4: Exact `matchesGlob` Pipeline

**Decision**: Reproduce the pinned minimatch 10.2.5 pipeline, not merely the 20 string cases in
`test-path-glob.js`:

1. Port pinned minimatch and brace-expansion compilation behavior and retain applicable notices.
2. Apply Node's exact options: `windowsPathsNoEscape`, `nonegate`, `nocomment`, optimization level 2,
   namespace-selected platform, host-selected `nocase`, and `nocaseMagicOnly`.
3. Preserve the 65,536 UTF-16-code-unit pattern limit and pinned expansion/recursion caps.
4. Compile generated ECMAScript regex through `regress` using UTF-16 source units.
5. Select UCS-2 or UTF-16 matching according to the generated regex flags.
6. Differential-test glob syntax families and randomized Unicode inputs against the pinned oracle.

`GlobError` exposes observable resource errors such as an overlong pattern. Malformed minimatch
syntax retains pinned literal/non-match behavior instead of introducing a new Rust-only parse error.

**Rationale**: Node delegates `path.matchesGlob` to internal minimatch. Similar-looking Rust glob
dialects can pass the 20 positive/negative rows and still be observably incompatible for braces,
extglobs, case handling, astral characters, or malformed patterns.

**Alternatives considered**:

- Implement only the path test vectors: rejected as insufficient evidence for the advertised API.
- Rust `regex`: rejected because it lacks required lookaround and non-`u` JavaScript semantics.
- A handwritten matcher: potentially valuable later, but too risky for the first compatible release.

Sources: [pinned Node glob wrapper](https://github.com/nodejs/node/blob/3f42cfacf27e348297a52d89b4cdc48b35cb7559/lib/internal/fs/glob.js#L932-L943),
[pinned path glob tests](https://github.com/nodejs/node/blob/3f42cfacf27e348297a52d89b4cdc48b35cb7559/test/parallel/test-path-glob.js),
and [pinned minimatch package](https://github.com/nodejs/node/blob/3f42cfacf27e348297a52d89b4cdc48b35cb7559/deps/minimatch/package.json).

## Decision 5: Upstream Test Inventory and Porting Rules

**Decision**: Track both static source coverage and expanded executable-case coverage. The initial
verified planning inventory is:

| Upstream file | Assertion sites | Expanded cases | Main behavior |
|---------------|----------------:|---------------:|---------------|
| `test-path-basename.js` | 61 | 61 | Basename and suffix removal |
| `test-path-dirname.js` | 45 | 45 | Roots, drives, UNC, streams |
| `test-path-extname.js` | 17 | 145 | Dot and separator edge cases |
| `test-path-glob.js` | 3 | 22 | Glob dialect and dynamic types |
| `test-path-isabsolute.js` | 22 | 22 | POSIX, drive, and UNC absoluteness |
| `test-path-join.js` | 1 | 183 | Join, normalization, traversal regressions |
| `test-path-makelong.js` | 34 | 34 | Namespaced/device paths and dynamic pass-through |
| `test-path-normalize.js` | 66 | 66 | Normalization and security regressions |
| `test-path-parse-format.js` | 18 | 486 | Fields, precedence, round trips, invalid types |
| `test-path-posix-exists.js` | 1 | 1 | CommonJS module identity |
| `test-path-posix-relative-on-windows.js` | 1 | 1 | POSIX relative with implicit cwd |
| `test-path-relative.js` | 1 | 40 | Relative, drives, UNC, Unicode case behavior |
| `test-path-resolve.js` | 4 | 25 | Resolve, cwd, drive-specific cwd |
| `test-path-win32-exists.js` | 1 | 1 | CommonJS module identity |
| `test-path-win32-normalize-device-names.js` | 3 | 78 | Reserved names and traversal |
| `test-path-zero-length-strings.js` | 16 | 16 | Empty input and cwd |
| `test-path.js` | 7 | 180 | Dynamic types, constants, host identity |
| **Total** | **301** | **1,406** | |

Release gates require 17/17 files, 301/301 assertion sites, 1,406/1,406 expanded cases classified,
zero unclassified cases, and every representable case passing. Each counterpart retains the exact
inputs and expected observable result. Only syntax, loop structure, fixture path, deterministic
context, or host harness setup may be adapted.

Known harness adaptations include `__filename`, cwd, hidden Windows drive cwd, local fixture paths,
host-default selection, Windows-only upstream guards, and table-driven aggregate collectors. Explicit
Windows test vectors run on every supported host rather than retaining upstream host skips.

The provisional non-representable set is 29 assertion sites and 420 expanded instances: dynamic
non-string failures, non-string `toNamespacedPath` identity/pass-through, JavaScript runtime field
type checks, CommonJS module identity, and exact JavaScript error metadata. Each requires an approved
boundary explanation and an equivalent Rust type/export check where one exists.

**Rationale**: Source-site counts detect omitted helpers/files; expanded counts detect omitted table
rows or namespace permutations. Deterministic harness adaptation preserves semantic evidence while
removing dependency on Node's test runner and the developer's machine.

**Alternatives considered**:

- Count only source assertions: rejected because one join assertion represents 183 outcomes.
- Count only expanded rows: rejected because helper-level contracts could disappear unnoticed.
- Execute JavaScript tests through a shim: rejected because it does not directly test the Rust API.

## Decision 6: Parity Ledger, Provenance, and State

**Decision**: Assign IDs containing the baseline short commit, upstream file, vector/helper name,
stable index, namespace, and contract dimension, for example:

```text
node-path@3f42cfac/test-path-extname/testPaths/000/posix
node-path@3f42cfac/test-path-parse-format/winPaths/000/field-root
```

Record the full repository URL and commit, per-file blob OID, assertion/helper line, vector line,
case index, operation, namespace, canonical arguments/context, expected result and comparator,
content hash, local test ID, disposition, verification, adaptation/boundary rationale, approval, and
license notice.

Keep treatment and lifecycle orthogonal:

- `disposition`: `ported`, `harness-adapted`, `non-representable`
- `verification`: `pending`, `passing`, `failing`, `proposed`, `approved`, `stale`, `rejected`

The accepted specification statuses are derived combinations: ported/passing,
harness-adapted/passing, or non-representable/approved. Pending, failing, stale, proposed, rejected,
missing, or orphan entries fail the release gate. A baseline update marks prior entries stale and
requires reconciliation; removed cases remain in historical reports.

**Rationale**: Lines alone cannot identify loop-expanded cases. Separating disposition from
verification supports implementation progress, failure reporting, approval, and baseline refreshes
without weakening final status rules.

**Alternatives considered**:

- A single three-value status field: rejected because it cannot represent pending/failing work.
- Source-line-only IDs: rejected because one line may expand into hundreds of cases.

## Decision 7: Test and Report Architecture

**Decision**: Mirror the 17 upstream file boundaries with Rust modules under `tests/node_path/` and
use a central `tests/node_path.rs` runner. Each module returns named case results; the central runner
validates ledger coverage, writes `target/parity-summary.json`, and fails if any release gate is not
met. `target/` output is generated evidence and is never treated as checked-in truth.

Run explicit POSIX and Windows cases on Linux, macOS, and Windows. Run host-default adapter smoke
tests in the matching host jobs. Use constructed `PathContext` fixtures; no tests mutate cwd or the
environment. Add property and differential suites for normalization, parsing, and globbing. The
pinned Node executable is an opt-in oracle/update dependency, not a runtime or normal test dependency.

The parity summary contains baseline identity, inventory counts, disposition/verification counts,
failure IDs, exclusion approvals, orphan/missing IDs, and a final releasable boolean. Its format is
defined separately from the source ledger.

**Rationale**: One-to-one module boundaries keep source provenance visible, while a central runner
can aggregate all case outcomes into the machine-checkable report required by the specification.

**Alternatives considered**:

- Independent uncoordinated `#[test]` functions only: rejected because they do not naturally produce
  a complete ledger-aware report.
- Requiring Node for every test run: rejected because normal development must remain self-contained.

## Decision 8: CodSpeed Harness and Regression Policy

**Decision**: Use Divan through the current CodSpeed compatibility package:

```toml
[dev-dependencies]
divan = { package = "codspeed-divan-compat", version = "5.0.1" }

[[bench]]
name = "path_ops"
harness = false
```

Install `cargo-codspeed` 5.0.1 as a developer/CI tool. Use CPU Simulation with allocation costs
included. Run one benchmark workflow on Ubuntu 24.04 and Rust 1.97.1 with OIDC and
`CodSpeedHQ/action@v5`; keep Linux/macOS/Windows conformance in the normal test matrix.

Create the first baseline only after conformance passes. Start with CodSpeed's documented global
10% regression threshold, require the `CodSpeed Performance Analysis` check, and tune per-benchmark
thresholds after stable repeated data exists. Compatibility-required slowdowns follow the exception
process in the constitution.

Use stable IDs:

```text
path_ops/{posix|win32}/{operation}/{semantic_case}_v1
```

Cover every path-processing operation with applicable short (at most 64 bytes), long (about 1 KiB),
clean, normalization-required, root/drive/UNC, Unicode, suffix/dot, and glob hit/miss categories.
Benchmark pure context-taking operations so system calls and Node subprocesses are outside the
measured closure. Version an ID when its fixture meaning changes.

**Rationale**: CodSpeed recommends Divan for new Rust suites and CPU Simulation for deterministic
CPU/allocation-bound code. One pinned environment avoids compiler or runner drift and duplicate
benchmark identities.

**Alternatives considered**:

- Criterion compatibility: appropriate for an existing Criterion suite, but there is none.
- Walltime: rejected for deterministic in-process string algorithms.
- Aggregate benchmarks: rejected because they obscure the regressing operation.
- Rust-versus-Node timing: rejected; Node is the semantic oracle, while CodSpeed compares Rust
  revisions.

Sources: [CodSpeed Rust](https://codspeed.io/docs/benchmarks/rust),
[Divan integration](https://codspeed.io/docs/benchmarks/rust/divan),
[CPU Simulation](https://codspeed.io/docs/instruments/cpu),
[GitHub Actions integration](https://codspeed.io/docs/integrations/ci/github-actions), and
[CodSpeed performance checks](https://codspeed.io/docs/features/performance-checks).

## Decision 9: Upstream Notices

**Decision**: Keep the project MIT license and add the pinned Node license as
`UPSTREAM_LICENSE_NODE.txt` plus an upstream provenance notice. Every derived test module references
the repository and commit. Preserve the full Joyent/Node header in the three counterparts derived
from `test-path-makelong.js`, `test-path-parse-format.js`, and `test-path.js`. Any ported minimatch or
brace-expansion material retains its applicable Blue Oak/MIT notices. Cargo packaging includes all
required notices.

**Rationale**: Copied test vectors and any ported matcher logic are distributed derived material;
central licenses plus file-level provenance make their origin auditable.

**Alternative considered**: Rely on the project's own MIT file only. Rejected because it omits
upstream copyright and permission notices.

## Resolved Risks and Follow-up Warnings

- **Node host versus namespace**: resolved by first-class `NodeHost` in the normative context API.
- **Windows drive cwd**: resolved by explicit case-insensitive drive mappings and an environment
  adapter tested separately.
- **UTF-16 glob semantics**: resolved by UTF-16 counting/source plus `regress` UTF-16/UCS-2 modes.
- **Test-count drift**: resolved by blob OIDs, generated inventories, content hashes, and dual counts.
- **Performance reproducibility**: resolved by pinned compiler/runner/dependencies and stable IDs.
- **ReDoS/hostile glob patterns**: exact pinned behavior is retained; adding stricter resource limits
  would be a contract change and needs a future specification.
- **Dependency MSRV**: the exact `regress` version has no declared MSRV and uses let chains; the
  empirically verified Rust 1.88 minimum is a mandatory bootstrap/CI gate.
- **Constitution metadata**: the existing Sync Impact Report says `1.0.0 -> 1.0.1`, while the footer
  still declares `1.0.0`. This is outside the planning command and must be corrected with
  `$speckit-constitution`; it does not change the principles used by this design.

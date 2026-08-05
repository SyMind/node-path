# Feature Specification: Node.js Path Behavioral Parity

**Feature Branch**: `N/A (no branch hook configured)`

**Created**: 2026-08-04

**Status**: Draft

**Input**: User description: "Implement a high-performance Rust path library with the same
behavior as Node.js `path`, directly port the related Node.js path tests to the Rust project, and
ensure all applicable tests pass."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Use Node-Compatible Path Operations (Priority: P1)

As a library adopter, I can process POSIX and Windows path strings and receive the same observable
results as the pinned Node.js `path` behavior, regardless of the operating system running my code.

**Why this priority**: Behavioral compatibility is the product's primary purpose. Without it, the
library cannot safely replace Node.js path handling in cross-language systems.

**Independent Test**: Select any supported operation and namespace, run the same inputs and
environmental context against the library and the pinned Node.js baseline, and compare the
observable results.

**Acceptance Scenarios**:

1. **Given** a supported path operation, namespace, input, and current-directory context, **When**
   the operation is evaluated by both products, **Then** their returned path data is identical.
2. **Given** Windows path inputs on a non-Windows host, **When** the explicit Windows namespace is
   used, **Then** the results remain identical to Node.js `path.win32`.
3. **Given** POSIX path inputs on a Windows host, **When** the explicit POSIX namespace is used,
   **Then** the results remain identical to Node.js `path.posix`.
4. **Given** an input that the declared Rust API can represent but Node.js rejects, **When** it is
   processed, **Then** the library reports an equivalent failure rather than a different path.

---

### User Story 2 - Verify Parity from Upstream Tests (Priority: P2)

As a maintainer, I can run a Rust-side conformance corpus directly derived from the pinned Node.js
path tests and see which upstream cases pass, were adapted only for harness differences, or cannot
be represented by the declared Rust API.

**Why this priority**: Direct test provenance turns compatibility from an assertion into auditable
evidence and minimizes omissions when Node.js exercises unusual path behavior.

**Independent Test**: Inspect the upstream-to-project test ledger, verify that every assertion in
the 17 pinned `test-path*.js` files is accounted for, and run every in-scope ported case.

**Acceptance Scenarios**:

1. **Given** a Node.js test assertion about public path behavior, **When** it is moved into the
   project conformance corpus, **Then** its inputs and expected result are preserved and it passes.
2. **Given** a test that depends on the Node.js test harness or host setup, **When** it is adapted,
   **Then** only the setup changes and the path inputs and expected observable result remain intact.
3. **Given** a JavaScript-only value or runtime identity check that the declared Rust API cannot
   represent, **When** the corpus is reviewed, **Then** it is recorded as a representation boundary
   with its source, rationale, and any equivalent Rust contract check.
4. **Given** the complete pinned upstream corpus, **When** the parity report is produced, **Then**
   there are no unclassified files or assertions and no failing in-scope cases.

---

### User Story 3 - Protect Path-Processing Performance (Priority: P3)

As a maintainer, I can compare representative path-processing workloads with an accepted baseline
and detect regressions without weakening compatibility coverage.

**Why this priority**: Performance is the second project priority, but optimization is meaningful
only after the behavioral contract is satisfied.

**Independent Test**: Starting from a passing conformance baseline, run the established workload
set and compare every affected operation with its accepted result and performance baseline.

**Acceptance Scenarios**:

1. **Given** a change to a path-processing hot path, **When** its representative workloads run,
   **Then** the comparison reports no unexplained regression under the project policy.
2. **Given** an optimization that changes any ported test result, **When** the full conformance
   corpus runs, **Then** the change is rejected even if performance improves.
3. **Given** a compatibility correction with a measured slowdown, **When** it is proposed, **Then**
   the report identifies the corrected case, measured cost, approval, and mitigation follow-up.

### Edge Cases

- Empty strings, omitted path lists where the operation permits them, `.` and `..` segments, paths
  consisting only of separators, repeated separators, and trailing separators.
- POSIX root paths, Windows drive-relative and drive-absolute paths, mixed drive-letter case,
  drive-specific current directories, UNC shares, device paths, and namespaced paths.
- Mixed forward and backward separators, including cases where a backslash is data under POSIX
  rules but a separator under Windows rules.
- Reserved Windows device names, colons used as stream markers, question marks, and other inputs
  covered by the pinned Windows normalization tests.
- Basenames with full, partial, absent, repeated, or dot-only suffixes; extensions involving leading
  dots, multiple dots, directory dots, and trailing separators.
- Parse/format objects with absent or competing fields, including root, directory, base, name, and
  extension precedence.
- Relative and resolved paths whose result depends on current directory or host-default namespace.
- Glob patterns with separators, wildcard syntax, invalid argument categories, and platform-specific
  matching behavior.
- Valid Unicode path strings. JavaScript values that cannot be represented in the declared Rust
  input domain, including unpaired UTF-16 surrogate code units, require an explicit ledger entry.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The library MUST provide behaviorally equivalent contracts for Node.js path
  operations `resolve`, `normalize`, `isAbsolute`, `join`, `relative`, `toNamespacedPath`,
  `dirname`, `basename`, `extname`, `format`, `parse`, and `matchesGlob`.
- **FR-002**: The library MUST expose explicit POSIX and Windows variants equivalent to
  `path.posix` and `path.win32`, including their `sep` and `delimiter` values.
- **FR-003**: The library MUST provide host-default behavior equivalent to Node.js platform
  selection and MUST retain the deprecated `_makeLong` behavior when claiming full API parity.
- **FR-004**: For every input in the declared Rust domain, operation results MUST match the pinned
  Node.js baseline for content, separator choice, trailing-separator handling, normalization, and
  structured parse data.
- **FR-005**: Operations that depend on a current directory or Windows drive-specific context MUST
  accept or obtain an equivalent context and produce the Node.js result for that same context.
- **FR-006**: The project MUST port the complete public-path-behavior corpus from all 17 files
  matching `test/parallel/test-path*.js` at the pinned Node.js commit into Rust-side automated
  conformance tests.
- **FR-007**: A ported semantic test MUST preserve the upstream input vectors and expected results;
  adaptations MAY change test syntax, iteration structure, fixtures, or environment setup only when
  required by the destination test environment.
- **FR-008**: Every upstream file and assertion MUST appear in a parity ledger with exactly one
  status: ported and passing, harness-adapted and passing, or non-representable. Non-representable
  status MUST be limited to JavaScript runtime or value semantics outside the declared Rust API and
  MUST include a specific rationale; it MUST NOT be used for unimplemented path behavior.
- **FR-009**: Each ported or adapted case MUST retain traceability to its upstream source file and
  pinned Node.js revision, and copied material MUST retain applicable upstream copyright and license
  notices.
- **FR-010**: Representable invalid-input cases MUST produce an equivalent failure condition.
  JavaScript dynamic-type assertions that are prevented by the declared Rust type boundary MUST be
  recorded in the parity ledger rather than silently dropped.
- **FR-011**: Explicit POSIX and Windows behavior MUST be testable on every supported host without
  relying on that host's native path interpretation.
- **FR-012**: The full ported conformance corpus MUST pass before any release or accepted performance
  baseline is created.
- **FR-013**: CodSpeed comparisons MUST cover representative inputs for every path-processing
  operation, both explicit namespaces, and the short, long, already-normalized, and
  normalization-required input categories where applicable.
- **FR-014**: A change flagged as a performance regression MUST be blocked unless it is required for
  Node.js compatibility and has the documented exception evidence required by the constitution.
- **FR-015**: The project MUST produce a machine-checkable parity summary containing the pinned
  baseline, upstream case counts, status counts, failing cases, and exclusions.
- **FR-016**: Changing the pinned Node.js revision MUST trigger a fresh upstream corpus inventory,
  test-ledger update, complete conformance run, and performance-baseline comparison.

### Scope Boundaries

**In scope**:

- Public behavior from Node.js `lib/path.js` at commit
  `3f42cfacf27e348297a52d89b4cdc48b35cb7559`.
- Explicit POSIX, explicit Windows, and host-default variants; constants; the deprecated `_makeLong`
  alias; and `matchesGlob`.
- All public path semantics exercised by the 17 pinned `test-path*.js` files, including cases that
  require deterministic current-directory or platform context.

**Out of scope**:

- File-system access, path existence, permissions, file contents, URL conversion, and module
  loading behavior not exposed by Node.js `path`.
- Literal emulation of JavaScript object identity, coercion, `undefined`, or dynamic-type behavior
  that the declared Rust API cannot represent. These cases remain subject to FR-008 and FR-010.
- Optimizations that require a behavioral deviation from the pinned Node.js baseline.

### Key Entities

- **Compatibility Baseline**: The immutable Node.js revision, source implementation, and upstream
  test corpus used as the behavioral oracle.
- **API Parity Entry**: A public operation or constant, its POSIX/Windows/default variants, supported
  input domain, observable result, and any declared representation boundary.
- **Upstream Test Case**: A source file and assertion with its input vector, context, expected
  result, copyright/license provenance, and stable reference within the pinned revision.
- **Ported Conformance Case**: The project-side counterpart to an upstream test case, including its
  ledger status and any harness-only adaptation rationale.
- **Performance Scenario**: A representative operation, namespace, input category, accepted
  baseline, latest comparison result, and compatibility status.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of advertised operations and variants produce the pinned Node.js observable
  result across the complete accepted conformance corpus, with zero known compatibility failures.
- **SC-002**: All 17 pinned upstream `test-path*.js` files and 100% of their assertions are present
  in the parity ledger; every path-semantic assertion is ported or harness-adapted and passes.
- **SC-003**: The parity summary reports zero unclassified assertions and zero exclusions caused by
  missing implementation; all non-representable cases include an approved representation rationale.
- **SC-004**: Explicit POSIX and Windows conformance cases pass on 100% of supported host platforms
  with identical results for identical inputs and supplied context.
- **SC-005**: The accepted performance suite covers 100% of path-processing operations with all
  applicable representative input categories and reports zero unexplained regressions.
- **SC-006**: For every conformance failure, a maintainer can identify the originating upstream file,
  assertion, baseline revision, and local counterpart directly from the parity report.
- **SC-007**: A library adopter can replace path transformations within the declared input domain
  without observing a behavioral difference in any acceptance scenario.

## Assumptions

- The initial compatibility baseline is Node.js commit
  `3f42cfacf27e348297a52d89b4cdc48b35cb7559`, with
  [`lib/path.js`](https://github.com/nodejs/node/blob/3f42cfacf27e348297a52d89b4cdc48b35cb7559/lib/path.js)
  as the implementation oracle.
- The initial upstream corpus consists of the 17 files matching `test/parallel/test-path*.js` at the
  pinned commit. New matching files are not silently added until the compatibility baseline changes.
- "Directly port" means preserving path inputs, context, and expected observable results while
  translating test syntax and replacing Node.js-specific harness setup where necessary.
- The default Rust string domain contains valid Unicode scalar values. Non-string JavaScript values
  and unpaired UTF-16 surrogate code units are documented representation boundaries unless a later
  specification explicitly expands the public input domain.
- The project supports deterministic injection or control of current-directory and platform context
  wherever the Node.js result depends on them.
- Compatibility is accepted before performance; a faster result that differs from Node.js is a
  failed result.

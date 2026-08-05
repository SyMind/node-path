<!--
Sync Impact Report
- Version change: 1.0.0 -> 1.0.1
- Modified principles: None
- Modified sections:
  - Compatibility and Performance Standards (local path -> GitHub commit permalink)
- Added sections: None
- Removed sections: None
- Follow-up TODOs: None
-->
# node-path Constitution

## Core Principles

### I. Node.js Behavioral Parity (NON-NEGOTIABLE)
The Rust library MUST match the observable behavior of Node.js `path` for every API and input
domain that the library advertises. Observable behavior includes returned values, normalization,
trailing separators, dot segments, roots, drive letters, UNC and namespaced paths, dependence on
the current working directory, platform selection, and exposed error conditions. A known parity
failure blocks release. When Rust cannot represent a JavaScript input or result literally, the
feature specification MUST define the supported boundary and its mapping explicitly; the library
MUST NOT silently substitute different semantics. Behavioral compatibility takes precedence over
performance, convenience, and internal design preferences because compatibility is the library's
primary purpose.

### II. Explicit POSIX and Windows Semantics
POSIX and Windows path behavior MUST be implemented as distinct, explicit contracts equivalent to
Node.js `path.posix` and `path.win32`. Their results MUST NOT vary with the host operating system.
Any host-default API MUST select semantics in the same circumstances as Node.js. Tests for both
contracts MUST run on every supported development host, including cases for separators, roots,
drive-relative paths, UNC paths, namespaced paths, reserved names, and case sensitivity. This
prevents accidental reliance on the machine running the Rust code.

### III. Oracle-Driven Conformance Testing
Every supported method and every compatibility bug fix MUST have automated tests whose expected
behavior is validated against the pinned Node.js reference. The suite MUST cover canonical cases,
boundary cases, invalid inputs within the Rust API's domain, and interactions among path
components. High-risk parsing and normalization code MUST also use differential, table-driven, or
property-based cases broad enough to expose edge conditions. Checked-in fixtures MUST identify the
Node.js revision that produced them. Rust-only expected values are insufficient evidence of parity
when the Node.js implementation can act as an oracle. A feature is complete only when its parity
matrix and regression tests pass.

### IV. Measured Performance Without Semantic Trade-offs
Performance claims and optimizations MUST be supported by reproducible CodSpeed benchmarks.
Benchmarks MUST exercise representative POSIX and Windows inputs, including short and long paths,
already-normalized paths, and paths requiring normalization. An optimization MUST include a
before-and-after comparison for the affected hot path. A regression reported by the configured
CodSpeed comparison policy blocks merge unless the change is required to correct Node.js behavior;
such an exception MUST document the compatibility defect, measured cost, and a follow-up mitigation
plan. No optimization may alter observable behavior. Measurement is required because performance
is the second project priority, subordinate only to compatibility.

### V. Safe, Allocation-Conscious Rust
Implementations MUST use safe Rust by default and MUST avoid unnecessary allocation, copying, and
intermediate string construction on path-processing hot paths. `unsafe` code is permitted only
when a CodSpeed result demonstrates a material benefit, the safety invariants are documented next
to the code, and focused tests exercise those invariants. Public APIs MUST make ownership,
encoding, platform selection, and failure behavior explicit. Internal complexity MUST be justified
by either Node.js parity evidence or benchmark evidence so that speed does not make the library
unverifiable or fragile.

## Compatibility and Performance Standards

The authoritative behavior source and initial oracle is Node.js
[`lib/path.js`](https://github.com/nodejs/node/blob/3f42cfacf27e348297a52d89b4cdc48b35cb7559/lib/path.js)
at commit `3f42cfacf27e348297a52d89b4cdc48b35cb7559`. Specifications, generated fixtures, and benchmark
reports MUST record the Node.js commit or released version they target. Updating that baseline MUST
include a review of upstream behavioral changes and corresponding conformance updates; it does not
by itself require a constitution amendment.

Each feature specification MUST enumerate the Node.js methods and variants in scope, the supported
Rust input domain, deliberate representation mappings, and any excluded behavior. Exclusions MUST
be explicit and MUST NOT be described as parity. Each method MUST have traceable conformance tests.
Release evidence MUST include the full test result and the relevant CodSpeed comparison against the
established baseline. Benchmark inputs, toolchain details, and comparison settings MUST be kept
stable enough for results to be reproduced.

## Development Workflow

1. Record the targeted Node.js revision and create an API/parity matrix before implementing or
   changing behavior.
2. Add or update oracle-backed conformance cases that demonstrate the required behavior and fail
   for an unimplemented feature or confirmed defect.
3. Implement the smallest safe Rust change that satisfies those cases across both POSIX and
   Windows semantics.
4. Run formatting, linting, unit tests, differential tests, and supported cross-platform checks.
5. For hot-path changes or performance claims, run the relevant CodSpeed benchmarks and attach the
   comparison result.
6. Reviewers MUST verify traceability to Node.js behavior, adequate edge-case coverage, and the
   absence of an unexplained performance regression before approval.

Compatibility defects take priority over optimization work. Performance work MUST begin from a
passing conformance baseline, and refactoring MUST retain both conformance coverage and benchmark
coverage for affected paths.

## Governance

This constitution supersedes conflicting project practices and feature-level decisions.
Amendments MUST be proposed in writing with a rationale, the affected principles, compatibility
and performance consequences, and any migration plan. Approval requires explicit maintainer
review and an updated Sync Impact Report in this file.

Constitution versions follow semantic versioning: MAJOR for removal or incompatible redefinition
of a principle or governance rule, MINOR for a new principle or materially expanded obligation,
and PATCH for clarifications that do not change obligations. Every pull request MUST be checked
for compliance with the applicable parity, testing, safety, and performance gates. Every release
MUST have a pinned Node.js baseline, a passing conformance suite, and CodSpeed evidence for covered
hot paths. Any exception MUST be documented and approved before merge; exceptions may not waive
Principle I.

**Version**: 1.0.1 | **Ratified**: 2026-08-04 | **Last Amended**: 2026-08-04

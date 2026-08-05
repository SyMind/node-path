# Data Model: Node.js Path Behavioral Parity

This library has no runtime persistence. The model describes public value types, checked-in
compatibility evidence, generated validation evidence, and benchmark metadata.

## 1. Compatibility Baseline

Represents one immutable Node.js oracle revision.

| Field | Type | Rules |
|-------|------|-------|
| `repository` | URL | MUST be `https://github.com/nodejs/node` for the initial baseline |
| `commit` | 40-character SHA-1 | MUST equal the Git revision used for every recorded blob |
| `node_version` | String | Informational release/development version; initial value `27.0.0-dev` |
| `implementation_path` | Repository path | Initial value `lib/path.js` |
| `implementation_blob` | Git object ID | MUST resolve from `commit` and `implementation_path` |
| `test_glob` | String | Initial value `test/parallel/test-path*.js` |
| `minimatch_version` | String | Initial value `10.2.5` |
| `source_files` | List of Source File | Exactly 17 for the initial baseline |
| `inventoried_at` | ISO 8601 timestamp | Records when counts and hashes were generated |

Validation rules:

- The commit, implementation blob, and every source-file blob MUST belong to the same Git tree.
- The generated source-file, assertion-site, and expanded-case totals MUST match the ledger.
- A baseline is immutable after it becomes verified. Updating Node creates a new baseline.

State transitions:

```text
draft -> inventoried -> verified -> superseded
                    \-> rejected
```

Only `verified` may back a release or performance baseline.

## 2. Source File

Represents one upstream `test-path*.js` file.

| Field | Type | Rules |
|-------|------|-------|
| `path` | Repository path | Unique within the baseline |
| `blob_oid` | Git object ID | MUST match the pinned commit |
| `assertion_sites` | Non-negative integer | Generated count of syntactic assertion call sites |
| `expanded_cases` | Non-negative integer | Generated count after tables/loops/branches expand |
| `counterpart` | Project-relative path | Rust module derived from this file |
| `license_notice` | Project-relative path | MUST resolve to retained upstream notice text |

The initial aggregate is 17 files, 301 assertion sites, and 1,406 expanded cases.

## 3. Node Execution Context

Represents all environment inputs that may affect a path result.

### Node Host

Enum values:

- `win32`
- `darwin`
- `other-posix`

The host is distinct from the path namespace. It controls host-default namespace selection, POSIX
cwd conversion on Windows, and pinned `matchesGlob` case behavior.

### Drive Cwd

| Field | Type | Rules |
|-------|------|-------|
| `device` | String | Exactly one ASCII letter followed by `:` |
| `cwd` | Unicode string | May be empty or mismatched to exercise Node fallback behavior |

Device lookup is ASCII-case-insensitive. Duplicate folded devices are invalid.

### Path Context

| Field | Type | Rules |
|-------|------|-------|
| `host` | Node Host | Required |
| `cwd` | Unicode string | Required; empty is permitted for the pinned failure-safety case |
| `drive_cwds` | List of Drive Cwd | Unique by folded device |

An environment-backed context snapshot may fail when cwd access fails or any required OS value is
not representable in the declared Unicode string domain. Pure algorithms receive an already-created
context and never read or mutate global process state.

## 4. Path Object

Represents Node `parse` output and `format` input.

| Field | Type | Rules |
|-------|------|-------|
| `root` | String-like | Empty when absent |
| `dir` | String-like | Takes precedence over `root` during format when non-empty |
| `base` | String-like | Takes precedence over `name` + `ext` when non-empty |
| `ext` | String-like | A missing leading dot is added by pinned format behavior |
| `name` | String-like | Combined with `ext` only when `base` is empty |

`parse` returns fields that borrow slices of the input. Owned and borrowed `PathObject` values share
the same public field contract.

## 5. Upstream Assertion Case

Represents one expanded upstream test outcome, not merely one source line.

| Field | Type | Rules |
|-------|------|-------|
| `id` | Stable string | Unique; includes baseline, file, vector/helper, index, and dimension |
| `source_file` | Relation to Source File | Required |
| `assertion_line` | Positive integer | Assertion/helper call site at pinned commit |
| `vector_line` | Positive integer or null | Input/invocation site for expanded cases |
| `vector_index` | Non-negative integer or null | Stable table/helper index |
| `namespace` | Enum | `posix`, `win32`, or `host-default` |
| `operation` | Enum | Public operation, constant, type boundary, or module identity |
| `host_branch` | Node Host or null | Branch context when upstream is conditional |
| `arguments` | JSON value | Canonical input representation |
| `context` | Path Context or null | Required for context-dependent cases |
| `expected` | JSON value | Upstream observable result |
| `comparator` | String | Exact, case-folded, regexp, throws, identity, or field-type |
| `content_hash` | SHA-256 | Hash of canonical semantic case content |

Lines provide evidence but are never the sole identity. Table-driven helpers carry both assertion
and vector anchors.

## 6. Ported Conformance Case

Represents the Rust-side disposition and verification of an Upstream Assertion Case.

| Field | Type | Rules |
|-------|------|-------|
| `upstream_case_id` | Relation | Exactly one per expanded case |
| `local_test_id` | String or null | Required for ported/adapted cases |
| `disposition` | Enum | `ported`, `harness-adapted`, `non-representable` |
| `verification` | Enum | See lifecycle below |
| `adaptation` | String or null | Required only for harness-adapted cases |
| `boundary_reason` | String or null | Required only for non-representable cases |
| `replacement_check` | String or null | Equivalent typed/export check when available |
| `approval` | Approval or null | Required for a non-representable accepted case |
| `license_notice` | Project-relative path | Required for all derived cases |

Lifecycle:

```text
discovered
  -> ported/pending -> ported/passing | ported/failing
  -> harness-adapted/pending -> harness-adapted/passing | harness-adapted/failing
  -> non-representable/proposed -> non-representable/approved | rejected

baseline update -> stale -> re-inventoried
```

Accepted final combinations are only:

- `ported` + `passing`
- `harness-adapted` + `passing`
- `non-representable` + `approved`

Every other combination blocks release.

## 7. Approval

Documents a deliberate representation boundary.

| Field | Type | Rules |
|-------|------|-------|
| `approved_by` | String | Maintainer identity; non-empty |
| `approved_at` | ISO 8601 timestamp | Required |
| `rationale` | String | MUST name the unavailable JavaScript value/runtime property |
| `replacement_evidence` | String or null | Local typed or namespace check where available |

An approval cannot describe missing implementation as a representation boundary.

## 8. Parity Ledger

Checked-in document that joins Compatibility Baseline, Source Files, Upstream Assertion Cases, and
Ported Conformance Cases. Its normative shape is
[contracts/parity-ledger.schema.json](contracts/parity-ledger.schema.json).

Validation invariants:

- Initial coverage is exactly 17 files, 301 sites, and 1,406 expanded cases.
- Case IDs and content hashes are unique.
- There are no missing or orphan local test IDs.
- Counts are derived from entries and MUST match declared totals.
- Every source blob and case content hash matches the pinned baseline.
- Release-ready ledgers contain only accepted final disposition/verification combinations.

## 9. Parity Summary

Generated run evidence written to `target/parity-summary.json`.

| Field | Type | Rules |
|-------|------|-------|
| `schema_version` | Integer | Initial value 1 |
| `baseline_commit` | SHA-1 | MUST match the checked-in ledger |
| `run` | Run metadata | Host, target, toolchain, and timestamp |
| `inventory` | Count object | Files, assertion sites, expanded cases |
| `results` | Count object | Passing, failing, pending, stale, exclusions |
| `failures` | List of case IDs/messages | Empty for release |
| `missing_case_ids` | List | Empty for release |
| `orphan_local_test_ids` | List | Empty for release |
| `unapproved_boundaries` | List | Empty for release |
| `releasable` | Boolean | True only when all gates pass |

The normative shape is [contracts/parity-summary.schema.json](contracts/parity-summary.schema.json).

## 10. Glob Program

Internal compiled representation for one pinned minimatch pattern.

| Field | Type | Rules |
|-------|------|-------|
| `pattern` | Unicode string | Maximum 65,536 UTF-16 code units |
| `namespace` | Enum | POSIX or Windows platform option |
| `node_host` | Node Host | Controls `nocase` exactly as pinned Node |
| `regex_source` | UTF-16 sequence | Generated from pinned minimatch rules |
| `matching_mode` | Enum | UCS-2 or UTF-16 according to regex flags |
| `options_fingerprint` | Stable string | Identifies the pinned Node option set |

Compiled programs are not persisted or globally cached in v1. A cache would change memory and
concurrency behavior and requires separate evidence.

## 11. Performance Scenario

Represents one stable CodSpeed benchmark identity.

| Field | Type | Rules |
|-------|------|-------|
| `id` | String | `path_ops/{namespace}/{operation}/{semantic_case}_vN` |
| `namespace` | Enum | POSIX or Windows |
| `operation` | Enum | One path-processing operation |
| `semantic_case` | String | Stable descriptive category |
| `fixture_revision` | Positive integer | Increment when fixture meaning changes |
| `input_size` | Integer | UTF-8 byte count used by benchmark fixture |
| `context` | Path Context or null | Fixed for contextual operations |
| `parity_case_ids` | List | At least one related conformance case where applicable |
| `baseline_environment` | String | Pinned runner/compiler identity |
| `threshold_percent` | Number | Initial global default 10 unless overridden |

Lifecycle:

```text
draft -> conformance-verified -> baselined -> accepted
                                      |-> regressed -> rejected | approved-exception
                                      \-> retired
```

No scenario becomes baselined until all related conformance cases pass. Renaming or semantically
changing a fixture creates a versioned benchmark ID rather than overwriting comparison history.

## Relationships

```text
Compatibility Baseline
  ├── 17 Source Files
  │     └── 301 Assertion Sites
  │            └── 1,406 Upstream Assertion Cases
  │                    └── 1 Ported Conformance Case each
  ├── 1 Parity Ledger
  │     └── many generated Parity Summaries
  └── many Performance Scenarios
          └── one or more Ported Conformance Cases

Path Context ──> contextual API calls, assertion cases, and performance scenarios
Path Object  ──> parse outputs and format inputs
Glob Program ──> matches_glob calls and glob performance scenarios
```

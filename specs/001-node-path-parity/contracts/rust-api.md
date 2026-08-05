# Public Rust API Contract

This contract defines the observable v1 surface. Exact trait derives and documentation layout may
change during implementation, but names, input-domain mappings, return ownership, context behavior,
and Node semantics are normative.

## Crate and Namespace Mapping

Package name: `node-path`

Rust import name: `node_path`

| Node.js surface | Rust surface |
|-----------------|--------------|
| `path` | crate root `node_path` |
| `path.posix` | `node_path::posix` |
| `path.win32` | `node_path::win32` |
| camel-case method | equivalent snake-case function |
| `path._makeLong` | deprecated `_make_long` alias |
| JavaScript variadic strings | `&[&str]` |
| JavaScript path object | `PathObject<S>` |
| process cwd / `=C:` environment | `PathContext` or environment-backed wrapper |

Crate-root functions select Windows semantics on Windows and POSIX semantics on every other target,
matching pinned Node's default export selection.

## Public Constants

Each explicit module and the crate-root selected namespace expose:

```rust
pub const SEP: &str;
pub const DELIMITER: &str;
```

| Namespace | `SEP` | `DELIMITER` |
|-----------|-------|-------------|
| POSIX | `/` | `:` |
| Windows | `\` | `;` |

Constants are conformance-tested but not benchmarked.

## Public Value Types

### `NodeHost`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeHost {
    Win32,
    Darwin,
    OtherPosix,
}
```

`NodeHost` represents the Node execution host, not the path namespace. It controls host-default
selection, Windows-to-POSIX cwd conversion, and the pinned `matchesGlob` `nocase` option.

### `DriveCwd`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveCwd {
    pub device: String,
    pub cwd: String,
}
```

`device` is exactly one ASCII letter plus `:`. Lookup is ASCII-case-insensitive. A cwd whose root
does not match `device` is retained because pinned Node detects the mismatch and falls back to the
drive root.

### `PathContext`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathContext { /* private fields */ }

impl PathContext {
    pub fn new(
        host: NodeHost,
        cwd: impl Into<String>,
        drive_cwds: Vec<DriveCwd>,
    ) -> Result<Self, ContextError>;

    pub fn from_env() -> Result<Self, ContextError>;
    pub fn host(&self) -> NodeHost;
    pub fn cwd(&self) -> &str;
    pub fn drive_cwds(&self) -> &[DriveCwd];
}
```

Construction rejects malformed or duplicate folded drive devices. An empty cwd is accepted because
the pinned test corpus verifies safe behavior when Node's cwd provider returns an empty string.
`from_env` snapshots state once; core algorithms never read or mutate global environment state.

### `PathObject`

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PathObject<S = String> {
    pub root: S,
    pub dir: S,
    pub base: S,
    pub ext: S,
    pub name: S,
}

pub type ParsedPath<'a> = PathObject<&'a str>;
```

For `format`, empty and absent string fields are equivalent within the declared typed input domain.
Precedence matches Node: non-empty `dir` over `root`, then non-empty `base` over `name` + `ext`.

### Errors

```rust
#[derive(Debug)]
pub enum ContextError {
    CurrentDirectory(std::io::Error),
    NonUnicodeCurrentDirectory(std::path::PathBuf),
    InvalidDriveDevice(String),
    DuplicateDriveDevice(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobError {
    PatternTooLong { utf16_units: usize, maximum: usize },
    MatcherInvariant,
}
```

Errors implement `Display` and `std::error::Error`. Dependency-owned error types are not exposed.
`MatcherInvariant` indicates a library defect, not a valid incompatibility or a reason to exclude a
test. Dynamic JavaScript invalid-type errors are prevented by Rust types and recorded in the parity
ledger rather than recreated at runtime.

## Context-Free Operations

The crate root, `posix`, and `win32` expose the same signatures:

```rust
pub fn normalize(path: &str) -> std::borrow::Cow<'_, str>;
pub fn is_absolute(path: &str) -> bool;
pub fn join(paths: &[&str]) -> String;
pub fn dirname(path: &str) -> &str;
pub fn basename<'a>(path: &'a str, suffix: Option<&str>) -> &'a str;
pub fn extname(path: &str) -> &str;
pub fn parse(path: &str) -> ParsedPath<'_>;
pub fn format<S: AsRef<str>>(path_object: &PathObject<S>) -> String;
```

Returned contents MUST exactly match the selected pinned Node namespace. Borrowed returns may point
into the input or use a static empty/`.` value; callers must not infer Node object identity from Rust
borrowing.

## Context-Dependent Operations

Every namespace exposes environment-backed convenience functions:

```rust
pub fn resolve(paths: &[&str]) -> Result<String, ContextError>;
pub fn relative(from: &str, to: &str) -> Result<String, ContextError>;
pub fn to_namespaced_path(path: &str)
    -> Result<std::borrow::Cow<'_, str>, ContextError>;
pub fn matches_glob(path: &str, pattern: &str) -> Result<bool, GlobError>;
```

The explicit modules additionally expose deterministic forms:

```rust
pub fn resolve_with_context(context: &PathContext, paths: &[&str]) -> String;
pub fn relative_with_context(context: &PathContext, from: &str, to: &str) -> String;
pub fn to_namespaced_path_with_context<'a>(
    context: &PathContext,
    path: &'a str,
) -> std::borrow::Cow<'a, str>;
pub fn matches_glob_with_context(
    context: &PathContext,
    path: &str,
    pattern: &str,
) -> Result<bool, GlobError>;
```

The `_with_context` functions are the normative parity and benchmark surface. The convenience forms
MUST be thin adapters that snapshot equivalent process context and delegate. `matches_glob` derives
`NodeHost` from the current target but does not read cwd.

For POSIX `to_namespaced_path`, the result is always the original string content. The uniform
context-aware contract remains so callers and conformance tables can use the same operation model.

## Deprecated Alias

```rust
#[deprecated(note = "Node compatibility alias; use to_namespaced_path")]
pub fn _make_long(path: &str)
    -> Result<std::borrow::Cow<'_, str>, ContextError>;
```

Each explicit namespace also exposes the equivalent context-aware alias. Alias behavior MUST be
identical to `to_namespaced_path`; it has no independent implementation.

## `matches_glob` Behavioral Contract

The matcher reproduces pinned minimatch 10.2.5 using Node's exact option set:

- namespace chooses `platform = posix | win32`;
- `context.host` chooses `nocase` for Darwin and Win32;
- `windowsPathsNoEscape`, `nonegate`, and `nocomment` are enabled;
- `nocaseMagicOnly` is enabled;
- optimization level is 2;
- pattern length is limited to 65,536 JavaScript UTF-16 code units;
- pinned brace-expansion and recursion limits are retained;
- generated non-`u` regular expressions use UCS-2 matching; `u` expressions use UTF-16 matching.

A third-party glob dialect is not a compatible substitute. Actual Rust host state MUST NOT affect a
context-taking call.

## Threading and Side Effects

- Context-taking operations and every context-free operation are deterministic and side-effect-free.
- No v1 operation mutates cwd, environment variables, global caches, or filesystem state.
- No v1 operation performs filesystem I/O.
- Environment-backed wrappers read process state only while creating their local context snapshot.
- No runtime cache is part of v1; adding one requires parity, concurrency, memory, and benchmark
  evidence.

## Compatibility Matrix

| Operation | POSIX | Windows | Host-default | Uses context | Benchmarked |
|-----------|:-----:|:-------:|:------------:|:------------:|:-----------:|
| `resolve` | Yes | Yes | Yes | Yes | Yes |
| `normalize` | Yes | Yes | Yes | No | Yes |
| `is_absolute` | Yes | Yes | Yes | No | Yes |
| `join` | Yes | Yes | Yes | No | Yes |
| `relative` | Yes | Yes | Yes | Yes | Yes |
| `to_namespaced_path` | Yes | Yes | Yes | Uniform contract | Yes |
| `dirname` | Yes | Yes | Yes | No | Yes |
| `basename` | Yes | Yes | Yes | No | Yes |
| `extname` | Yes | Yes | Yes | No | Yes |
| `format` | Yes | Yes | Yes | No | Yes |
| `parse` | Yes | Yes | Yes | No | Yes |
| `matches_glob` | Yes | Yes | Yes | Node host | Yes |
| `_make_long` | Yes | Yes | Yes | Uniform contract | Alias case |
| `SEP` / `DELIMITER` | Yes | Yes | Yes | No | No |

## Packaging Contract

The published package MUST include:

- project `LICENSE`;
- pinned Node license (`UPSTREAM_LICENSE_NODE.txt`);
- third-party/provenance notices for derived Node tests and matcher material;
- public API documentation naming the Node commit used as the semantic baseline.

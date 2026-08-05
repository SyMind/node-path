# node-path

`node-path` is a safe, allocation-conscious Rust implementation of Node.js string path semantics.
Its compatibility baseline is Node.js commit
[`3f42cfac`](https://github.com/nodejs/node/blob/3f42cfacf27e348297a52d89b4cdc48b35cb7559/lib/path.js).
Compatibility is the release gate; performance work is accepted only after parity passes.

```toml
[dependencies]
node-path = "0.1"
```

## API mapping

The crate exposes host-independent `node_path::posix` and `node_path::win32` namespaces. The crate
root selects POSIX on non-Windows targets and Win32 on Windows, matching Node's default namespace.

| Node.js | Rust |
|---|---|
| `path.normalize(value)` | `normalize(value)` |
| `path.isAbsolute(value)` | `is_absolute(value)` |
| `path.join(...values)` | `join(&values)` |
| `path.resolve(...values)` | `resolve(&values)` |
| `path.relative(from, to)` | `relative(from, to)` |
| `path.toNamespacedPath(value)` | `to_namespaced_path(value)` |
| `path.dirname(value)` | `dirname(value)` |
| `path.basename(value, suffix)` | `basename(value, suffix)` |
| `path.extname(value)` | `extname(value)` |
| `path.parse(value)` / `path.format(object)` | `parse(value)` / `format(&object)` |
| `path.matchesGlob(value, pattern)` | `matches_glob(value, pattern)` |
| `path._makeLong(value)` | `_make_long(value)` (deprecated alias) |

```rust
use node_path::{PathContext, NodeHost, posix, win32};

assert_eq!(posix::normalize("/srv//app/../lib"), "/srv/lib");
assert_eq!(win32::normalize("C:/srv\\app\\..\\lib"), "C:\\srv\\lib");

let context = PathContext::new(NodeHost::OtherPosix, "/workspace/project", vec![])?;
assert_eq!(
    posix::resolve_with_context(&context, &["src", "../tests"]),
    "/workspace/project/tests",
);
# Ok::<(), node_path::ContextError>(())
```

Context-free functions accept valid Rust UTF-8 strings and do not touch the filesystem. JavaScript
non-string values, `undefined`, object identity, and unpaired UTF-16 surrogate values are outside
the public Rust string domain and are explicitly classified in the parity ledger. `parse` borrows
slices from its input; transforming functions allocate only when their return contract requires an
owned string. Environment-backed `resolve`, `relative`, and namespaced-path functions may return a
`ContextError`; their `_with_context` variants are deterministic and never mutate process state.

`matches_glob` follows the pinned Node/minimatch option set. Pattern size is limited to 65,536
UTF-16 code units and errors use the dependency-independent `GlobError` type.

## Conformance

The 17 pinned `test/parallel/test-path*.js` sources are mirrored by Rust modules and a checked-in
1,406-case provenance ledger. Run the release gate with:

```console
cargo +1.97.1 test --all-targets --all-features --locked
cargo +1.97.1 test --test node_path -- --nocapture
jq -e '.releasable == true' target/parity-summary.json
```

The optional differential test requires a Node 27 executable built from the pinned commit:

```console
NODE_PATH_ORACLE_BIN=/absolute/path/to/node \
  cargo +1.97.1 test --test differential -- --ignored --nocapture
```

## Performance

The `path_ops` Divan/CodSpeed matrix covers both namespaces and every operation with stable,
versioned IDs. Local discovery is:

```console
cargo +1.97.1 bench --bench path_ops
cargo +1.97.1 codspeed build -m simulation --locked --bench path_ops
cargo +1.97.1 codspeed run -m simulation --bench path_ops
```

See [`benches/README.md`](benches/README.md) for fixture and regression policy. The initial CodSpeed
threshold is 10%; a faster implementation that changes a conformance result is still rejected.

## License and provenance

The crate is MIT licensed. Node-derived behavior/tests and the pinned minimatch/brace-expansion
material are documented in `UPSTREAM_LICENSE_NODE.txt` and `THIRD_PARTY_NOTICES.md`.

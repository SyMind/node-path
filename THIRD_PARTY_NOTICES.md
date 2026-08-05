# Third-Party Notices

This crate reproduces observable behavior and contains test vectors derived from Node.js at commit
`3f42cfacf27e348297a52d89b4cdc48b35cb7559`. See `UPSTREAM_LICENSE_NODE.txt`.

## Node.js path implementation and tests

- Source: <https://github.com/nodejs/node/tree/3f42cfacf27e348297a52d89b4cdc48b35cb7559>
- Derived implementation source: `lib/path.js`
- Derived test corpus: `test/parallel/test-path*.js`
- License: MIT; copyright Node.js contributors and, for historical files, Joyent, Inc. and other
  Node contributors.

Rust counterpart files identify their upstream path and pinned revision. Files corresponding to
upstream sources with a full Joyent/Node header retain that header.

## minimatch 10.2.5

The `matches_glob` implementation is derived from minimatch 10.2.5 as vendored by the pinned Node
revision. minimatch is licensed under the Blue Oak Model License 1.0.0:
<https://blueoakcouncil.org/license/1.0.0>.

Copyright and patent permissions are granted by each contributor under that license. The software
is provided as-is, without warranty or condition, and contributors are not liable for related
damages to the extent allowed by law.

## brace-expansion 5.0.5

The brace expansion behavior used by minimatch is derived from brace-expansion 5.0.5 and is
distributed under the MIT License. Copyright is retained by its upstream contributors. The full MIT
grant and warranty disclaimer are reproduced in the project `LICENSE` file.

## Runtime Rust dependency audit

The locked runtime graph was reviewed on 2026-08-04. It contains `regress 0.11.1`
(`MIT OR Apache-2.0`), `hashbrown 0.16.1` (`MIT OR Apache-2.0`), `allocator-api2 0.2.21`
(`MIT OR Apache-2.0`), `equivalent 1.0.2` (`Apache-2.0 OR MIT`), `foldhash 0.2.0` (`Zlib`), and
`memchr 2.8.3` (`Unlicense OR MIT`). These are compatible permissive licenses. Development-only
benchmark and test dependencies are not linked into the published library. The complete locked
development graph was also checked from Cargo metadata: every package declares a license; the graph
uses permissive alternatives, plus development-only MPL-2.0 (`colored`) and optional platform
licenses with MIT/Apache alternatives. No dependency license text is represented as the project's
own license.

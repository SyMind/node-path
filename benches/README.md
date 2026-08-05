# Path benchmark policy

`path_ops` is the compatibility-gated Divan/CodSpeed suite for the crate's public path-processing
operations. The reference compiler is Rust 1.97.1, the hosted runner is Ubuntu 24.04, and CI installs
exact `cargo-codspeed` 5.0.1. CodSpeed runs CPU Simulation through `CodSpeedHQ/action@v5` with OIDC.

Run the ordinary harness and the local simulation discovery with:

```console
cargo +1.97.1 bench --bench path_ops -- --max-time 0.02
cargo +1.97.1 codspeed build -m simulation --locked --bench path_ops
cargo +1.97.1 codspeed run -m simulation --bench path_ops
```

The suite measures the operation itself, including any returned allocation. Immutable strings,
`PathObject` values, and `PathContext` snapshots are prepared outside the measured closure. It never
benchmarks a Node subprocess or environment capture.

Every identity is `path_ops/{posix|win32}/{operation}/{semantic_case}_vN`. IDs are comparison keys:
do not rename one in place. If input content or semantic meaning changes, add a new `_vN` fixture and
retire the old ID only after the new default-branch baseline exists. Short fixtures remain at most
64 bytes and long fixtures remain approximately 1 KiB.

The initial project policy is a 10% global regression threshold, tuned per benchmark only after
stable default-branch observations. `CodSpeed Performance Analysis` is intended to be a required
branch check. A compatibility-required slowdown must link the affected parity case, before/after
measurements, approval, and mitigation follow-up; performance never overrides a failing parity
report.

Remote project state is established only after `target/parity-summary.json` reports
`"releasable": true`: run the workflow on the default branch to create the first baseline, verify
all IDs in `BENCHMARK_CASES`, configure the 10% threshold in CodSpeed, then require the named check in
GitHub branch protection. Record the resulting baseline/report URL here once that workflow exists.

Current remote evidence: pending the first committed default-branch workflow run.

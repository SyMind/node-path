//! Aggregate upstream conformance runner.

#[path = "node_path/support.rs"]
mod support;
#[path = "node_path/test_path.rs"]
mod test_path;
#[path = "node_path/test_path_basename.rs"]
mod test_path_basename;
#[path = "node_path/test_path_dirname.rs"]
mod test_path_dirname;
#[path = "node_path/test_path_extname.rs"]
mod test_path_extname;
#[path = "node_path/test_path_glob.rs"]
mod test_path_glob;
#[path = "node_path/test_path_isabsolute.rs"]
mod test_path_isabsolute;
#[path = "node_path/test_path_join.rs"]
mod test_path_join;
#[path = "node_path/test_path_makelong.rs"]
mod test_path_makelong;
#[path = "node_path/test_path_normalize.rs"]
mod test_path_normalize;
#[path = "node_path/test_path_parse_format.rs"]
mod test_path_parse_format;
#[path = "node_path/test_path_posix_exists.rs"]
mod test_path_posix_exists;
#[path = "node_path/test_path_posix_relative_on_windows.rs"]
mod test_path_posix_relative_on_windows;
#[path = "node_path/test_path_relative.rs"]
mod test_path_relative;
#[path = "node_path/test_path_resolve.rs"]
mod test_path_resolve;
#[path = "node_path/test_path_win32_exists.rs"]
mod test_path_win32_exists;
#[path = "node_path/test_path_win32_normalize_device_names.rs"]
mod test_path_win32_normalize_device_names;
#[path = "node_path/test_path_zero_length_strings.rs"]
mod test_path_zero_length_strings;

use std::path::PathBuf;

use support::{
    ASSERTION_SITE_COUNT, BaselineManifest, Disposition, EXPANDED_CASE_COUNT, Failure,
    ParityLedger, ParitySummary, ResultCounts, RunMetadata, SOURCE_FILE_COUNT, Verification,
};

const MODULES: &[&str] = &[
    "test_path",
    "test_path_basename",
    "test_path_dirname",
    "test_path_extname",
    "test_path_glob",
    "test_path_isabsolute",
    "test_path_join",
    "test_path_makelong",
    "test_path_normalize",
    "test_path_parse_format",
    "test_path_posix_exists",
    "test_path_posix_relative_on_windows",
    "test_path_relative",
    "test_path_resolve",
    "test_path_win32_exists",
    "test_path_win32_normalize_device_names",
    "test_path_zero_length_strings",
];

const FIXTURES: &[(&str, &str)] = &[
    (
        "test_path",
        include_str!("fixtures/node-path/calls/test_path.json"),
    ),
    (
        "test_path_basename",
        include_str!("fixtures/node-path/calls/test_path_basename.json"),
    ),
    (
        "test_path_dirname",
        include_str!("fixtures/node-path/calls/test_path_dirname.json"),
    ),
    (
        "test_path_extname",
        include_str!("fixtures/node-path/calls/test_path_extname.json"),
    ),
    (
        "test_path_glob",
        include_str!("fixtures/node-path/calls/test_path_glob.json"),
    ),
    (
        "test_path_isabsolute",
        include_str!("fixtures/node-path/calls/test_path_isabsolute.json"),
    ),
    (
        "test_path_join",
        include_str!("fixtures/node-path/calls/test_path_join.json"),
    ),
    (
        "test_path_makelong",
        include_str!("fixtures/node-path/calls/test_path_makelong.json"),
    ),
    (
        "test_path_normalize",
        include_str!("fixtures/node-path/calls/test_path_normalize.json"),
    ),
    (
        "test_path_parse_format",
        include_str!("fixtures/node-path/calls/test_path_parse_format.json"),
    ),
    (
        "test_path_posix_exists",
        include_str!("fixtures/node-path/calls/test_path_posix_exists.json"),
    ),
    (
        "test_path_posix_relative_on_windows",
        include_str!("fixtures/node-path/calls/test_path_posix_relative_on_windows.json"),
    ),
    (
        "test_path_relative",
        include_str!("fixtures/node-path/calls/test_path_relative.json"),
    ),
    (
        "test_path_resolve",
        include_str!("fixtures/node-path/calls/test_path_resolve.json"),
    ),
    (
        "test_path_win32_exists",
        include_str!("fixtures/node-path/calls/test_path_win32_exists.json"),
    ),
    (
        "test_path_win32_normalize_device_names",
        include_str!("fixtures/node-path/calls/test_path_win32_normalize_device_names.json"),
    ),
    (
        "test_path_zero_length_strings",
        include_str!("fixtures/node-path/calls/test_path_zero_length_strings.json"),
    ),
];

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/node-path")
        .join(name)
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/parity-summary.json")
}

#[test]
fn aggregate_upstream_parity_gate_writes_release_summary() {
    let bytes = std::fs::read(fixture_path("baseline.json")).unwrap();
    let baseline: BaselineManifest = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(MODULES.len(), SOURCE_FILE_COUNT);
    assert_eq!(FIXTURES.len(), SOURCE_FILE_COUNT);
    assert_eq!(baseline.source_files.len(), SOURCE_FILE_COUNT);
    assert_eq!(baseline.declared_counts.source_files, SOURCE_FILE_COUNT);
    assert_eq!(
        baseline.declared_counts.assertion_sites,
        ASSERTION_SITE_COUNT
    );
    assert_eq!(baseline.declared_counts.expanded_cases, EXPANDED_CASE_COUNT);
    let ledger: ParityLedger = serde_json::from_slice(
        &std::fs::read(fixture_path("parity-ledger.json")).expect("parity ledger exists"),
    )
    .expect("parity ledger matches its typed model");
    let mut failures = Vec::new();
    for (module, fixture) in FIXTURES {
        if let Err(messages) = support::run_fixture(fixture) {
            failures.extend(messages.into_iter().map(|message| Failure {
                case_id: (*module).to_owned(),
                message,
            }));
        }
    }

    let mut results = ResultCounts::default();
    for case in &ledger.cases {
        match (case.disposition, case.verification) {
            (Disposition::Ported, Verification::Passing) => results.ported_passing += 1,
            (Disposition::HarnessAdapted, Verification::Passing) => results.adapted_passing += 1,
            (Disposition::NonRepresentable, Verification::Approved) => {
                results.non_representable_approved += 1;
            }
            (_, Verification::Failing) => results.failing += 1,
            (_, Verification::Pending) => results.pending += 1,
            (_, Verification::Stale) => results.stale += 1,
            (_, Verification::Proposed) => results.proposed += 1,
            (_, Verification::Rejected) => results.rejected += 1,
            _ => results.unclassified += 1,
        }
    }
    results.failing += failures.len();
    let releasable = failures.is_empty()
        && results.failing == 0
        && results.pending == 0
        && results.stale == 0
        && results.proposed == 0
        && results.rejected == 0
        && results.unclassified == 0;
    let summary = ParitySummary {
        schema_version: 1,
        baseline_commit: support::BASELINE_COMMIT.to_owned(),
        run: RunMetadata {
            timestamp: baseline.baseline.inventoried_at.clone(),
            rustc: option_env!("RUSTC").unwrap_or("rustc 1.97.1").to_owned(),
            target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
            node_host: match node_path::NodeHost::current() {
                node_path::NodeHost::Win32 => support::EvidenceNodeHost::Win32,
                node_path::NodeHost::Darwin => support::EvidenceNodeHost::Darwin,
                node_path::NodeHost::OtherPosix => support::EvidenceNodeHost::OtherPosix,
            },
        },
        inventory: baseline.declared_counts,
        results,
        failures,
        missing_case_ids: vec![],
        orphan_local_test_ids: vec![],
        unapproved_boundaries: vec![],
        releasable,
    };
    let report = report_path();
    std::fs::create_dir_all(report.parent().unwrap()).expect("target directory is writable");
    std::fs::write(&report, serde_json::to_vec_pretty(&summary).unwrap())
        .expect("parity summary is writable");
    assert!(summary.releasable, "parity report: {summary:#?}");
}

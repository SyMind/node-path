#[path = "node_path/support.rs"]
mod support;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use support::{
    ASSERTION_SITE_COUNT, Disposition, EXPANDED_CASE_COUNT, ParityLedger, SOURCE_FILE_COUNT,
    Verification, is_content_hash, is_git_oid,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/node-path")
        .join(name)
}

#[test]
fn checked_in_ledger_is_complete_and_release_ready() {
    let ledger: ParityLedger = serde_json::from_slice(
        &std::fs::read(fixture_path("parity-ledger.json")).expect("ledger fixture exists"),
    )
    .expect("ledger matches the typed contract");

    assert_eq!(ledger.schema_version, 1);
    assert_eq!(ledger.baseline.commit, support::BASELINE_COMMIT);
    assert!(is_git_oid(&ledger.baseline.commit));
    assert!(is_git_oid(&ledger.baseline.implementation_blob));
    assert_eq!(ledger.source_files.len(), SOURCE_FILE_COUNT);
    assert_eq!(ledger.declared_counts.source_files, SOURCE_FILE_COUNT);
    assert_eq!(ledger.declared_counts.assertion_sites, ASSERTION_SITE_COUNT);
    assert_eq!(ledger.declared_counts.expanded_cases, EXPANDED_CASE_COUNT);
    assert_eq!(ledger.cases.len(), EXPANDED_CASE_COUNT);

    let mut source_paths = HashSet::new();
    let mut expected_per_file = HashMap::new();
    for source in &ledger.source_files {
        assert!(
            source_paths.insert(source.path.as_str()),
            "duplicate source file"
        );
        assert!(is_git_oid(&source.blob_oid));
        expected_per_file.insert(source.path.as_str(), source.expanded_cases);
        assert!(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(&source.counterpart)
                .exists(),
            "missing counterpart {}",
            source.counterpart
        );
    }

    let mut ids = HashSet::new();
    let mut hashes = HashSet::new();
    let mut actual_per_file = HashMap::<&str, usize>::new();
    for case in &ledger.cases {
        assert!(
            ids.insert(case.id.as_str()),
            "duplicate case id {}",
            case.id
        );
        assert!(
            is_content_hash(&case.content_hash),
            "invalid content hash for {}",
            case.id
        );
        assert!(
            hashes.insert(case.content_hash.as_str()),
            "duplicate semantic hash for {}",
            case.id
        );
        assert!(source_paths.contains(case.source.file.as_str()));
        *actual_per_file
            .entry(case.source.file.as_str())
            .or_default() += 1;
        match (case.disposition, case.verification) {
            (Disposition::Ported | Disposition::HarnessAdapted, Verification::Passing) => {
                assert!(
                    case.local_test_id
                        .as_deref()
                        .is_some_and(|id| !id.is_empty())
                );
                assert!(case.boundary_reason.is_none());
                assert!(case.approval.is_none());
            }
            (Disposition::NonRepresentable, Verification::Approved) => {
                assert!(
                    case.boundary_reason
                        .as_deref()
                        .is_some_and(|value| !value.is_empty())
                );
                assert!(case.approval.is_some());
            }
            state => panic!("non-release lifecycle state {state:?} for {}", case.id),
        }
    }
    for (file, expected) in expected_per_file {
        assert_eq!(
            actual_per_file.get(file),
            Some(&expected),
            "coverage for {file}"
        );
    }
}

#[path = "../benches/cases.rs"]
mod cases;

use std::collections::HashSet;

#[test]
fn benchmark_matrix_has_stable_complete_ids_and_fixtures() {
    const OPERATIONS: &[&str] = &[
        "resolve",
        "normalize",
        "is_absolute",
        "join",
        "relative",
        "to_namespaced_path",
        "dirname",
        "basename",
        "extname",
        "format",
        "parse",
        "matches_glob",
    ];
    let source = include_str!("../benches/path_ops.rs");
    let mut ids = HashSet::new();
    for case in cases::BENCHMARK_CASES {
        assert!(ids.insert(case.id), "duplicate benchmark id {}", case.id);
        assert!(case.id.starts_with("path_ops/"));
        assert!(case.id.ends_with(&format!("_v{}", case.fixture_version)));
        assert!(source.contains(case.id), "{} is not registered", case.id);
    }
    for namespace in ["posix", "win32"] {
        for operation in OPERATIONS {
            assert!(
                cases::BENCHMARK_CASES
                    .iter()
                    .any(|case| case.namespace == namespace && case.operation == *operation),
                "missing {namespace}/{operation} benchmark"
            );
        }
    }
    for category in [
        "short",
        "long",
        "clean",
        "dirty",
        "structural",
        "unicode",
        "suffix_dot",
    ] {
        assert!(
            cases::BENCHMARK_CASES
                .iter()
                .any(|case| case.category.contains(category)),
            "missing semantic category {category}"
        );
    }
    assert!(cases::SHORT_POSIX_CLEAN.len() <= 64);
    assert!(cases::SHORT_WIN32_CLEAN.len() <= 64);
    assert!((768..=1_280).contains(&cases::LONG_POSIX_DIRTY.len()));
    assert!((768..=1_280).contains(&cases::LONG_WIN32_DIRTY.len()));
    assert_eq!(cases::posix_context().cwd(), "/workspace/project");
    assert_eq!(cases::win32_context().cwd(), "C:\\workspace\\project");
}

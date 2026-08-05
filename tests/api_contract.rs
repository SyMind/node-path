use std::borrow::Cow;

use node_path::{NodeHost, ParsedPath, PathContext, PathObject};

#[test]
fn exposes_node_namespaces_constants_and_snake_case_functions() {
    assert_eq!(node_path::posix::SEP, "/");
    assert_eq!(node_path::posix::DELIMITER, ":");
    assert_eq!(node_path::win32::SEP, "\\");
    assert_eq!(node_path::win32::DELIMITER, ";");

    let parsed: ParsedPath<'_> = node_path::posix::parse("/a/file.txt");
    assert_eq!(parsed.base, "file.txt");
    let normalized: Cow<'_, str> = node_path::posix::normalize("already/clean");
    assert_eq!(normalized, "already/clean");
}

#[test]
fn owned_and_borrowed_path_objects_share_format_semantics() {
    let owned = PathObject {
        root: "/".to_owned(),
        dir: "/tmp".to_owned(),
        base: String::new(),
        ext: "txt".to_owned(),
        name: "report".to_owned(),
    };
    let borrowed = PathObject {
        root: "/",
        dir: "/tmp",
        base: "",
        ext: "txt",
        name: "report",
    };
    assert_eq!(node_path::posix::format(&owned), "/tmp/report.txt");
    assert_eq!(node_path::posix::format(&borrowed), "/tmp/report.txt");
}

#[test]
fn crate_root_selects_the_target_namespace() {
    if cfg!(windows) {
        assert_eq!(node_path::SEP, node_path::win32::SEP);
        assert_eq!(
            node_path::normalize("a/b"),
            node_path::win32::normalize("a/b")
        );
    } else {
        assert_eq!(node_path::SEP, node_path::posix::SEP);
        assert_eq!(
            node_path::normalize("a/b"),
            node_path::posix::normalize("a/b")
        );
    }
}

#[test]
#[allow(deprecated)]
fn deprecated_alias_is_identical_to_namespaced_path() {
    let context = PathContext::new(NodeHost::OtherPosix, "/tmp", vec![]).unwrap();
    assert_eq!(
        node_path::posix::_make_long_with_context(&context, "a"),
        node_path::posix::to_namespaced_path_with_context(&context, "a")
    );
}

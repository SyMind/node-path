use std::borrow::Cow;

use node_path::{DriveCwd, NodeHost, PathContext, posix, win32};

#[test]
fn posix_contextual_operations_are_deterministic() {
    let context = PathContext::new(NodeHost::OtherPosix, "/a/b", vec![]).unwrap();
    assert_eq!(posix::resolve_with_context(&context, &["../c"]), "/a/c");
    assert_eq!(
        posix::relative_with_context(&context, "/a/b", "/a/c/d"),
        "../c/d"
    );
    assert_eq!(
        posix::to_namespaced_path_with_context(&context, "a/b"),
        Cow::Borrowed("a/b")
    );

    let empty = PathContext::new(NodeHost::OtherPosix, "", vec![]).unwrap();
    assert_eq!(posix::resolve_with_context(&empty, &[]), ".");
}

#[test]
fn win32_contextual_operations_honor_drive_cwds_and_unc() {
    let context = PathContext::new(
        NodeHost::Win32,
        "C:\\work\\project",
        vec![DriveCwd {
            device: "C:".into(),
            cwd: "C:\\drive-cwd".into(),
        }],
    )
    .unwrap();
    assert_eq!(
        win32::resolve_with_context(&context, &["C:child"]),
        "C:\\drive-cwd\\child"
    );
    assert_eq!(
        win32::relative_with_context(&context, "C:\\a\\b", "C:\\a\\c"),
        "..\\c"
    );
    assert_eq!(
        win32::to_namespaced_path_with_context(&context, "C:\\a"),
        "\\\\?\\C:\\a"
    );
    assert_eq!(
        win32::to_namespaced_path_with_context(&context, "\\\\server\\share\\a"),
        "\\\\?\\UNC\\server\\share\\a"
    );
}

#[test]
fn environment_facades_delegate_to_equivalent_snapshots() {
    let context = PathContext::from_env().unwrap();
    assert_eq!(
        posix::resolve(&["."]).unwrap(),
        posix::resolve_with_context(&context, &["."])
    );
}

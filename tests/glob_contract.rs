use node_path::{GlobError, NodeHost, PathContext, posix, win32};

fn context(host: NodeHost) -> PathContext {
    PathContext::new(host, "/", vec![]).unwrap()
}

#[test]
fn glob_supports_node_separator_magic_and_case_modes() {
    assert!(
        posix::matches_glob_with_context(&context(NodeHost::OtherPosix), "foo/bar.js", "**/*.js")
            .unwrap()
    );
    assert!(
        !posix::matches_glob_with_context(
            &context(NodeHost::OtherPosix),
            "foo\\bar.js",
            "foo/*.js"
        )
        .unwrap()
    );
    assert!(
        win32::matches_glob_with_context(&context(NodeHost::Win32), "FOO\\bar.JS", "**/*.js")
            .unwrap()
    );
    assert!(
        posix::matches_glob_with_context(&context(NodeHost::Darwin), "FILE.TXT", "*.txt").unwrap()
    );
}

#[test]
fn glob_supports_classes_braces_extglobs_and_malformed_literals() {
    let context = context(NodeHost::OtherPosix);
    assert!(posix::matches_glob_with_context(&context, "a.js", "[ab].js").unwrap());
    assert!(posix::matches_glob_with_context(&context, "a.ts", "*.{js,ts}").unwrap());
    assert!(posix::matches_glob_with_context(&context, "foo.js", "@(foo|bar).js").unwrap());
    assert!(posix::matches_glob_with_context(&context, "[", "[").unwrap());
}

#[test]
fn glob_counts_pattern_limits_in_utf16_units() {
    let pattern = format!("{}a", "😀".repeat(32_768));
    assert_eq!(
        posix::matches_glob_with_context(&context(NodeHost::OtherPosix), "x", &pattern),
        Err(GlobError::PatternTooLong {
            utf16_units: 65_537,
            maximum: 65_536,
        })
    );
}

use node_path::{ParsedPath, PathObject, win32};

#[test]
fn win32_context_free_operations_match_node_examples() {
    assert_eq!(win32::normalize(""), ".");
    assert_eq!(
        win32::normalize("C:/temp//foo\\bar\\..\\"),
        "C:\\temp\\foo\\"
    );
    assert_eq!(win32::normalize("\\\\server\\share"), "\\\\server\\share\\");
    assert!(win32::is_absolute("C:\\root"));
    assert!(win32::is_absolute("\\\\server\\share"));
    assert!(!win32::is_absolute("C:relative"));
    assert_eq!(
        win32::join(&["C:\\foo", "bar", "..", "baz"]),
        "C:\\foo\\baz"
    );
    assert_eq!(win32::dirname("C:\\foo\\bar.txt"), "C:\\foo");
    assert_eq!(win32::basename("C:\\foo\\bar.txt", Some(".txt")), "bar");
    assert_eq!(win32::extname("C:\\foo\\archive.tar.gz"), ".gz");
}

#[test]
fn win32_parse_format_and_reserved_names_match_node() {
    let parsed: ParsedPath<'_> = win32::parse("C:\\用户\\file.txt");
    assert_eq!(parsed.root, "C:\\");
    assert_eq!(parsed.dir, "C:\\用户");
    assert_eq!(parsed.base, "file.txt");
    assert_eq!(parsed.ext, ".txt");
    assert_eq!(parsed.name, "file");

    let object = PathObject {
        root: "C:\\",
        dir: "C:\\chosen",
        base: "",
        name: "report",
        ext: "log",
    };
    assert_eq!(win32::format(&object), "C:\\chosen\\report.log");
    assert_eq!(win32::normalize("CON:"), ".\\CON:.");
}

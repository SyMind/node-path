use node_path::{ParsedPath, PathObject, posix};

#[test]
fn posix_context_free_operations_match_node_examples() {
    assert_eq!(posix::normalize(""), ".");
    assert_eq!(posix::normalize("/foo//bar/../baz/"), "/foo/baz/");
    assert_eq!(posix::normalize("a\\b/./c"), "a\\b/c");
    assert!(posix::is_absolute("/root"));
    assert!(!posix::is_absolute("root"));
    assert_eq!(posix::join(&["/foo", "bar", "..", "baz"]), "/foo/baz");
    assert_eq!(posix::dirname("/foo/bar/"), "/foo");
    assert_eq!(posix::dirname("//foo"), "//");
    assert_eq!(posix::basename("/foo/bar.txt", Some(".txt")), "bar");
    assert_eq!(posix::extname("/foo/.profile"), "");
    assert_eq!(posix::extname("/foo/archive.tar.gz"), ".gz");
}

#[test]
fn posix_parse_and_format_preserve_node_precedence() {
    let parsed: ParsedPath<'_> = posix::parse("/home/用户/file.txt");
    assert_eq!(parsed.root, "/");
    assert_eq!(parsed.dir, "/home/用户");
    assert_eq!(parsed.base, "file.txt");
    assert_eq!(parsed.ext, ".txt");
    assert_eq!(parsed.name, "file");

    let object = PathObject {
        root: "/",
        dir: "/chosen",
        base: "base.bin",
        name: "ignored",
        ext: "txt",
    };
    assert_eq!(posix::format(&object), "/chosen/base.bin");
}

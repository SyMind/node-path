use camino::Utf8Path;

// Test cases from https://github.com/nodejs/node/blob/1b2d2f7e682268228b1352cba7389db01614812a/test/parallel/test-path-normalize.js

#[test]
#[cfg(target_os = "windows")]
fn test_path_normalize_windows() {
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_path_normalize_posix() {
    assert_eq!(Utf8Path::new("./fixtures///b/../b/c.js").normalize().as_str(), "fixtures/b/c.js");
    assert_eq!(Utf8Path::new("/foo/../../../bar").normalize().as_str(), "/bar");
    assert_eq!(Utf8Path::new("a//b//../b").normalize().as_str(), "a/b");
    assert_eq!(Utf8Path::new("a//b//./c").normalize().as_str(), "a/b/c");
    assert_eq!(Utf8Path::new("./fixtures///b/../b/c.js").normalize().as_str(), "fixtures/b/c.js");
    assert_eq!(Utf8Path::new("a//b//.").normalize().as_str(), "a/b");
    assert_eq!(Utf8Path::new("/a/b/c/../../../x/y/z").normalize().as_str(), "/x/y/z");
    assert_eq!(Utf8Path::new("///..//./foo/.//bar").normalize().as_str(), "/foo/bar");
    assert_eq!(Utf8Path::new("bar/foo../../").normalize().as_str(), "bar/");
    assert_eq!(Utf8Path::new("bar/foo../..").normalize().as_str(), "bar");
    assert_eq!(Utf8Path::new("bar/foo../../baz").normalize().as_str(), "bar/baz");
    assert_eq!(Utf8Path::new("bar/foo../").normalize().as_str(), "bar/foo../");
    assert_eq!(Utf8Path::new("bar/foo..").normalize().as_str(), "bar/foo..");
    assert_eq!(Utf8Path::new("../foo../../../bar").normalize().as_str(), "../../bar");
    assert_eq!(Utf8Path::new("../.../.././.../../../bar").normalize().as_str(), "../../bar");
    assert_eq!(Utf8Path::new("../../../foo/../../../bar").normalize().as_str(), "../../../../../bar");
    assert_eq!(Utf8Path::new("../../../foo/../../../bar/../../").normalize().as_str(), "../../../../../../");
    assert_eq!(Utf8Path::new("../foobar/barfoo/foo/../../../bar/../../").normalize().as_str(), "../../");
    assert_eq!(Utf8Path::new("../.../../foobar/../../../bar/../../baz").normalize().as_str(), "../../../../baz");
    assert_eq!(Utf8Path::new("foo/bar\\baz").normalize().as_str(), "foo/bar\\baz");
}

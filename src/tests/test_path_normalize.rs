use super::Utf8Path;

// Test cases from https://github.com/nodejs/node/blob/1b2d2f7e682268228b1352cba7389db01614812a/test/parallel/test-path-normalize.js

#[test]
fn test_path_normalize_windows() {
    assert_eq!(
        Utf8Path::new("./fixtures///b/../b/c.js")
            .normalize_win32()
            .as_str(),
        "fixtures\\b\\c.js"
    );
    assert_eq!(
        Utf8Path::new("/foo/../../../bar")
            .normalize_win32()
            .as_str(),
        "\\bar"
    );
    assert_eq!(
        Utf8Path::new("a//b//../b").normalize_win32().as_str(),
        "a\\b"
    );
    assert_eq!(
        Utf8Path::new("a//b//./c").normalize_win32().as_str(),
        "a\\b\\c"
    );
    assert_eq!(Utf8Path::new("a//b//.").normalize_win32().as_str(), "a\\b");
    assert_eq!(
        Utf8Path::new("//server/share/dir/file.ext")
            .normalize_win32()
            .as_str(),
        "\\\\server\\share\\dir\\file.ext"
    );
    assert_eq!(
        Utf8Path::new("/a/b/c/../../../x/y/z")
            .normalize_win32()
            .as_str(),
        "\\x\\y\\z"
    );
    assert_eq!(Utf8Path::new("C:").normalize_win32().as_str(), "C:.");
    assert_eq!(
        Utf8Path::new("C:..\\abc").normalize_win32().as_str(),
        "C:..\\abc"
    );
    assert_eq!(
        Utf8Path::new("C:..\\..\\abc\\..\\def")
            .normalize_win32()
            .as_str(),
        "C:..\\..\\def"
    );
    assert_eq!(Utf8Path::new("C:\\.").normalize_win32().as_str(), "C:\\");
    assert_eq!(
        Utf8Path::new("file:stream").normalize_win32().as_str(),
        "file:stream"
    );
    assert_eq!(
        Utf8Path::new("bar\\foo..\\..\\").normalize_win32().as_str(),
        "bar\\"
    );
    assert_eq!(
        Utf8Path::new("bar\\foo..\\..").normalize_win32().as_str(),
        "bar"
    );
    assert_eq!(
        Utf8Path::new("bar\\foo..\\..\\baz")
            .normalize_win32()
            .as_str(),
        "bar\\baz"
    );
    assert_eq!(
        Utf8Path::new("bar\\foo..\\").normalize_win32().as_str(),
        "bar\\foo..\\"
    );
    assert_eq!(
        Utf8Path::new("bar\\foo..").normalize_win32().as_str(),
        "bar\\foo.."
    );
    assert_eq!(
        Utf8Path::new("..\\foo..\\..\\..\\bar")
            .normalize_win32()
            .as_str(),
        "..\\..\\bar"
    );
    assert_eq!(
        Utf8Path::new("..\\...\\..\\.\\...\\..\\..\\bar")
            .normalize_win32()
            .as_str(),
        "..\\..\\bar"
    );
    assert_eq!(
        Utf8Path::new("../../../foo/../../../bar")
            .normalize_win32()
            .as_str(),
        "..\\..\\..\\..\\..\\bar"
    );
    assert_eq!(
        Utf8Path::new("../../../foo/../../../bar/../../")
            .normalize_win32()
            .as_str(),
        "..\\..\\..\\..\\..\\..\\"
    );
    assert_eq!(
        Utf8Path::new("../foobar/barfoo/foo/../../../bar/../../")
            .normalize_win32()
            .as_str(),
        "..\\..\\"
    );
    assert_eq!(
        Utf8Path::new("../.../../foobar/../../../bar/../../baz")
            .normalize_win32()
            .as_str(),
        "..\\..\\..\\..\\baz"
    );
    assert_eq!(
        Utf8Path::new("foo/bar\\baz").normalize_win32().as_str(),
        "foo\\bar\\baz"
    );
    assert_eq!(
        Utf8Path::new("\\\\.\\foo").normalize_win32().as_str(),
        "\\\\.\\foo"
    );
    assert_eq!(
        Utf8Path::new("\\\\.\\foo\\").normalize_win32().as_str(),
        "\\\\.\\foo\\"
    );
    assert_eq!(
        Utf8Path::new("test/../C:/Windows")
            .normalize_win32()
            .as_str(),
        ".\\C:\\Windows"
    );
    assert_eq!(
        Utf8Path::new("test/../C:Windows")
            .normalize_win32()
            .as_str(),
        ".\\C:Windows"
    );
    assert_eq!(
        Utf8Path::new("./upload/../C:/Windows")
            .normalize_win32()
            .as_str(),
        ".\\C:\\Windows"
    );
    assert_eq!(
        Utf8Path::new("./upload/../C:x").normalize_win32().as_str(),
        ".\\C:x"
    );
    assert_eq!(
        Utf8Path::new("test/../??/D:/Test")
            .normalize_win32()
            .as_str(),
        ".\\??\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("test/C:/../../F:").normalize_win32().as_str(),
        ".\\F:"
    );
    assert_eq!(
        Utf8Path::new("test/C:foo/../../F:")
            .normalize_win32()
            .as_str(),
        ".\\F:"
    );
    assert_eq!(
        Utf8Path::new("test/C:/../../F:\\")
            .normalize_win32()
            .as_str(),
        ".\\F:\\"
    );
    assert_eq!(
        Utf8Path::new("test/C:foo/../../F:\\")
            .normalize_win32()
            .as_str(),
        ".\\F:\\"
    );
    assert_eq!(
        Utf8Path::new("test/C:/../../F:x")
            .normalize_win32()
            .as_str(),
        ".\\F:x"
    );
    assert_eq!(
        Utf8Path::new("test/C:foo/../../F:x")
            .normalize_win32()
            .as_str(),
        ".\\F:x"
    );
    assert_eq!(
        Utf8Path::new("/test/../??/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\??\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("/test/../?/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\?\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("//test/../??/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\\\test\\..\\??\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("//test/../?/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\\\test\\..\\?\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("\\\\?\\test/../?/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\\\?\\?\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("\\\\?\\test/../../?/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\\\?\\?\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("\\\\.\\test/../?/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\\\.\\?\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("\\\\.\\test/../../?/D:/Test")
            .normalize_win32()
            .as_str(),
        "\\\\.\\?\\D:\\Test"
    );
    assert_eq!(
        Utf8Path::new("//server/share/dir/../../../?/D:/file")
            .normalize_win32()
            .as_str(),
        "\\\\server\\share\\?\\D:\\file"
    );
    assert_eq!(
        Utf8Path::new("//server/goodshare/../badshare/file")
            .normalize_win32()
            .as_str(),
        "\\\\server\\goodshare\\badshare\\file"
    );
}

#[test]
fn test_path_normalize_posix() {
    assert_eq!(
        Utf8Path::new("./fixtures///b/../b/c.js")
            .normalize_posix()
            .as_str(),
        "fixtures/b/c.js"
    );
    assert_eq!(
        Utf8Path::new("/foo/../../../bar")
            .normalize_posix()
            .as_str(),
        "/bar"
    );
    assert_eq!(
        Utf8Path::new("a//b//../b").normalize_posix().as_str(),
        "a/b"
    );
    assert_eq!(
        Utf8Path::new("a//b//./c").normalize_posix().as_str(),
        "a/b/c"
    );
    assert_eq!(
        Utf8Path::new("./fixtures///b/../b/c.js")
            .normalize_posix()
            .as_str(),
        "fixtures/b/c.js"
    );
    assert_eq!(Utf8Path::new("a//b//.").normalize_posix().as_str(), "a/b");
    assert_eq!(
        Utf8Path::new("/a/b/c/../../../x/y/z")
            .normalize_posix()
            .as_str(),
        "/x/y/z"
    );
    assert_eq!(
        Utf8Path::new("///..//./foo/.//bar")
            .normalize_posix()
            .as_str(),
        "/foo/bar"
    );
    assert_eq!(
        Utf8Path::new("bar/foo../../").normalize_posix().as_str(),
        "bar/"
    );
    assert_eq!(
        Utf8Path::new("bar/foo../..").normalize_posix().as_str(),
        "bar"
    );
    assert_eq!(
        Utf8Path::new("bar/foo../../baz").normalize_posix().as_str(),
        "bar/baz"
    );
    assert_eq!(
        Utf8Path::new("bar/foo../").normalize_posix().as_str(),
        "bar/foo../"
    );
    assert_eq!(
        Utf8Path::new("bar/foo..").normalize_posix().as_str(),
        "bar/foo.."
    );
    assert_eq!(
        Utf8Path::new("../foo../../../bar")
            .normalize_posix()
            .as_str(),
        "../../bar"
    );
    assert_eq!(
        Utf8Path::new("../.../.././.../../../bar")
            .normalize_posix()
            .as_str(),
        "../../bar"
    );
    assert_eq!(
        Utf8Path::new("../../../foo/../../../bar")
            .normalize_posix()
            .as_str(),
        "../../../../../bar"
    );
    assert_eq!(
        Utf8Path::new("../../../foo/../../../bar/../../")
            .normalize_posix()
            .as_str(),
        "../../../../../../"
    );
    assert_eq!(
        Utf8Path::new("../foobar/barfoo/foo/../../../bar/../../")
            .normalize_posix()
            .as_str(),
        "../../"
    );
    assert_eq!(
        Utf8Path::new("../.../../foobar/../../../bar/../../baz")
            .normalize_posix()
            .as_str(),
        "../../../../baz"
    );
    assert_eq!(
        Utf8Path::new("foo/bar\\baz").normalize_posix().as_str(),
        "foo/bar\\baz"
    );
}

use super::Utf8PathBuf;

const JOIN_TESTS: &[(&[&str], &str)] = &[
    (&[".", "x/b", "..", "/b/c.js"], "x/b/c.js"),
    (&[], "."),
    (&["/.", "x/b", "..", "/b/c.js"], "/x/b/c.js"),
    (&["/foo", "../../../bar"], "/bar"),
    (&["foo", "../../../bar"], "../../bar"),
    (&["foo/", "../../../bar"], "../../bar"),
    (&["foo/x", "../../../bar"], "../bar"),
    (&["foo/x", "./bar"], "foo/x/bar"),
    (&["foo/x/", "./bar"], "foo/x/bar"),
    (&["foo/x/", ".", "bar"], "foo/x/bar"),
    (&["./"], "./"),
    (&[".", "./"], "./"),
    (&[".", ".", "."], "."),
    (&[".", "./", "."], "."),
    (&[".", "/./", "."], "."),
    (&[".", "/////./", "."], "."),
    (&["."], "."),
    (&["", "."], "."),
    (&["", "foo"], "foo"),
    (&["foo", "/bar"], "foo/bar"),
    (&["", "/foo"], "/foo"),
    (&["", "", "/foo"], "/foo"),
    (&["", "", "foo"], "foo"),
    (&["foo", ""], "foo"),
    (&["foo/", ""], "foo/"),
    (&["foo", "", "/bar"], "foo/bar"),
    (&["./", "..", "/foo"], "../foo"),
    (&["./", "..", "..", "/foo"], "../../foo"),
    (&[".", "..", "..", "/foo"], "../../foo"),
    (&["", "..", "..", "/foo"], "../../foo"),
    (&["/"], "/"),
    (&["/", "."], "/"),
    (&["/", ".."], "/"),
    (&["/", "..", ".."], "/"),
    (&[""], "."),
    (&["", ""], "."),
    (&[" /foo"], " /foo"),
    (&[" ", "foo"], " /foo"),
    (&[" ", "."], " "),
    (&[" ", "/"], " /"),
    (&[" ", ""], " "),
    (&["/", "foo"], "/foo"),
    (&["/", "/foo"], "/foo"),
    (&["/", "//foo"], "/foo"),
    (&["/", "", "/foo"], "/foo"),
    (&["", "/", "foo"], "/foo"),
    (&["", "/", "/foo"], "/foo"),
];

#[test]
fn test_path_join() {
    JOIN_TESTS.iter().for_each(|(paths, expected)| {
        let mut path = Utf8PathBuf::new();
        for p in *paths {
            path = path.join_posix(p);
        }
        assert_eq!(path.normalize_posix().as_str(), *expected);

        let mut path = Utf8PathBuf::new();
        for p in *paths {
            path = path.join_win32(p);
        }
        // For non-Windows specific tests with the Windows join(), we need to try
        // replacing the slashes since the non-Windows specific tests" `expected`
        // use forward slashes
        assert_eq!(
            path.normalize_win32().as_str().replace("\\", "/"),
            *expected
        );
    });
}

const WINDOWS_SPECIFIC_JOIN_TESTS: &[(&[&str], &str)] = &[
    // UNC path expected
    (&["//foo/bar"], "\\\\foo\\bar\\"),
    (&["\\/foo/bar"], "\\\\foo\\bar\\"),
    (&["\\\\foo/bar"], "\\\\foo\\bar\\"),
    // UNC path expected - server and share separate
    (&["//foo", "bar"], "\\\\foo\\bar\\"),
    (&["//foo/", "bar"], "\\\\foo\\bar\\"),
    (&["//foo", "/bar"], "\\\\foo\\bar\\"),
    // UNC path expected - questionable
    (&["//foo", "", "bar"], "\\\\foo\\bar\\"),
    (&["//foo/", "", "bar"], "\\\\foo\\bar\\"),
    (&["//foo/", "", "/bar"], "\\\\foo\\bar\\"),
    // UNC path expected - even more questionable
    (&["", "//foo", "bar"], "\\\\foo\\bar\\"),
    (&["", "//foo/", "bar"], "\\\\foo\\bar\\"),
    (&["", "//foo/", "/bar"], "\\\\foo\\bar\\"),
    // No UNC path expected (no double slash in first component)
    (&["\\", "foo/bar"], "\\foo\\bar"),
    (&["\\", "/foo/bar"], "\\foo\\bar"),
    (&["", "/", "/foo/bar"], "\\foo\\bar"),
    // No UNC path expected (no non-slashes in first component -
    // questionable)
    (&["//", "foo/bar"], "\\foo\\bar"),
    (&["//", "/foo/bar"], "\\foo\\bar"),
    (&["\\\\", "/", "/foo/bar"], "\\foo\\bar"),
    (&["//"], "\\"),
    // No UNC path expected (share name missing - questionable).
    (&["//foo"], "\\foo"),
    (&["//foo/"], "\\foo\\"),
    (&["//foo", "/"], "\\foo\\"),
    (&["//foo", "", "/"], "\\foo\\"),
    // No UNC path expected (too many leading slashes - questionable)
    (&["///foo/bar"], "\\foo\\bar"),
    (&["////foo", "bar"], "\\foo\\bar"),
    (&["\\\\\\/foo/bar"], "\\foo\\bar"),
    // Drive-relative vs drive-absolute paths. This merely describes the
    // status quo, rather than being obviously right
    (&["c:"], "c:."),
    (&["c:."], "c:."),
    (&["c:", ""], "c:."),
    (&["", "c:"], "c:."),
    (&["c:.", "/"], "c:.\\"),
    (&["c:.", "file"], "c:file"),
    (&["c:", "/"], "c:\\"),
    (&["c:", "file"], "c:\\file"),
    // Path traversal in previous versions of Node.js.
    (&["./upload", "/../C:/Windows"], ".\\C:\\Windows"),
    (&["upload", "../", "C:foo"], ".\\C:foo"),
    (&["test/..", "??/D:/Test"], ".\\??\\D:\\Test"),
    (&["test", "..", "D:"], ".\\D:"),
    (&["test", "..", "D:\\"], ".\\D:\\"),
    (&["test", "..", "D:foo"], ".\\D:foo"),
];

#[test]
fn test_path_join_windows() {
    WINDOWS_SPECIFIC_JOIN_TESTS
        .iter()
        .for_each(|(paths, expected)| {
            let mut path = Utf8PathBuf::new();
            for p in *paths {
                path = path.join_win32(p);
            }
            assert_eq!(path.normalize_win32().as_str(), *expected);
        });
}

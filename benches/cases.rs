#![allow(dead_code)]

use node_path::{DriveCwd, NodeHost, PathContext};

pub const SHORT_POSIX_CLEAN: &str = "/srv/app/src/lib.rs";
pub const SHORT_POSIX_DIRTY: &str = "/srv//app/./src/../lib.rs";
pub const SHORT_WIN32_CLEAN: &str = "C:\\srv\\app\\src\\lib.rs";
pub const SHORT_WIN32_DIRTY: &str = "C:\\srv\\app\\.\\src\\..\\lib.rs";
pub const POSIX_UNICODE: &str = "/资料/项目/İ/文件.tar.gz";
pub const WIN32_UNICODE: &str = "C:\\资料\\项目\\İ\\文件.tar.gz";
pub const POSIX_GLOB: &str = "/资料/**/文件.*";
pub const WIN32_GLOB: &str = "C:\\资料\\**\\文件.*";
pub const GLOB_MISS: &str = "**/definitely-not-present/*.map";

pub const LONG_POSIX_DIRTY: &str = concat!(
    "/workspace//segment-00/./segment-01/../segment-02/segment-03/segment-04/segment-05/",
    "segment-06/segment-07/segment-08/segment-09/segment-10/segment-11/segment-12/segment-13/",
    "segment-14/segment-15/segment-16/segment-17/segment-18/segment-19/segment-20/segment-21/",
    "segment-22/segment-23/segment-24/segment-25/segment-26/segment-27/segment-28/segment-29/",
    "segment-30/segment-31/segment-32/segment-33/segment-34/segment-35/segment-36/segment-37/",
    "segment-38/segment-39/segment-40/segment-41/segment-42/segment-43/segment-44/segment-45/",
    "segment-46/segment-47/segment-48/segment-49/segment-50/segment-51/segment-52/segment-53/",
    "segment-54/segment-55/segment-56/segment-57/segment-58/segment-59/segment-60/segment-61/",
    "segment-62/segment-63/segment-64/segment-65/segment-66/segment-67/segment-68/segment-69/",
    "segment-70/segment-71/segment-72/segment-73/segment-74/segment-75/segment-76/segment-77/",
    "segment-78/segment-79/final.bundle.js",
);

pub const LONG_WIN32_DIRTY: &str = concat!(
    "C:\\workspace\\\\segment-00\\.\\segment-01\\..\\segment-02\\segment-03\\segment-04\\segment-05\\",
    "segment-06\\segment-07\\segment-08\\segment-09\\segment-10\\segment-11\\segment-12\\segment-13\\",
    "segment-14\\segment-15\\segment-16\\segment-17\\segment-18\\segment-19\\segment-20\\segment-21\\",
    "segment-22\\segment-23\\segment-24\\segment-25\\segment-26\\segment-27\\segment-28\\segment-29\\",
    "segment-30\\segment-31\\segment-32\\segment-33\\segment-34\\segment-35\\segment-36\\segment-37\\",
    "segment-38\\segment-39\\segment-40\\segment-41\\segment-42\\segment-43\\segment-44\\segment-45\\",
    "segment-46\\segment-47\\segment-48\\segment-49\\segment-50\\segment-51\\segment-52\\segment-53\\",
    "segment-54\\segment-55\\segment-56\\segment-57\\segment-58\\segment-59\\segment-60\\segment-61\\",
    "segment-62\\segment-63\\segment-64\\segment-65\\segment-66\\segment-67\\segment-68\\segment-69\\",
    "segment-70\\segment-71\\segment-72\\segment-73\\segment-74\\segment-75\\segment-76\\segment-77\\",
    "segment-78\\segment-79\\final.bundle.js",
);

pub fn posix_context() -> PathContext {
    PathContext::new(NodeHost::OtherPosix, "/workspace/project", vec![]).unwrap()
}

pub fn win32_context() -> PathContext {
    PathContext::new(
        NodeHost::Win32,
        "C:\\workspace\\project",
        vec![DriveCwd {
            device: "D:".into(),
            cwd: "D:\\drive-work".into(),
        }],
    )
    .unwrap()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchmarkCase {
    pub id: &'static str,
    pub namespace: &'static str,
    pub operation: &'static str,
    pub category: &'static str,
    pub fixture_version: u8,
}

macro_rules! case {
    ($namespace:literal, $operation:literal, $category:literal) => {
        BenchmarkCase {
            id: concat!(
                "path_ops/",
                $namespace,
                "/",
                $operation,
                "/",
                $category,
                "_v1"
            ),
            namespace: $namespace,
            operation: $operation,
            category: $category,
            fixture_version: 1,
        }
    };
}

pub const BENCHMARK_CASES: &[BenchmarkCase] = &[
    case!("posix", "resolve", "structural"),
    case!("win32", "resolve", "structural"),
    case!("posix", "normalize", "short_clean"),
    case!("posix", "normalize", "long_dirty"),
    case!("win32", "normalize", "short_clean"),
    case!("win32", "normalize", "long_dirty"),
    case!("posix", "is_absolute", "short_clean"),
    case!("win32", "is_absolute", "short_clean"),
    case!("posix", "join", "short_dirty"),
    case!("win32", "join", "short_dirty"),
    case!("posix", "relative", "structural"),
    case!("win32", "relative", "structural"),
    case!("posix", "to_namespaced_path", "structural"),
    case!("win32", "to_namespaced_path", "structural"),
    case!("posix", "dirname", "long_clean"),
    case!("win32", "dirname", "long_clean"),
    case!("posix", "basename", "suffix_dot"),
    case!("win32", "basename", "suffix_dot"),
    case!("posix", "extname", "unicode"),
    case!("win32", "extname", "unicode"),
    case!("posix", "format", "structural"),
    case!("win32", "format", "structural"),
    case!("posix", "parse", "long_clean"),
    case!("win32", "parse", "long_clean"),
    case!("posix", "matches_glob", "unicode_hit"),
    case!("posix", "matches_glob", "long_miss"),
    case!("win32", "matches_glob", "unicode_hit"),
    case!("win32", "matches_glob", "long_miss"),
];

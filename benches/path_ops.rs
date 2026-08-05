mod cases;

use divan::{Bencher, black_box};
use node_path::{PathObject, posix, win32};

fn main() {
    divan::main();
}

macro_rules! bench_case {
    ($function:ident, $id:literal, $body:expr) => {
        #[divan::bench(name = $id)]
        fn $function(bencher: Bencher) {
            bencher.bench_local(|| black_box($body));
        }
    };
}

bench_case!(
    posix_normalize_short_clean,
    "path_ops/posix/normalize/short_clean_v1",
    posix::normalize(black_box(cases::SHORT_POSIX_CLEAN))
);
bench_case!(
    posix_normalize_long_dirty,
    "path_ops/posix/normalize/long_dirty_v1",
    posix::normalize(black_box(cases::LONG_POSIX_DIRTY))
);
bench_case!(
    win32_normalize_short_clean,
    "path_ops/win32/normalize/short_clean_v1",
    win32::normalize(black_box(cases::SHORT_WIN32_CLEAN))
);
bench_case!(
    win32_normalize_long_dirty,
    "path_ops/win32/normalize/long_dirty_v1",
    win32::normalize(black_box(cases::LONG_WIN32_DIRTY))
);
bench_case!(
    posix_is_absolute,
    "path_ops/posix/is_absolute/short_clean_v1",
    posix::is_absolute(black_box(cases::SHORT_POSIX_CLEAN))
);
bench_case!(
    win32_is_absolute,
    "path_ops/win32/is_absolute/short_clean_v1",
    win32::is_absolute(black_box(cases::SHORT_WIN32_CLEAN))
);
bench_case!(
    posix_join,
    "path_ops/posix/join/short_dirty_v1",
    posix::join(black_box(&["/srv", "app/./src", "../lib.rs"]))
);
bench_case!(
    win32_join,
    "path_ops/win32/join/short_dirty_v1",
    win32::join(black_box(&["C:\\srv", "app\\.\\src", "..\\lib.rs"]))
);
bench_case!(
    posix_dirname,
    "path_ops/posix/dirname/long_clean_v1",
    posix::dirname(black_box(cases::LONG_POSIX_DIRTY))
);
bench_case!(
    win32_dirname,
    "path_ops/win32/dirname/long_clean_v1",
    win32::dirname(black_box(cases::LONG_WIN32_DIRTY))
);
bench_case!(
    posix_basename,
    "path_ops/posix/basename/suffix_dot_v1",
    posix::basename(black_box(cases::POSIX_UNICODE), Some(".gz"))
);
bench_case!(
    win32_basename,
    "path_ops/win32/basename/suffix_dot_v1",
    win32::basename(black_box(cases::WIN32_UNICODE), Some(".gz"))
);
bench_case!(
    posix_extname,
    "path_ops/posix/extname/unicode_v1",
    posix::extname(black_box(cases::POSIX_UNICODE))
);
bench_case!(
    win32_extname,
    "path_ops/win32/extname/unicode_v1",
    win32::extname(black_box(cases::WIN32_UNICODE))
);
bench_case!(
    posix_parse,
    "path_ops/posix/parse/long_clean_v1",
    posix::parse(black_box(cases::LONG_POSIX_DIRTY))
);
bench_case!(
    win32_parse,
    "path_ops/win32/parse/long_clean_v1",
    win32::parse(black_box(cases::LONG_WIN32_DIRTY))
);

#[divan::bench(name = "path_ops/posix/resolve/structural_v1")]
fn posix_resolve(bencher: Bencher) {
    let context = cases::posix_context();
    bencher.bench_local(|| {
        black_box(posix::resolve_with_context(
            &context,
            black_box(&["src", "../target", "artifact"]),
        ))
    });
}

#[divan::bench(name = "path_ops/win32/resolve/structural_v1")]
fn win32_resolve(bencher: Bencher) {
    let context = cases::win32_context();
    bencher.bench_local(|| {
        black_box(win32::resolve_with_context(
            &context,
            black_box(&["D:src", "..\\target", "artifact"]),
        ))
    });
}

#[divan::bench(name = "path_ops/posix/relative/structural_v1")]
fn posix_relative(bencher: Bencher) {
    let context = cases::posix_context();
    bencher.bench_local(|| {
        black_box(posix::relative_with_context(
            &context,
            black_box("/workspace/project/src/lib"),
            black_box("/workspace/project/tests/fixtures"),
        ))
    });
}

#[divan::bench(name = "path_ops/win32/relative/structural_v1")]
fn win32_relative(bencher: Bencher) {
    let context = cases::win32_context();
    bencher.bench_local(|| {
        black_box(win32::relative_with_context(
            &context,
            black_box("C:\\workspace\\project\\src\\lib"),
            black_box("C:\\workspace\\project\\tests\\fixtures"),
        ))
    });
}

#[divan::bench(name = "path_ops/posix/to_namespaced_path/structural_v1")]
fn posix_to_namespaced_path(bencher: Bencher) {
    let context = cases::posix_context();
    bencher.bench_local(|| {
        black_box(posix::to_namespaced_path_with_context(
            &context,
            black_box(cases::SHORT_POSIX_DIRTY),
        ))
    });
}

#[divan::bench(name = "path_ops/win32/to_namespaced_path/structural_v1")]
fn win32_to_namespaced_path(bencher: Bencher) {
    let context = cases::win32_context();
    bencher.bench_local(|| {
        black_box(win32::to_namespaced_path_with_context(
            &context,
            black_box(cases::SHORT_WIN32_DIRTY),
        ))
    });
}

#[divan::bench(name = "path_ops/posix/format/structural_v1")]
fn posix_format(bencher: Bencher) {
    let object = PathObject {
        root: "/",
        dir: "/workspace/project/src",
        base: "",
        name: "lib",
        ext: "rs",
    };
    bencher.bench_local(|| black_box(posix::format(black_box(&object))));
}

#[divan::bench(name = "path_ops/win32/format/structural_v1")]
fn win32_format(bencher: Bencher) {
    let object = PathObject {
        root: "C:\\",
        dir: "C:\\workspace\\project\\src",
        base: "",
        name: "lib",
        ext: "rs",
    };
    bencher.bench_local(|| black_box(win32::format(black_box(&object))));
}

#[divan::bench(name = "path_ops/posix/matches_glob/unicode_hit_v1")]
fn posix_glob_hit(bencher: Bencher) {
    let context = cases::posix_context();
    bencher.bench_local(|| {
        black_box(posix::matches_glob_with_context(
            &context,
            black_box(cases::POSIX_UNICODE),
            black_box(cases::POSIX_GLOB),
        ))
    });
}

#[divan::bench(name = "path_ops/posix/matches_glob/long_miss_v1")]
fn posix_glob_miss(bencher: Bencher) {
    let context = cases::posix_context();
    bencher.bench_local(|| {
        black_box(posix::matches_glob_with_context(
            &context,
            black_box(cases::LONG_POSIX_DIRTY),
            black_box(cases::GLOB_MISS),
        ))
    });
}

#[divan::bench(name = "path_ops/win32/matches_glob/unicode_hit_v1")]
fn win32_glob_hit(bencher: Bencher) {
    let context = cases::win32_context();
    bencher.bench_local(|| {
        black_box(win32::matches_glob_with_context(
            &context,
            black_box(cases::WIN32_UNICODE),
            black_box(cases::WIN32_GLOB),
        ))
    });
}

#[divan::bench(name = "path_ops/win32/matches_glob/long_miss_v1")]
fn win32_glob_miss(bencher: Bencher) {
    let context = cases::win32_context();
    bencher.bench_local(|| {
        black_box(win32::matches_glob_with_context(
            &context,
            black_box(cases::LONG_WIN32_DIRTY),
            black_box(cases::GLOB_MISS),
        ))
    });
}

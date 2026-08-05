//! Node.js-compatible string path processing.
//!
//! Explicit [`posix`] and [`win32`] namespaces are host-independent. The crate-root facade selects
//! the same namespace Node selects for the compilation target. Context-free parsing functions do
//! not access the filesystem; functions that capture the environment return [`ContextError`].
//!
//! ```
//! use node_path::{NodeHost, PathContext, posix, win32};
//!
//! assert_eq!(posix::normalize("/a//b/../c"), "/a/c");
//! assert_eq!(win32::normalize("C:/a\\b\\..\\c"), "C:\\a\\c");
//!
//! let context = PathContext::new(NodeHost::OtherPosix, "/work", vec![])?;
//! assert_eq!(posix::resolve_with_context(&context, &["src", "../tests"]), "/work/tests");
//! # Ok::<(), node_path::ContextError>(())
//! ```

use std::borrow::Cow;

mod shared;

pub mod context;
pub mod error;
pub mod glob;
pub mod path_object;
pub mod posix;
pub mod win32;

pub use context::{DriveCwd, NodeHost, PathContext};
pub use error::{ContextError, GlobError};
pub use path_object::{ParsedPath, PathObject};

#[cfg(not(windows))]
pub use posix::{DELIMITER, SEP};
#[cfg(windows)]
pub use win32::{DELIMITER, SEP};

/// Normalizes a path using the target-selected Node namespace.
pub fn normalize(path: &str) -> Cow<'_, str> {
    if cfg!(windows) {
        win32::normalize(path)
    } else {
        posix::normalize(path)
    }
}

/// Tests absoluteness using the target-selected Node namespace.
#[must_use]
pub fn is_absolute(path: &str) -> bool {
    if cfg!(windows) {
        win32::is_absolute(path)
    } else {
        posix::is_absolute(path)
    }
}

/// Joins path fragments using the target-selected Node namespace.
pub fn join(paths: &[&str]) -> String {
    if cfg!(windows) {
        win32::join(paths)
    } else {
        posix::join(paths)
    }
}

/// Returns the directory portion using the target-selected Node namespace.
pub fn dirname(path: &str) -> &str {
    if cfg!(windows) {
        win32::dirname(path)
    } else {
        posix::dirname(path)
    }
}

/// Returns the final component and optionally removes a Node-compatible suffix.
pub fn basename<'a>(path: &'a str, suffix: Option<&str>) -> &'a str {
    if cfg!(windows) {
        win32::basename(path, suffix)
    } else {
        posix::basename(path, suffix)
    }
}

/// Returns the extension portion using the target-selected Node namespace.
pub fn extname(path: &str) -> &str {
    if cfg!(windows) {
        win32::extname(path)
    } else {
        posix::extname(path)
    }
}

/// Parses a path into five borrowed fields without allocating.
pub fn parse(path: &str) -> ParsedPath<'_> {
    if cfg!(windows) {
        win32::parse(path)
    } else {
        posix::parse(path)
    }
}

/// Formats a path object using Node field precedence.
pub fn format<S: AsRef<str>>(path_object: &PathObject<S>) -> String {
    if cfg!(windows) {
        win32::format(path_object)
    } else {
        posix::format(path_object)
    }
}

/// Resolves paths after capturing an immutable process context.
pub fn resolve(paths: &[&str]) -> Result<String, ContextError> {
    if cfg!(windows) {
        win32::resolve(paths)
    } else {
        posix::resolve(paths)
    }
}

/// Computes a relative path after capturing an immutable process context.
pub fn relative(from: &str, to: &str) -> Result<String, ContextError> {
    if cfg!(windows) {
        win32::relative(from, to)
    } else {
        posix::relative(from, to)
    }
}

/// Produces a namespaced path after capturing an immutable process context.
pub fn to_namespaced_path(path: &str) -> Result<Cow<'_, str>, ContextError> {
    if cfg!(windows) {
        win32::to_namespaced_path(path)
    } else {
        posix::to_namespaced_path(path)
    }
}

/// Matches a path with the pinned Node `matchesGlob` semantics.
pub fn matches_glob(path: &str, pattern: &str) -> Result<bool, GlobError> {
    if cfg!(windows) {
        win32::matches_glob(path, pattern)
    } else {
        posix::matches_glob(path, pattern)
    }
}

#[deprecated(note = "Node compatibility alias; use to_namespaced_path")]
/// Deprecated Node compatibility alias for [`to_namespaced_path`].
pub fn _make_long(path: &str) -> Result<Cow<'_, str>, ContextError> {
    to_namespaced_path(path)
}

//! Explicit, host-independent Node.js `path.posix` semantics.
//!
//! Use the `_with_context` variants when reproducible cwd behavior matters. Returned `&str` and
//! borrowed [`Cow`] values point into the supplied input whenever Node semantics permit it.

use std::borrow::Cow;

use crate::context::{NodeHost, PathContext};
use crate::error::{ContextError, GlobError};
use crate::path_object::{ParsedPath, PathObject};
use crate::shared::{is_posix_separator, normalize_string, sum_capacity};

pub const SEP: &str = "/";
pub const DELIMITER: &str = ":";

pub fn normalize(path: &str) -> Cow<'_, str> {
    if path.is_empty() {
        return Cow::Borrowed(".");
    }
    let absolute = path.starts_with('/');
    let trailing = path.ends_with('/');
    let mut normalized = normalize_string(path, !absolute, '/', is_posix_separator);

    if normalized.is_empty() {
        return if absolute {
            Cow::Borrowed("/")
        } else if trailing {
            Cow::Borrowed("./")
        } else {
            Cow::Borrowed(".")
        };
    }
    if trailing {
        normalized.push('/');
    }
    if absolute {
        normalized.insert(0, '/');
    }
    if normalized == path {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(normalized)
    }
}

#[must_use]
pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

pub fn join(paths: &[&str]) -> String {
    let mut joined =
        String::with_capacity(paths.iter().map(|path| path.len().saturating_add(1)).sum());
    for path in paths.iter().copied().filter(|path| !path.is_empty()) {
        if !joined.is_empty() {
            joined.push('/');
        }
        joined.push_str(path);
    }
    if joined.is_empty() {
        ".".to_owned()
    } else {
        normalize(&joined).into_owned()
    }
}

pub fn dirname(path: &str) -> &str {
    if path.is_empty() {
        return ".";
    }
    let bytes = path.as_bytes();
    let has_root = bytes[0] == b'/';
    let mut matched_slash = true;
    let mut end = None;
    for index in (1..bytes.len()).rev() {
        if bytes[index] == b'/' {
            if !matched_slash {
                end = Some(index);
                break;
            }
        } else {
            matched_slash = false;
        }
    }
    match end {
        None if has_root => "/",
        None => ".",
        Some(1) if has_root => "//",
        Some(end) => &path[..end],
    }
}

pub fn basename<'a>(path: &'a str, suffix: Option<&str>) -> &'a str {
    let bytes = path.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    if end == 0 {
        return "";
    }
    let start = path[..end].rfind('/').map_or(0, |index| index + 1);
    let base = &path[start..end];
    match suffix {
        Some(suffix) if !suffix.is_empty() && path == suffix => "",
        Some(suffix) if !suffix.is_empty() && base == suffix => base,
        Some(suffix) if !suffix.is_empty() => base.strip_suffix(suffix).unwrap_or(base),
        _ => base,
    }
}

fn component_ext(component: &str) -> &str {
    let Some(dot) = component.rfind('.') else {
        return "";
    };
    if dot == 0 || component == ".." {
        ""
    } else {
        &component[dot..]
    }
}

pub fn extname(path: &str) -> &str {
    component_ext(basename(path, None))
}

pub fn parse(path: &str) -> ParsedPath<'_> {
    if path.is_empty() {
        return PathObject::default();
    }
    let root = if path.starts_with('/') { "/" } else { "" };
    let root_end = root.len();
    let bytes = path.as_bytes();
    let mut end = bytes.len();
    while end > root_end && bytes[end - 1] == b'/' {
        end -= 1;
    }
    if end == root_end {
        return PathObject {
            root,
            dir: root,
            base: "",
            ext: "",
            name: "",
        };
    }

    let slash = path[root_end..end].rfind('/').map(|index| root_end + index);
    let base_start = slash.map_or(root_end, |index| index + 1);
    let dir = match slash {
        Some(0) => "/",
        Some(index) => &path[..index],
        None if !root.is_empty() => root,
        None => "",
    };
    let base = &path[base_start..end];
    let ext = component_ext(base);
    let name = &base[..base.len() - ext.len()];
    PathObject {
        root,
        dir,
        base,
        ext,
        name,
    }
}

pub fn format<S: AsRef<str>>(path_object: &PathObject<S>) -> String {
    let root = path_object.root.as_ref();
    let dir = if path_object.dir.as_ref().is_empty() {
        root
    } else {
        path_object.dir.as_ref()
    };
    let base_field = path_object.base.as_ref();
    let name = path_object.name.as_ref();
    let ext = path_object.ext.as_ref();
    let mut base = if base_field.is_empty() {
        String::with_capacity(name.len().saturating_add(ext.len()).saturating_add(1))
    } else {
        String::with_capacity(base_field.len())
    };
    if base_field.is_empty() {
        base.push_str(name);
        if !ext.is_empty() {
            if !ext.starts_with('.') {
                base.push('.');
            }
            base.push_str(ext);
        }
    } else {
        base.push_str(base_field);
    }
    if dir.is_empty() {
        return base;
    }
    let mut output = String::with_capacity(sum_capacity([dir.len(), base.len(), 1]));
    output.push_str(dir);
    if dir != root {
        output.push('/');
    }
    output.push_str(&base);
    output
}

fn cwd_for_context(context: &PathContext) -> Cow<'_, str> {
    if context.host() != NodeHost::Win32 {
        return Cow::Borrowed(context.cwd());
    }
    let converted = context.cwd().replace('\\', "/");
    match converted.find('/') {
        Some(index) => Cow::Owned(converted[index..].to_owned()),
        None => Cow::Borrowed(""),
    }
}

pub fn resolve_with_context(context: &PathContext, paths: &[&str]) -> String {
    let cwd = cwd_for_context(context);
    if (paths.is_empty() || (paths.len() == 1 && matches!(paths[0], "" | ".")))
        && cwd.starts_with('/')
    {
        return cwd.into_owned();
    }

    let capacity = paths
        .iter()
        .map(|path| path.len().saturating_add(1))
        .sum::<usize>()
        .saturating_add(cwd.len());
    let mut resolved = String::with_capacity(capacity);
    let mut absolute = false;
    for path in paths.iter().rev().copied() {
        if path.is_empty() {
            continue;
        }
        let mut next = String::with_capacity(path.len() + resolved.len() + 1);
        next.push_str(path);
        next.push('/');
        next.push_str(&resolved);
        resolved = next;
        absolute = path.starts_with('/');
        if absolute {
            break;
        }
    }
    if !absolute {
        let mut next = String::with_capacity(cwd.len() + resolved.len() + 1);
        next.push_str(&cwd);
        next.push('/');
        next.push_str(&resolved);
        absolute = cwd.starts_with('/');
        resolved = next;
    }
    let tail = normalize_string(&resolved, !absolute, '/', is_posix_separator);
    if absolute {
        format!("/{tail}")
    } else if tail.is_empty() {
        ".".to_owned()
    } else {
        tail
    }
}

pub fn resolve(paths: &[&str]) -> Result<String, ContextError> {
    let context = PathContext::from_env()?;
    Ok(resolve_with_context(&context, paths))
}

pub fn relative_with_context(context: &PathContext, from: &str, to: &str) -> String {
    if from == to {
        return String::new();
    }
    let from = resolve_with_context(context, &[from]);
    let to = resolve_with_context(context, &[to]);
    if from == to {
        return String::new();
    }
    let from_parts: Vec<_> = from[1..]
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let to_parts: Vec<_> = to[1..].split('/').filter(|part| !part.is_empty()).collect();
    let mut common = 0;
    while common < from_parts.len()
        && common < to_parts.len()
        && from_parts[common] == to_parts[common]
    {
        common += 1;
    }
    let mut output = String::new();
    for _ in common..from_parts.len() {
        if !output.is_empty() {
            output.push('/');
        }
        output.push_str("..");
    }
    for part in &to_parts[common..] {
        if !output.is_empty() {
            output.push('/');
        }
        output.push_str(part);
    }
    output
}

pub fn relative(from: &str, to: &str) -> Result<String, ContextError> {
    let context = PathContext::from_env()?;
    Ok(relative_with_context(&context, from, to))
}

pub fn to_namespaced_path_with_context<'a>(_context: &PathContext, path: &'a str) -> Cow<'a, str> {
    Cow::Borrowed(path)
}

pub fn to_namespaced_path(path: &str) -> Result<Cow<'_, str>, ContextError> {
    let context = PathContext::from_env()?;
    Ok(to_namespaced_path_with_context(&context, path))
}

#[deprecated(note = "Node compatibility alias; use to_namespaced_path")]
pub fn _make_long(path: &str) -> Result<Cow<'_, str>, ContextError> {
    to_namespaced_path(path)
}

#[deprecated(note = "Node compatibility alias; use to_namespaced_path_with_context")]
pub fn _make_long_with_context<'a>(context: &PathContext, path: &'a str) -> Cow<'a, str> {
    to_namespaced_path_with_context(context, path)
}

pub fn matches_glob(path: &str, pattern: &str) -> Result<bool, GlobError> {
    crate::glob::matches(false, NodeHost::current(), path, pattern)
}

pub fn matches_glob_with_context(
    context: &PathContext,
    path: &str,
    pattern: &str,
) -> Result<bool, GlobError> {
    crate::glob::matches(false, context.host(), path, pattern)
}

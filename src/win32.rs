//! Explicit, host-independent Node.js `path.win32` semantics.
//!
//! Drive-relative resolution and namespaced paths use [`PathContext`] rather than mutating hidden
//! process drive state, so Windows behavior is testable on every host.

use std::borrow::Cow;

use crate::context::{NodeHost, PathContext};
use crate::error::{ContextError, GlobError};
use crate::path_object::{ParsedPath, PathObject};
use crate::shared::{
    is_win32_separator, is_windows_device_root, is_windows_reserved_name, normalize_string,
    sum_capacity, win32_root_len,
};

pub const SEP: &str = "\\";
pub const DELIMITER: &str = ";";

#[derive(Debug)]
struct Root {
    device: String,
    root_end: usize,
    absolute: bool,
    unc_root_only: bool,
}

fn parse_root(path: &str) -> Root {
    let bytes = path.as_bytes();
    let mut root = Root {
        device: String::new(),
        root_end: 0,
        absolute: false,
        unc_root_only: false,
    };
    let Some(&first) = bytes.first() else {
        return root;
    };
    if is_win32_separator(first) {
        root.absolute = true;
        root.root_end = 1;
        if bytes.get(1).is_none_or(|byte| !is_win32_separator(*byte)) {
            return root;
        }
        let mut index = 2;
        let server_start = index;
        while bytes
            .get(index)
            .is_some_and(|byte| !is_win32_separator(*byte))
        {
            index += 1;
        }
        if index == server_start || index == bytes.len() {
            return root;
        }
        let server = &path[server_start..index];
        while bytes
            .get(index)
            .is_some_and(|byte| is_win32_separator(*byte))
        {
            index += 1;
        }
        let share_start = index;
        while bytes
            .get(index)
            .is_some_and(|byte| !is_win32_separator(*byte))
        {
            index += 1;
        }
        if index == share_start {
            return root;
        }
        if matches!(server, "." | "?") {
            root.device = format!("\\\\{server}");
            root.root_end = 4;
        } else {
            root.device = format!("\\\\{server}\\{}", &path[share_start..index]);
            root.root_end = index;
            root.unc_root_only = index == bytes.len();
        }
        return root;
    }
    if bytes.len() >= 2 && is_windows_device_root(first) && bytes[1] == b':' {
        root.device.push_str(&path[..2]);
        root.root_end = 2;
        if bytes.get(2).is_some_and(|byte| is_win32_separator(*byte)) {
            root.absolute = true;
            root.root_end = 3;
        }
    }
    root
}

fn first_reserved_device(path: &str) -> Option<(&str, usize)> {
    let colon = path.find(':')?;
    let candidate = &path[..colon];
    is_windows_reserved_name(candidate).then_some((&path[..=colon], colon + 1))
}

pub fn normalize(path: &str) -> Cow<'_, str> {
    let bytes = path.as_bytes();
    if bytes.is_empty() {
        return Cow::Borrowed(".");
    }
    if bytes.len() == 1 {
        return if bytes[0] == b'/' {
            Cow::Borrowed("\\")
        } else {
            Cow::Borrowed(path)
        };
    }

    let mut root = parse_root(path);
    if root.device == "\\\\?"
        && let Some(colon) = path.find(':')
    {
        let possible_device = &path[4..=colon];
        if is_windows_reserved_name(&possible_device[..possible_device.len() - 1]) {
            root.device = format!("\\\\?\\{possible_device}");
            root.root_end = 4 + possible_device.len();
        }
    }
    let mut reserved = false;
    if root.device.is_empty()
        && let Some((device, end)) = first_reserved_device(path)
    {
        root.device = device.to_owned();
        root.root_end = end;
        reserved = true;
    }
    if root.unc_root_only {
        return Cow::Owned(format!("{}\\", root.device));
    }

    let mut tail = if root.root_end < path.len() {
        normalize_string(
            &path[root.root_end..],
            !root.absolute,
            '\\',
            is_win32_separator,
        )
    } else {
        String::new()
    };
    if tail.is_empty() && !root.absolute {
        tail.push('.');
    }
    if !tail.is_empty() && bytes.last().is_some_and(|byte| is_win32_separator(*byte)) {
        tail.push('\\');
    }

    if !root.absolute && root.device.is_empty() && path.contains(':') {
        let drive_like = tail
            .as_bytes()
            .get(0..2)
            .is_some_and(|pair| is_windows_device_root(pair[0]) && pair[1] == b':');
        let dangerous_colon = path.match_indices(':').any(|(index, _)| {
            index + 1 == path.len()
                || bytes
                    .get(index + 1)
                    .is_some_and(|byte| is_win32_separator(*byte))
        });
        if drive_like || dangerous_colon {
            return Cow::Owned(format!(".\\{tail}"));
        }
    }
    if reserved {
        return Cow::Owned(format!(".\\{}{}", root.device, tail));
    }
    if !path.contains(':') && path.len() > 1 {
        let last_character = path
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index);
        if is_windows_reserved_name(&path[..last_character]) {
            return Cow::Owned(format!(".\\{tail}"));
        }
    }

    let output = if root.device.is_empty() {
        if root.absolute {
            format!("\\{tail}")
        } else {
            tail
        }
    } else if root.absolute {
        format!("{}\\{tail}", root.device)
    } else {
        format!("{}{}", root.device, tail)
    };
    if output == path {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(output)
    }
}

#[must_use]
pub fn is_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.first().is_some_and(|byte| is_win32_separator(*byte))
        || (bytes.len() > 2
            && is_windows_device_root(bytes[0])
            && bytes[1] == b':'
            && is_win32_separator(bytes[2]))
}

pub fn join(paths: &[&str]) -> String {
    let nonempty: Vec<_> = paths
        .iter()
        .copied()
        .filter(|path| !path.is_empty())
        .collect();
    if nonempty.is_empty() {
        return ".".to_owned();
    }
    let first = nonempty[0];
    let mut joined = nonempty.join("\\");
    let mut slash_count = first
        .as_bytes()
        .iter()
        .take_while(|byte| is_win32_separator(**byte))
        .count();
    let clearly_unc = slash_count == 2
        && first
            .as_bytes()
            .get(2)
            .is_some_and(|byte| !is_win32_separator(*byte));
    if !clearly_unc {
        slash_count = joined
            .as_bytes()
            .iter()
            .take_while(|byte| is_win32_separator(**byte))
            .count();
        if slash_count >= 2 {
            joined = format!("\\{}", &joined[slash_count..]);
        }
    }
    let contains_reserved_stream = joined.split('\\').any(|part| {
        part.find(':')
            .is_some_and(|colon| is_windows_reserved_name(&part[..colon]))
    });
    if contains_reserved_stream {
        joined.replace('/', "\\")
    } else {
        normalize(&joined).into_owned()
    }
}

pub fn dirname(path: &str) -> &str {
    if path.is_empty() {
        return ".";
    }
    let bytes = path.as_bytes();
    if bytes.len() == 1 {
        return if is_win32_separator(bytes[0]) {
            path
        } else {
            "."
        };
    }
    let root_end = win32_root_len(path);
    if root_end == path.len() && root_end > 0 {
        return path;
    }
    let mut matched_slash = true;
    let mut end = None;
    for index in (root_end..bytes.len()).rev() {
        if is_win32_separator(bytes[index]) {
            if !matched_slash {
                end = Some(index);
                break;
            }
        } else {
            matched_slash = false;
        }
    }
    match end {
        Some(end) => &path[..end],
        None if root_end > 0 => &path[..root_end],
        None => ".",
    }
}

pub fn basename<'a>(path: &'a str, suffix: Option<&str>) -> &'a str {
    let bytes = path.as_bytes();
    let drive_start = if bytes.len() >= 2 && is_windows_device_root(bytes[0]) && bytes[1] == b':' {
        2
    } else {
        0
    };
    let mut end = bytes.len();
    while end > drive_start && is_win32_separator(bytes[end - 1]) {
        end -= 1;
    }
    if end == drive_start {
        return "";
    }
    let start = bytes[drive_start..end]
        .iter()
        .rposition(|byte| is_win32_separator(*byte))
        .map_or(drive_start, |index| drive_start + index + 1);
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
    let root_end = win32_root_len(path);
    let root = &path[..root_end];
    let bytes = path.as_bytes();
    let mut end = bytes.len();
    while end > root_end && is_win32_separator(bytes[end - 1]) {
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
    let slash = bytes[root_end..end]
        .iter()
        .rposition(|byte| is_win32_separator(*byte))
        .map(|index| root_end + index);
    let base_start = slash.map_or(root_end, |index| index + 1);
    let dir = match slash {
        Some(index) if index + 1 == root_end => root,
        Some(index) => &path[..index],
        None if root_end > 0 => root,
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
    let mut base = String::new();
    if base_field.is_empty() {
        base.reserve(name.len().saturating_add(ext.len()).saturating_add(1));
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
        output.push('\\');
    }
    output.push_str(&base);
    output
}

fn context_cwd(context: &PathContext) -> Cow<'_, str> {
    if context.host() == NodeHost::Win32 {
        Cow::Borrowed(context.cwd())
    } else {
        Cow::Owned(context.cwd().replace('/', "\\"))
    }
}

pub fn resolve_with_context(context: &PathContext, paths: &[&str]) -> String {
    let mut resolved_device = String::new();
    let mut resolved_tail = String::new();
    let mut resolved_absolute = false;

    for index in (0..=paths.len()).rev() {
        let path = if index > 0 {
            let value = paths[index - 1];
            if value.is_empty() {
                continue;
            }
            Cow::Borrowed(value)
        } else if resolved_device.is_empty() {
            let cwd = context_cwd(context);
            if (paths.is_empty() || (paths.len() == 1 && matches!(paths[0], "" | ".")))
                && cwd
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| is_win32_separator(*byte))
            {
                return cwd.into_owned();
            }
            cwd
        } else {
            let candidate = context.drive_cwd(&resolved_device).unwrap_or(context.cwd());
            let candidate = if context.host() == NodeHost::Win32 {
                Cow::Borrowed(candidate)
            } else {
                Cow::Owned(candidate.replace('/', "\\"))
            };
            let matches_device = candidate
                .get(..2)
                .is_some_and(|device| device.eq_ignore_ascii_case(&resolved_device));
            if !matches_device
                && candidate
                    .as_bytes()
                    .get(2)
                    .is_some_and(|byte| *byte == b'\\')
            {
                Cow::Owned(format!("{resolved_device}\\"))
            } else {
                candidate
            }
        };

        let root = parse_root(&path);
        if !root.device.is_empty() {
            if !resolved_device.is_empty() && !root.device.eq_ignore_ascii_case(&resolved_device) {
                continue;
            }
            if resolved_device.is_empty() {
                resolved_device = root.device;
            }
        }
        if !resolved_absolute {
            let mut tail = String::with_capacity(
                path.len()
                    .saturating_sub(root.root_end)
                    .saturating_add(resolved_tail.len())
                    .saturating_add(1),
            );
            tail.push_str(&path[root.root_end..]);
            tail.push('\\');
            tail.push_str(&resolved_tail);
            resolved_tail = tail;
            resolved_absolute = root.absolute;
        }
        if resolved_absolute && !resolved_device.is_empty() {
            break;
        }
    }
    let tail = normalize_string(&resolved_tail, !resolved_absolute, '\\', is_win32_separator);
    let result = if resolved_absolute {
        format!("{resolved_device}\\{tail}")
    } else {
        format!("{resolved_device}{tail}")
    };
    if result.is_empty() {
        ".".to_owned()
    } else {
        result
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
    let from_resolved = resolve_with_context(context, &[from]);
    let to_resolved = resolve_with_context(context, &[to]);
    if from_resolved.eq_ignore_ascii_case(&to_resolved) {
        return String::new();
    }
    let from_parts: Vec<_> = from_resolved
        .split('\\')
        .filter(|part| !part.is_empty())
        .collect();
    let to_parts: Vec<_> = to_resolved
        .split('\\')
        .filter(|part| !part.is_empty())
        .collect();
    let mut common = 0;
    while common < from_parts.len()
        && common < to_parts.len()
        && from_parts[common].to_lowercase() == to_parts[common].to_lowercase()
    {
        common += 1;
    }
    if common == 0 {
        return to_resolved;
    }
    let mut output = Vec::with_capacity(from_parts.len() + to_parts.len() - common * 2);
    output.extend((common..from_parts.len()).map(|_| ".."));
    output.extend(to_parts[common..].iter().copied());
    output.join("\\")
}

pub fn relative(from: &str, to: &str) -> Result<String, ContextError> {
    let context = PathContext::from_env()?;
    Ok(relative_with_context(&context, from, to))
}

pub fn to_namespaced_path_with_context<'a>(context: &PathContext, path: &'a str) -> Cow<'a, str> {
    if path.is_empty() {
        return Cow::Borrowed(path);
    }
    let resolved = resolve_with_context(context, &[path]);
    let bytes = resolved.as_bytes();
    if bytes.len() <= 2 {
        return Cow::Borrowed(path);
    }
    if let Some(stripped) = resolved.strip_prefix("\\\\") {
        if !matches!(stripped.as_bytes().first(), Some(b'?') | Some(b'.')) {
            return Cow::Owned(format!("\\\\?\\UNC\\{stripped}"));
        }
    } else if bytes.len() > 2
        && is_windows_device_root(bytes[0])
        && bytes[1] == b':'
        && bytes[2] == b'\\'
    {
        return Cow::Owned(format!("\\\\?\\{resolved}"));
    }
    Cow::Owned(resolved)
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
    crate::glob::matches(true, NodeHost::current(), path, pattern)
}

pub fn matches_glob_with_context(
    context: &PathContext,
    path: &str,
    pattern: &str,
) -> Result<bool, GlobError> {
    crate::glob::matches(true, context.host(), path, pattern)
}

//! Safe string primitives shared by the explicit namespaces.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DotSegment {
    Current,
    Parent,
    Other,
}

#[inline]
pub(crate) const fn is_posix_separator(byte: u8) -> bool {
    byte == b'/'
}

#[inline]
pub(crate) const fn is_win32_separator(byte: u8) -> bool {
    byte == b'/' || byte == b'\\'
}

#[inline]
pub(crate) fn dot_segment(value: &str) -> DotSegment {
    match value.as_bytes() {
        b"." => DotSegment::Current,
        b".." => DotSegment::Parent,
        _ => DotSegment::Other,
    }
}

pub(crate) fn win32_root_len(path: &str) -> usize {
    let bytes = path.as_bytes();
    let Some(&first) = bytes.first() else {
        return 0;
    };

    if is_win32_separator(first) {
        if bytes.get(1).is_none_or(|byte| !is_win32_separator(*byte)) {
            return 1;
        }

        let mut index = 2;
        let server_start = index;
        while bytes
            .get(index)
            .is_some_and(|byte| !is_win32_separator(*byte))
        {
            index += 1;
        }
        if index == server_start {
            return 1;
        }
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
            return 1;
        }
        return if index < bytes.len() {
            index + 1
        } else {
            index
        };
    }

    if bytes.len() >= 2 && first.is_ascii_alphabetic() && bytes[1] == b':' {
        if bytes.get(2).is_some_and(|byte| is_win32_separator(*byte)) {
            3
        } else {
            2
        }
    } else {
        0
    }
}

#[inline]
pub(crate) fn slice(value: &str, start: usize, end: usize) -> &str {
    debug_assert!(start <= end);
    debug_assert!(value.is_char_boundary(start));
    debug_assert!(value.is_char_boundary(end));
    &value[start..end]
}

pub(crate) fn sum_capacity(lengths: impl IntoIterator<Item = usize>) -> usize {
    lengths
        .into_iter()
        .fold(0usize, |total, length| total.saturating_add(length))
}

pub(crate) fn normalize_string(
    path: &str,
    allow_above_root: bool,
    separator: char,
    is_separator: impl Fn(u8) -> bool,
) -> String {
    let bytes = path.as_bytes();
    let mut segments: Vec<&str> = Vec::new();
    let mut start = 0;

    for index in 0..=bytes.len() {
        if index != bytes.len() && !is_separator(bytes[index]) {
            continue;
        }
        let segment = slice(path, start, index);
        match dot_segment(segment) {
            DotSegment::Current => {}
            DotSegment::Parent => {
                if segments.last().is_some_and(|last| *last != "..") {
                    segments.pop();
                } else if allow_above_root {
                    segments.push("..");
                }
            }
            DotSegment::Other if !segment.is_empty() => segments.push(segment),
            DotSegment::Other => {}
        }
        start = index.saturating_add(1);
    }

    if segments.is_empty() {
        return String::new();
    }
    let capacity = sum_capacity(
        segments
            .iter()
            .map(|segment| segment.len())
            .chain(std::iter::once(segments.len().saturating_sub(1))),
    );
    let mut output = String::with_capacity(capacity);
    for (index, segment) in segments.into_iter().enumerate() {
        if index != 0 {
            output.push(separator);
        }
        output.push_str(segment);
    }
    output
}

#[inline]
pub(crate) const fn is_windows_device_root(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

pub(crate) fn is_windows_reserved_name(value: &str) -> bool {
    const NAMES: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "COM¹",
        "COM²", "COM³", "LPT¹", "LPT²", "LPT³",
    ];
    NAMES.iter().any(|name| value.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_ascii_separators() {
        assert!(is_posix_separator(b'/'));
        assert!(!is_posix_separator(b'\\'));
        assert!(is_win32_separator(b'/'));
        assert!(is_win32_separator(b'\\'));
        assert!(!is_win32_separator(b':'));
    }

    #[test]
    fn recognizes_drive_and_unc_roots() {
        assert_eq!(win32_root_len("C:\\work"), 3);
        assert_eq!(win32_root_len("C:work"), 2);
        assert_eq!(win32_root_len("\\\\server\\share\\file"), 15);
        assert_eq!(win32_root_len("relative"), 0);
    }

    #[test]
    fn classifies_dot_segments() {
        assert_eq!(dot_segment("."), DotSegment::Current);
        assert_eq!(dot_segment(".."), DotSegment::Parent);
        assert_eq!(dot_segment("..."), DotSegment::Other);
        assert_eq!(dot_segment("文件"), DotSegment::Other);
    }

    #[test]
    fn slicing_stays_on_utf8_boundaries() {
        let value = "a/文件.txt";
        assert_eq!(slice(value, 2, value.len()), "文件.txt");
        assert_eq!(slice(value, 2, 2), "");
    }

    #[test]
    fn capacity_planning_saturates() {
        assert_eq!(sum_capacity([1, 2, 3]), 6);
        assert_eq!(sum_capacity([usize::MAX, 1]), usize::MAX);
    }
}

//! Pinned brace-expansion behavior.
//!
//! Derived from brace-expansion 5.0.5 as bundled by minimatch 10.2.5. See
//! `THIRD_PARTY_NOTICES.md`.

use crate::error::GlobError;

const MAX_DEPTH: usize = 32;
const MAX_EXPANSIONS: usize = 1_024;

pub(crate) fn expand(pattern: &str) -> Result<Vec<String>, GlobError> {
    let mut output = Vec::new();
    expand_into(pattern, 0, &mut output)?;
    if output.is_empty() {
        output.push(pattern.to_owned());
    }
    Ok(output)
}

fn expand_into(pattern: &str, depth: usize, output: &mut Vec<String>) -> Result<(), GlobError> {
    if depth > MAX_DEPTH || output.len() >= MAX_EXPANSIONS {
        return Err(GlobError::MatcherInvariant);
    }
    let Some((open, close)) = first_brace(pattern) else {
        output.push(pattern.to_owned());
        return Ok(());
    };
    let prefix = &pattern[..open];
    let body = &pattern[open + 1..close];
    let suffix = &pattern[close + 1..];
    let alternatives = brace_alternatives(body);
    if alternatives.is_empty() {
        output.push(pattern.to_owned());
        return Ok(());
    }
    for alternative in alternatives {
        let mut expanded = String::with_capacity(prefix.len() + alternative.len() + suffix.len());
        expanded.push_str(prefix);
        expanded.push_str(&alternative);
        expanded.push_str(suffix);
        expand_into(&expanded, depth + 1, output)?;
    }
    Ok(())
}

fn first_brace(pattern: &str) -> Option<(usize, usize)> {
    let mut open = None;
    let mut depth = 0;
    for (index, character) in pattern.char_indices() {
        match character {
            '{' if open.is_none() => {
                open = Some(index);
                depth = 1;
            }
            '{' if open.is_some() => depth += 1,
            '}' if open.is_some() => {
                depth -= 1;
                if depth == 0 {
                    return open.map(|open| (open, index));
                }
            }
            _ => {}
        }
    }
    None
}

fn brace_alternatives(body: &str) -> Vec<String> {
    let comma = split_top_level(body, ',');
    if comma.len() > 1 {
        return comma.into_iter().map(str::to_owned).collect();
    }
    let range = split_top_level_double_dot(body);
    if !(2..=3).contains(&range.len()) {
        return Vec::new();
    }
    let step = range
        .get(2)
        .and_then(|step| step.parse::<i64>().ok())
        .filter(|step| *step != 0)
        .map_or(1, i64::abs);
    if let (Ok(start), Ok(end)) = (range[0].parse::<i64>(), range[1].parse::<i64>()) {
        let width = range[0].len().max(range[1].len());
        let padded = range[0].starts_with('0') || range[1].starts_with('0');
        let direction = if start <= end { step } else { -step };
        let mut value = start;
        let mut values = Vec::new();
        while (direction > 0 && value <= end) || (direction < 0 && value >= end) {
            values.push(if padded {
                format!("{value:0width$}")
            } else {
                value.to_string()
            });
            if values.len() >= MAX_EXPANSIONS {
                break;
            }
            value += direction;
        }
        return values;
    }
    let mut start_chars = range[0].chars();
    let mut end_chars = range[1].chars();
    let (Some(start), None, Some(end), None) = (
        start_chars.next(),
        start_chars.next(),
        end_chars.next(),
        end_chars.next(),
    ) else {
        return Vec::new();
    };
    let start = start as i64;
    let end = end as i64;
    let direction = if start <= end { step } else { -step };
    let mut value = start;
    let mut values = Vec::new();
    while (direction > 0 && value <= end) || (direction < 0 && value >= end) {
        if let Some(character) = char::from_u32(value as u32) {
            values.push(character.to_string());
        }
        if values.len() >= MAX_EXPANSIONS {
            break;
        }
        value += direction;
    }
    values
}

fn split_top_level(value: &str, needle: char) -> Vec<&str> {
    let mut output = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => depth -= 1,
            character if character == needle && depth == 0 => {
                output.push(&value[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    output.push(&value[start..]);
    output
}

fn split_top_level_double_dot(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut output = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'.' if depth == 0 && bytes.get(index + 1) == Some(&b'.') => {
                output.push(&value[start..index]);
                index += 1;
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    output.push(&value[start..]);
    output
}

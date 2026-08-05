//! Pinned minimatch compilation behavior.
//!
//! The option surface follows minimatch 10.2.5 as bundled by the pinned Node
//! revision. See `THIRD_PARTY_NOTICES.md`.

use crate::error::GlobError;

pub(crate) fn compile(pattern: &str, win32: bool) -> Result<String, GlobError> {
    let separator = if win32 { r"[\\/]" } else { "/" };
    let not_separator = if win32 { r"[^\\/]" } else { "[^/]" };
    let body = compile_fragment(pattern, separator, not_separator)?;
    Ok(format!(r"^(?:{body})$"))
}

fn compile_fragment(
    pattern: &str,
    separator: &str,
    not_separator: &str,
) -> Result<String, GlobError> {
    let characters: Vec<char> = pattern.chars().collect();
    let mut output = String::new();
    let mut index = 0;
    let mut segment_start = true;
    while index < characters.len() {
        let character = characters[index];
        if is_separator(character, separator) {
            output.push_str(separator);
            segment_start = true;
            index += 1;
            continue;
        }
        if matches!(character, '@' | '+' | '?' | '*' | '!')
            && characters.get(index + 1) == Some(&'(')
            && let Some(close) = closing_paren(&characters, index + 1)
        {
            let inner: String = characters[index + 2..close].iter().collect();
            let alternatives = split_alternatives(&inner)
                .into_iter()
                .map(|part| compile_fragment(part, separator, not_separator))
                .collect::<Result<Vec<_>, _>>()?;
            let joined = alternatives.join("|");
            match character {
                '@' => output.push_str(&format!("(?:{joined})")),
                '+' => output.push_str(&format!("(?:{joined})+")),
                '?' => output.push_str(&format!("(?:{joined})?")),
                '*' => output.push_str(&format!("(?:{joined})*")),
                '!' => output.push_str(&format!("(?!(?:{joined})$){not_separator}*")),
                _ => unreachable!(),
            }
            segment_start = false;
            index = close + 1;
            continue;
        }
        match character {
            '*' => {
                let globstar = characters.get(index + 1) == Some(&'*');
                if segment_start {
                    output.push_str(r"(?!\.)");
                }
                if globstar {
                    while characters.get(index + 1) == Some(&'*') {
                        index += 1;
                    }
                    if characters
                        .get(index + 1)
                        .is_some_and(|next| is_separator(*next, separator))
                    {
                        output.push_str(&format!("(?:{not_separator}+{separator})*"));
                        index += 1;
                        segment_start = true;
                    } else {
                        output.push_str(".*");
                        segment_start = false;
                    }
                } else {
                    output.push_str(not_separator);
                    output.push('*');
                    segment_start = false;
                }
            }
            '?' => {
                if segment_start {
                    output.push_str(r"(?!\.)");
                }
                output.push_str(not_separator);
                segment_start = false;
            }
            '[' => {
                if let Some(close) = characters[index + 1..]
                    .iter()
                    .position(|character| *character == ']')
                    .map(|offset| index + 1 + offset)
                {
                    if segment_start {
                        output.push_str(r"(?!\.)");
                    }
                    output.push('[');
                    let mut start = index + 1;
                    if matches!(characters.get(start), Some('!') | Some('^')) {
                        output.push('^');
                        start += 1;
                    }
                    for class_character in &characters[start..close] {
                        if *class_character == '\\' {
                            output.push_str(r"\\");
                        } else {
                            output.push(*class_character);
                        }
                    }
                    output.push(']');
                    index = close;
                    segment_start = false;
                } else {
                    output.push_str(r"\[");
                    segment_start = false;
                }
            }
            _ => {
                push_regex_literal(&mut output, character);
                segment_start = false;
            }
        }
        index += 1;
    }
    Ok(output)
}

fn is_separator(character: char, separator: &str) -> bool {
    character == '/' || (separator != "/" && character == '\\')
}

fn closing_paren(characters: &[char], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (index, character) in characters.iter().enumerate().skip(open) {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_alternatives(value: &str) -> Vec<&str> {
    let mut output = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            '|' if depth == 0 => {
                output.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    output.push(&value[start..]);
    output
}

fn push_regex_literal(output: &mut String, character: char) {
    if matches!(
        character,
        '.' | '+' | '^' | '$' | '(' | ')' | '|' | '{' | '}' | '\\'
    ) {
        output.push('\\');
    }
    output.push(character);
}

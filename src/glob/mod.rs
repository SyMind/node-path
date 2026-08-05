//! Pinned Node.js `matchesGlob` compatibility implementation.
//!
//! Patterns use the Node/minimatch separator, globstar, brace, extglob, class, case-mode, and
//! 65,536 UTF-16-unit resource rules. The implementation does not maintain global matcher state.

mod brace;
mod minimatch;

use crate::context::NodeHost;
use crate::error::GlobError;

const MAX_PATTERN_UTF16_UNITS: usize = 65_536;

pub(crate) fn matches(
    win32: bool,
    host: NodeHost,
    path: &str,
    pattern: &str,
) -> Result<bool, GlobError> {
    let utf16_units = pattern.encode_utf16().count();
    if utf16_units > MAX_PATTERN_UTF16_UNITS {
        return Err(GlobError::PatternTooLong {
            utf16_units,
            maximum: MAX_PATTERN_UTF16_UNITS,
        });
    }
    let path_utf16: Vec<u16> = path.encode_utf16().collect();
    let flags = if matches!(host, NodeHost::Win32 | NodeHost::Darwin) {
        "iu"
    } else {
        "u"
    };
    for expanded in brace::expand(pattern)? {
        let source = minimatch::compile(&expanded, win32)?;
        let regex =
            regress::Regex::with_flags(&source, flags).map_err(|_| GlobError::MatcherInvariant)?;
        if regex.find_from_utf16(&path_utf16, 0).next().is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

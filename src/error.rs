//! Public error types.

use std::fmt;
use std::path::PathBuf;

/// Failure while capturing a deterministic Node execution context.
#[derive(Debug)]
pub enum ContextError {
    /// The operating system did not provide the current directory.
    CurrentDirectory(std::io::Error),
    /// The current directory is outside the crate's Unicode string domain.
    NonUnicodeCurrentDirectory(PathBuf),
    /// A drive key was not one ASCII letter followed by `:`.
    InvalidDriveDevice(String),
    /// Two drive keys were equal under ASCII case folding.
    DuplicateDriveDevice(String),
}

impl fmt::Display for ContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDirectory(error) => {
                write!(formatter, "failed to read the current directory: {error}")
            }
            Self::NonUnicodeCurrentDirectory(path) => write!(
                formatter,
                "current directory is not valid Unicode: {}",
                path.display()
            ),
            Self::InvalidDriveDevice(device) => {
                write!(formatter, "invalid Windows drive device: {device}")
            }
            Self::DuplicateDriveDevice(device) => {
                write!(formatter, "duplicate Windows drive device: {device}")
            }
        }
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CurrentDirectory(error) => Some(error),
            _ => None,
        }
    }
}

/// Failure while compiling or executing a Node-compatible glob pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobError {
    /// The pattern exceeds Node's limit in JavaScript UTF-16 code units.
    PatternTooLong { utf16_units: usize, maximum: usize },
    /// The internal matcher violated an invariant.
    MatcherInvariant,
}

impl fmt::Display for GlobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PatternTooLong {
                utf16_units,
                maximum,
            } => write!(
                formatter,
                "glob pattern contains {utf16_units} UTF-16 units; maximum is {maximum}"
            ),
            Self::MatcherInvariant => formatter.write_str("glob matcher invariant violated"),
        }
    }
}

impl std::error::Error for GlobError {}

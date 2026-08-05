//! Deterministic Node execution context.

use std::collections::HashSet;

use crate::error::ContextError;

/// The platform on which Node semantics are being evaluated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeHost {
    Win32,
    Darwin,
    OtherPosix,
}

impl NodeHost {
    /// Returns the Node host corresponding to the compilation target.
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Win32
        } else if cfg!(target_os = "macos") {
            Self::Darwin
        } else {
            Self::OtherPosix
        }
    }
}

/// A snapshot of Node's hidden per-drive current-directory state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveCwd {
    pub device: String,
    pub cwd: String,
}

/// Immutable environment inputs used by context-dependent path operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathContext {
    host: NodeHost,
    cwd: String,
    drive_cwds: Vec<DriveCwd>,
}

impl PathContext {
    /// Constructs and validates a context without reading process state.
    pub fn new(
        host: NodeHost,
        cwd: impl Into<String>,
        drive_cwds: Vec<DriveCwd>,
    ) -> Result<Self, ContextError> {
        let mut seen = HashSet::with_capacity(drive_cwds.len());
        for drive in &drive_cwds {
            let bytes = drive.device.as_bytes();
            if bytes.len() != 2 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' {
                return Err(ContextError::InvalidDriveDevice(drive.device.clone()));
            }
            let folded = bytes[0].to_ascii_uppercase();
            if !seen.insert(folded) {
                return Err(ContextError::DuplicateDriveDevice(drive.device.clone()));
            }
        }

        Ok(Self {
            host,
            cwd: cwd.into(),
            drive_cwds,
        })
    }

    /// Captures the current directory and Windows hidden drive directories once.
    pub fn from_env() -> Result<Self, ContextError> {
        let cwd_path = std::env::current_dir().map_err(ContextError::CurrentDirectory)?;
        let cwd = cwd_path
            .clone()
            .into_os_string()
            .into_string()
            .map_err(|_| ContextError::NonUnicodeCurrentDirectory(cwd_path))?;

        let mut drive_cwds = Vec::new();
        if NodeHost::current() == NodeHost::Win32 {
            for (key, value) in std::env::vars_os() {
                let Some(key) = key.to_str() else {
                    continue;
                };
                let bytes = key.as_bytes();
                if bytes.len() != 3
                    || bytes[0] != b'='
                    || !bytes[1].is_ascii_alphabetic()
                    || bytes[2] != b':'
                {
                    continue;
                }
                let Ok(cwd) = value.into_string() else {
                    continue;
                };
                drive_cwds.push(DriveCwd {
                    device: key[1..].to_owned(),
                    cwd,
                });
            }
        }

        Self::new(NodeHost::current(), cwd, drive_cwds)
    }

    #[must_use]
    pub const fn host(&self) -> NodeHost {
        self.host
    }

    #[must_use]
    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    #[must_use]
    pub fn drive_cwds(&self) -> &[DriveCwd] {
        &self.drive_cwds
    }

    pub(crate) fn drive_cwd(&self, device: &str) -> Option<&str> {
        self.drive_cwds
            .iter()
            .find(|entry| entry.device.eq_ignore_ascii_case(device))
            .map(|entry| entry.cwd.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_lookup_is_ascii_case_insensitive() {
        let context = PathContext::new(
            NodeHost::Win32,
            "C:\\root",
            vec![DriveCwd {
                device: "c:".into(),
                cwd: "C:\\work".into(),
            }],
        )
        .unwrap();

        assert_eq!(context.drive_cwd("C:"), Some("C:\\work"));
    }
}

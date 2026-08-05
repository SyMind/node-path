use node_path::{ContextError, DriveCwd, NodeHost, PathContext};

#[test]
fn detects_the_node_host_for_the_current_target() {
    let expected = if cfg!(windows) {
        NodeHost::Win32
    } else if cfg!(target_os = "macos") {
        NodeHost::Darwin
    } else {
        NodeHost::OtherPosix
    };
    assert_eq!(NodeHost::current(), expected);
}

#[test]
fn accepts_empty_cwd_and_exposes_an_immutable_snapshot() {
    let mut original_cwd = String::new();
    let mut original_drives = vec![DriveCwd {
        device: "C:".into(),
        cwd: "D:\\mismatched-but-retained".into(),
    }];
    let context = PathContext::new(
        NodeHost::Win32,
        original_cwd.clone(),
        original_drives.clone(),
    )
    .unwrap();

    original_cwd.push_str("changed");
    original_drives[0].cwd.push_str("\\changed");

    assert_eq!(context.cwd(), "");
    assert_eq!(context.host(), NodeHost::Win32);
    assert_eq!(context.drive_cwds()[0].device, "C:");
    assert_eq!(context.drive_cwds()[0].cwd, "D:\\mismatched-but-retained");
}

#[test]
fn rejects_malformed_and_ascii_case_duplicate_devices() {
    let malformed = PathContext::new(
        NodeHost::Win32,
        "C:\\work",
        vec![DriveCwd {
            device: "CC:".into(),
            cwd: "C:\\work".into(),
        }],
    )
    .unwrap_err();
    assert!(matches!(
        malformed,
        ContextError::InvalidDriveDevice(ref device) if device == "CC:"
    ));

    let duplicate = PathContext::new(
        NodeHost::Win32,
        "C:\\work",
        vec![
            DriveCwd {
                device: "c:".into(),
                cwd: "C:\\one".into(),
            },
            DriveCwd {
                device: "C:".into(),
                cwd: "C:\\two".into(),
            },
        ],
    )
    .unwrap_err();
    assert!(matches!(
        duplicate,
        ContextError::DuplicateDriveDevice(ref device) if device == "C:"
    ));
}

#[test]
fn context_errors_are_actionable_and_dependency_independent() {
    let invalid = ContextError::InvalidDriveDevice("1:".into());
    assert_eq!(invalid.to_string(), "invalid Windows drive device: 1:");
    assert!(std::error::Error::source(&invalid).is_none());

    let non_unicode = ContextError::NonUnicodeCurrentDirectory("bad-cwd".into());
    assert!(non_unicode.to_string().contains("not valid Unicode"));
}

#[test]
fn environment_context_is_a_self_contained_snapshot() {
    let context = PathContext::from_env().unwrap();
    let cwd = context.cwd().to_owned();
    let drives = context.drive_cwds().to_vec();

    assert_eq!(context.cwd(), cwd);
    assert_eq!(context.drive_cwds(), drives);
}

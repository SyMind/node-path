use node_path::{NodeHost, PathContext, posix, win32};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};

fn replay_config(seed: u64) -> Config {
    Config {
        cases: 128,
        rng_seed: RngSeed::Fixed(seed),
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(replay_config(0x3f42_cfac))]

    #[test]
    fn normalization_is_idempotent(path in "[a-zA-Z0-9._/\\\\]{0,96}") {
        let posix_once = posix::normalize(&path).into_owned();
        let posix_twice = posix::normalize(&posix_once);
        prop_assert_eq!(posix_twice.as_ref(), posix_once.as_str());

        let win_once = win32::normalize(&path).into_owned();
        let win_twice = win32::normalize(&win_once);
        prop_assert_eq!(win_twice.as_ref(), win_once.as_str());
    }

    #[test]
    fn parse_then_format_preserves_clean_paths(
        absolute in any::<bool>(),
        segments in prop::collection::vec("[a-zA-Z0-9_]{1,8}", 1..6),
        extension in prop::option::of("[a-z]{1,4}"),
    ) {
        let mut leaf = segments.last().unwrap().clone();
        if let Some(extension) = extension {
            leaf.push('.');
            leaf.push_str(&extension);
        }
        let mut posix_segments = segments.clone();
        *posix_segments.last_mut().unwrap() = leaf.clone();
        let posix_path = format!("{}{}", if absolute { "/" } else { "" }, posix_segments.join("/"));
        prop_assert_eq!(posix::format(&posix::parse(&posix_path)), posix_path);

        let mut win_segments = segments;
        *win_segments.last_mut().unwrap() = leaf;
        let win_path = format!("{}{}", if absolute { "C:\\" } else { "" }, win_segments.join("\\"));
        prop_assert_eq!(win32::format(&win32::parse(&win_path)), win_path);
    }

    #[test]
    fn explicit_context_calls_replay_identically(
        segment in "[a-zA-Z0-9_]{1,16}",
        parent in any::<bool>(),
    ) {
        let context = PathContext::new(NodeHost::OtherPosix, "/fixed/cwd", vec![]).unwrap();
        let input = if parent { format!("../{segment}") } else { segment };
        let first = posix::resolve_with_context(&context, &[&input]);
        let second = posix::resolve_with_context(&context, &[&input]);
        prop_assert_eq!(first, second);
    }

    #[test]
    fn glob_seed_is_replayable(
        directory in "[a-z]{1,8}",
        stem in "[a-z]{1,8}",
        extension in prop_oneof![Just("js"), Just("ts")],
    ) {
        let context = PathContext::new(NodeHost::OtherPosix, "/", vec![]).unwrap();
        let path = format!("{directory}/{stem}.{extension}");
        let pattern = format!("**/*.{extension}");
        prop_assert!(posix::matches_glob_with_context(&context, &path, &pattern).unwrap());
    }
}

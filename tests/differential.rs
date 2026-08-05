//! Opt-in randomized differential checks against a separately pinned Node binary.

use std::io::Write;
use std::process::{Command, Stdio};

use node_path::{NodeHost, PathContext, posix, win32};
use serde_json::{Value, json};

fn oracle_binary() -> String {
    std::env::var("NODE_PATH_ORACLE_BIN")
        .expect("set NODE_PATH_ORACLE_BIN to a Node 27 binary built from the pinned commit")
}

fn call_oracle(requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(oracle_binary())
        .arg(format!(
            "{}/tests/oracle/path-oracle.mjs",
            env!("CARGO_MANIFEST_DIR")
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("pinned Node oracle starts");
    {
        let stdin = child.stdin.as_mut().unwrap();
        for request in requests {
            writeln!(stdin, "{request}").unwrap();
        }
    }
    let output = child.wait_with_output().expect("oracle exits");
    assert!(output.status.success(), "oracle failed: {output:?}");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn generated_paths() -> Vec<String> {
    const PARTS: &[&str] = &["a", "b", ".", "..", "文件", "x.y", "", "CON:"];
    let mut state = 0x4e4f_4445_5041_5448_u64;
    (0..64)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let count = 1 + (state as usize % 6);
            let separator = if state & 1 == 0 { '/' } else { '\\' };
            (0..count)
                .map(|index| PARTS[(state.rotate_left(index as u32) as usize) % PARTS.len()])
                .collect::<Vec<_>>()
                .join(&separator.to_string())
        })
        .collect()
}

#[test]
#[ignore = "requires NODE_PATH_ORACLE_BIN built from Node commit 3f42cfac"]
fn pinned_node_randomized_normalize_parse_resolve_and_glob() {
    let binary = oracle_binary();
    let version = Command::new(&binary).arg("--version").output().unwrap();
    let version = String::from_utf8(version.stdout).unwrap();
    assert!(
        version.trim_start().starts_with("v27."),
        "unexpected oracle {version}"
    );

    let cwd = std::env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    let context = PathContext::new(NodeHost::current(), cwd, vec![]).unwrap();
    let paths = generated_paths();
    let mut requests = Vec::new();
    let mut expected = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        for namespace in ["posix", "win32"] {
            requests.push(json!({
                "id": format!("normalize-{namespace}-{index}"),
                "namespace": namespace,
                "operation": "normalize",
                "arguments": [path],
            }));
            expected.push(Value::String(if namespace == "posix" {
                posix::normalize(path).into_owned()
            } else {
                win32::normalize(path).into_owned()
            }));
            requests.push(json!({
                "id": format!("resolve-{namespace}-{index}"),
                "namespace": namespace,
                "operation": "resolve",
                "arguments": [path],
            }));
            expected.push(Value::String(if namespace == "posix" {
                posix::resolve_with_context(&context, &[path])
            } else {
                win32::resolve_with_context(&context, &[path])
            }));
            requests.push(json!({
                "id": format!("parse-{namespace}-{index}"),
                "namespace": namespace,
                "operation": "parse",
                "arguments": [path],
            }));
            let parsed = if namespace == "posix" {
                posix::parse(path)
            } else {
                win32::parse(path)
            };
            expected.push(json!({
                "root": parsed.root, "dir": parsed.dir, "base": parsed.base,
                "ext": parsed.ext, "name": parsed.name,
            }));
        }
    }
    for (index, (path, pattern)) in [
        ("a/b/c.txt", "**/*.txt"),
        ("src/lib.rs", "src/**"),
        ("文件/a", "文件/*"),
    ]
    .into_iter()
    .enumerate()
    {
        requests.push(json!({
            "id": format!("glob-{index}"), "namespace": "posix",
            "operation": "matches-glob", "arguments": [path, pattern],
        }));
        expected.push(Value::Bool(
            posix::matches_glob_with_context(&context, path, pattern).unwrap(),
        ));
    }

    let actual = call_oracle(&requests);
    assert_eq!(actual.len(), expected.len());
    for ((request, response), expected) in requests.iter().zip(actual).zip(expected) {
        assert_eq!(
            response["ok"], true,
            "oracle error for {request}: {response}"
        );
        assert_eq!(
            response["result"], expected,
            "differential mismatch for {request}"
        );
    }
}

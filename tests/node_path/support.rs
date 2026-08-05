#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use node_path::{NodeHost, PathContext, PathObject, posix, win32};

pub const BASELINE_COMMIT: &str = "3f42cfacf27e348297a52d89b4cdc48b35cb7559";
pub const SOURCE_FILE_COUNT: usize = 17;
pub const ASSERTION_SITE_COUNT: usize = 301;
pub const EXPANDED_CASE_COUNT: usize = 1_406;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityBaseline {
    pub repository: String,
    pub commit: String,
    pub node_version: String,
    pub implementation_path: String,
    pub implementation_blob: String,
    pub test_glob: String,
    pub minimatch_version: String,
    pub inventoried_at: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InventoryCounts {
    pub source_files: usize,
    pub assertion_sites: usize,
    pub expanded_cases: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub blob_oid: String,
    pub assertion_sites: usize,
    pub expanded_cases: usize,
    pub counterpart: String,
    pub license_notice: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BaselineManifest {
    pub schema_version: u32,
    pub baseline: CompatibilityBaseline,
    pub minimatch_bundle_blob: String,
    pub minimatch_package_blob: String,
    pub node_license_blob: String,
    pub declared_counts: InventoryCounts,
    pub source_files: Vec<SourceFile>,
    pub license_notices: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceNodeHost {
    Win32,
    Darwin,
    OtherPosix,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceDriveCwd {
    pub device: String,
    pub cwd: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceContext {
    pub host: EvidenceNodeHost,
    pub cwd: String,
    pub drive_cwds: Vec<EvidenceDriveCwd>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceAnchor {
    pub file: String,
    pub assertion_line: usize,
    pub vector_line: Option<usize>,
    pub vector_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Namespace {
    Posix,
    Win32,
    HostDefault,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Resolve,
    Normalize,
    IsAbsolute,
    Join,
    Relative,
    ToNamespacedPath,
    Dirname,
    Basename,
    Extname,
    Format,
    Parse,
    MatchesGlob,
    MakeLong,
    Sep,
    Delimiter,
    TypeBoundary,
    ModuleIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Comparator {
    Exact,
    CaseFolded,
    Regexp,
    Throws,
    Identity,
    FieldType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    Ported,
    HarnessAdapted,
    NonRepresentable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verification {
    Pending,
    Passing,
    Failing,
    Proposed,
    Approved,
    Stale,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Approval {
    pub approved_by: String,
    pub approved_at: String,
    pub rationale: String,
    pub replacement_evidence: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedgerCase {
    pub id: String,
    pub source: SourceAnchor,
    pub namespace: Namespace,
    pub operation: Operation,
    pub host_branch: Option<EvidenceNodeHost>,
    pub arguments: Value,
    pub context: Option<EvidenceContext>,
    pub expected: Value,
    pub comparator: Comparator,
    pub content_hash: String,
    pub disposition: Disposition,
    pub verification: Verification,
    pub local_test_id: Option<String>,
    pub adaptation: Option<String>,
    pub boundary_reason: Option<String>,
    pub replacement_check: Option<String>,
    pub approval: Option<Approval>,
    pub license_notice: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParityLedger {
    pub schema_version: u32,
    pub baseline: CompatibilityBaseline,
    pub declared_counts: InventoryCounts,
    pub source_files: Vec<SourceFile>,
    pub cases: Vec<LedgerCase>,
    pub license_notices: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunMetadata {
    pub timestamp: String,
    pub rustc: String,
    pub target: String,
    pub node_host: EvidenceNodeHost,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResultCounts {
    pub ported_passing: usize,
    pub adapted_passing: usize,
    pub non_representable_approved: usize,
    pub failing: usize,
    pub pending: usize,
    pub stale: usize,
    pub proposed: usize,
    pub rejected: usize,
    pub unclassified: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Failure {
    pub case_id: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ParitySummary {
    pub schema_version: u32,
    pub baseline_commit: String,
    pub run: RunMetadata,
    pub inventory: InventoryCounts,
    pub results: ResultCounts,
    pub failures: Vec<Failure>,
    pub missing_case_ids: Vec<String>,
    pub orphan_local_test_ids: Vec<String>,
    pub unapproved_boundaries: Vec<String>,
    pub releasable: bool,
}

pub fn is_git_oid(value: &str) -> bool {
    (40..=64).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn is_content_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpstreamCallFixture {
    pub baseline_commit: String,
    pub source_file: String,
    pub cases: Vec<UpstreamCall>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpstreamCall {
    pub id: String,
    pub namespace: String,
    pub operation: String,
    pub arguments: Vec<Value>,
    pub expected: Option<Value>,
    pub error: Option<UpstreamError>,
    pub source_line: usize,
    pub source_column: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpstreamError {
    pub name: String,
    pub code: Option<String>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FixtureRun {
    pub total: usize,
    pub passing: usize,
    pub boundaries: usize,
}

fn context(source_file: &str) -> PathContext {
    if source_file.ends_with("test-path-makelong.js") {
        PathContext::new(
            NodeHost::Win32,
            "C:\\node-source",
            vec![node_path::DriveCwd {
                device: "C:".into(),
                cwd: "C:\\node-source".into(),
            }],
        )
        .expect("fixed Windows context is valid")
    } else if source_file.ends_with("test-path-posix-relative-on-windows.js") {
        PathContext::new(NodeHost::OtherPosix, "/node-source/repository", vec![])
            .expect("fixed deep POSIX context is valid")
    } else {
        PathContext::new(NodeHost::OtherPosix, "/node-source", vec![])
            .expect("fixed POSIX host context is valid")
    }
}

fn string_arguments(arguments: &[Value]) -> Option<Vec<&str>> {
    arguments.iter().map(Value::as_str).collect()
}

fn path_object(value: &Value) -> Option<PathObject<String>> {
    let object = value.as_object()?;
    if object.contains_key("$type") {
        return None;
    }
    let field = |name: &str| {
        object
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    Some(PathObject {
        root: field("root"),
        dir: field("dir"),
        base: field("base"),
        ext: field("ext"),
        name: field("name"),
    })
}

fn parsed_value(parsed: PathObject<&str>) -> Value {
    serde_json::json!({
        "root": parsed.root,
        "dir": parsed.dir,
        "base": parsed.base,
        "ext": parsed.ext,
        "name": parsed.name,
    })
}

fn evaluate(call: &UpstreamCall, context: &PathContext) -> Option<Value> {
    let win32_namespace = call.namespace == "win32";
    if call.operation == "format" {
        let object = path_object(call.arguments.first()?)?;
        return Some(Value::String(if win32_namespace {
            win32::format(&object)
        } else {
            posix::format(&object)
        }));
    }
    let arguments = string_arguments(&call.arguments)?;
    let first = || arguments.first().copied();
    let second = || arguments.get(1).copied();
    let value = match call.operation.as_str() {
        "sep" => Value::String(if win32_namespace { "\\" } else { "/" }.into()),
        "delimiter" => Value::String(if win32_namespace { ";" } else { ":" }.into()),
        "normalize" => Value::String(if win32_namespace {
            win32::normalize(first()?).into_owned()
        } else {
            posix::normalize(first()?).into_owned()
        }),
        "isAbsolute" => Value::Bool(if win32_namespace {
            win32::is_absolute(first()?)
        } else {
            posix::is_absolute(first()?)
        }),
        "join" => Value::String(if win32_namespace {
            win32::join(&arguments)
        } else {
            posix::join(&arguments)
        }),
        "relative" => Value::String(if win32_namespace {
            win32::relative_with_context(context, first()?, second()?)
        } else {
            posix::relative_with_context(context, first()?, second()?)
        }),
        "resolve" => Value::String(if win32_namespace {
            win32::resolve_with_context(context, &arguments)
        } else {
            posix::resolve_with_context(context, &arguments)
        }),
        "toNamespacedPath" => Value::String(if win32_namespace {
            win32::to_namespaced_path_with_context(context, first()?).into_owned()
        } else {
            posix::to_namespaced_path_with_context(context, first()?).into_owned()
        }),
        "dirname" => Value::String(if win32_namespace {
            win32::dirname(first()?).into()
        } else {
            posix::dirname(first()?).into()
        }),
        "basename" => Value::String(if win32_namespace {
            win32::basename(first()?, second()).into()
        } else {
            posix::basename(first()?, second()).into()
        }),
        "extname" => Value::String(if win32_namespace {
            win32::extname(first()?).into()
        } else {
            posix::extname(first()?).into()
        }),
        "parse" => {
            if win32_namespace {
                parsed_value(win32::parse(first()?))
            } else {
                parsed_value(posix::parse(first()?))
            }
        }
        "matchesGlob" => Value::Bool(if win32_namespace {
            win32::matches_glob_with_context(context, first()?, second()?).ok()?
        } else {
            posix::matches_glob_with_context(context, first()?, second()?).ok()?
        }),
        _ => return None,
    };
    Some(value)
}

pub fn run_fixture(contents: &str) -> Result<FixtureRun, Vec<String>> {
    let fixture: UpstreamCallFixture = serde_json::from_str(contents)
        .map_err(|error| vec![format!("invalid upstream fixture: {error}")])?;
    if fixture.baseline_commit != BASELINE_COMMIT {
        return Err(vec![format!(
            "{} uses baseline {}, expected {BASELINE_COMMIT}",
            fixture.source_file, fixture.baseline_commit
        )]);
    }
    let context = context(&fixture.source_file);
    let mut run = FixtureRun {
        total: fixture.cases.len(),
        ..FixtureRun::default()
    };
    let mut failures = Vec::new();
    for call in &fixture.cases {
        let empty_cwd;
        let case_context =
            if fixture.source_file.ends_with("test-path-resolve.js") && call.source_line == 89 {
                empty_cwd = PathContext::new(NodeHost::OtherPosix, "", vec![])
                    .expect("empty cwd is a supported deterministic test context");
                &empty_cwd
            } else {
                &context
            };
        let Some(actual) = evaluate(call, case_context) else {
            run.boundaries += 1;
            continue;
        };
        let Some(expected) = &call.expected else {
            failures.push(format!(
                "{}:{}:{} {} unexpectedly returned {actual:?}; upstream error {:?}",
                fixture.source_file, call.source_line, call.source_column, call.id, call.error
            ));
            continue;
        };
        if actual == *expected {
            run.passing += 1;
        } else {
            failures.push(format!(
                "{}:{}:{} {} {}.{}({:?}) expected {expected:?}, got {actual:?}",
                fixture.source_file,
                call.source_line,
                call.source_column,
                call.id,
                call.namespace,
                call.operation,
                call.arguments,
            ));
        }
    }
    if failures.is_empty() {
        Ok(run)
    } else {
        Err(failures)
    }
}

pub fn assert_fixture(contents: &str) {
    if let Err(failures) = run_fixture(contents) {
        panic!("{}", failures.join("\n"));
    }
}

use std::{
    collections::HashSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};

use serde::Deserialize;

const DEFAULT_FIXTURE: &str = "eval/semantic-v1.jsonl";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Device {
    Auto,
    Cpu,
}

impl Device {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Cpu => "cpu",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "cpu" => Ok(Self::Cpu),
            _ => Err(format!(
                "unsupported --device value {value:?}; expected auto or cpu"
            )),
        }
    }
}

#[derive(Debug, Clone)]
struct EvalOptions {
    binary: PathBuf,
    fixture: PathBuf,
    model: Option<PathBuf>,
    threshold: f64,
    device: Device,
}

impl EvalOptions {
    fn default_binary() -> PathBuf {
        PathBuf::from("target")
            .join("release")
            .join(format!("jgrep{}", env::consts::EXE_SUFFIX))
    }

    fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let mut binary = env::var_os("JGREP_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(Self::default_binary);
        let mut fixture = PathBuf::from(DEFAULT_FIXTURE);
        let mut model = None;
        let mut threshold = 0.5;
        let mut device = Device::Cpu;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bin" => binary = PathBuf::from(required_value(&mut args, "--bin")?),
                "--fixture" => fixture = PathBuf::from(required_value(&mut args, "--fixture")?),
                "--model" => model = Some(PathBuf::from(required_value(&mut args, "--model")?)),
                "--threshold" => {
                    let value = required_value(&mut args, "--threshold")?;
                    threshold = value.parse::<f64>().map_err(|_| {
                        format!(
                            "invalid --threshold value {value:?}; expected a number from 0 to 1"
                        )
                    })?;
                    if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
                        return Err(format!(
                            "invalid --threshold value {value:?}; expected a finite number from 0 to 1"
                        ));
                    }
                }
                "--device" => device = Device::parse(&required_value(&mut args, "--device")?)?,
                _ => return Err(format!("unknown eval option {arg:?}\n\n{}", eval_usage())),
            }
        }

        Ok(Self {
            binary,
            fixture,
            model,
            threshold,
            device,
        })
    }
}

fn required_value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("{option} requires a value"))
}

fn eval_usage() -> String {
    format!(
        "Usage: cargo xtask eval [OPTIONS]\n\n\
         Evaluate {DEFAULT_FIXTURE} with an already-built semantic jgrep binary.\n\n\
         Options:\n\
           --bin <PATH>        jgrep executable (default: $JGREP_BIN or target/release/jgrep)\n\
           --fixture <PATH>    JSONL fixture (default: {DEFAULT_FIXTURE})\n\
           --model <PATH>      Explicit local GGUF model path\n\
           --threshold <0..1>  Semantic score cutoff (default: 0.5)\n\
           --device <auto|cpu> Inference device (default: cpu)\n\
           -h, --help          Show this help\n\n\
         Build with `cargo build --release` and fetch the model before running."
    )
}

#[derive(Debug, Deserialize)]
struct EvaluationCase {
    id: String,
    query: String,
    text: String,
    expected_match: bool,
}

fn read_fixture(path: &Path) -> Result<Vec<EvaluationCase>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "could not read evaluation fixture {}: {error}",
            path.display()
        )
    })?;
    parse_fixture(&contents, path)
}

fn parse_fixture(contents: &str, path: &Path) -> Result<Vec<EvaluationCase>, String> {
    let mut cases = Vec::new();
    let mut ids = HashSet::new();

    for (index, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let line_number = index + 1;
        let case = serde_json::from_str::<EvaluationCase>(line).map_err(|error| {
            format!(
                "invalid JSON object in evaluation fixture {} at line {line_number}: {error}",
                path.display()
            )
        })?;

        if case.id.trim().is_empty() {
            return Err(format!(
                "evaluation fixture {} has an empty id at line {line_number}",
                path.display()
            ));
        }
        if case.query.trim().is_empty() {
            return Err(format!(
                "evaluation fixture {} has an empty query for id {:?}",
                path.display(),
                case.id
            ));
        }
        if case.text.is_empty() || case.text.contains(['\r', '\n']) {
            return Err(format!(
                "evaluation fixture {} has invalid single-line text for id {:?}",
                path.display(),
                case.id
            ));
        }
        if !ids.insert(case.id.clone()) {
            return Err(format!(
                "evaluation fixture {} contains duplicate id {:?}",
                path.display(),
                case.id
            ));
        }

        cases.push(case);
    }

    if cases.is_empty() {
        return Err(format!(
            "evaluation fixture {} contains no cases",
            path.display()
        ));
    }

    Ok(cases)
}

#[derive(Debug)]
enum CaseResult {
    Match(bool),
    Error(String),
}

fn evaluate_case(case: &EvaluationCase, options: &EvalOptions) -> CaseResult {
    let mut command = Command::new(&options.binary);
    command
        .args(["--offline", "--device", options.device.as_str()])
        .arg("--threshold")
        .arg(options.threshold.to_string());
    if let Some(model) = &options.model {
        command.arg("--model").arg(model);
    }
    command
        .args(["-q", "--"])
        .arg(&case.query)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return CaseResult::Error(format!(
                "could not start {}: {error}",
                options.binary.display()
            ));
        }
    };

    let write_result = child
        .stdin
        .as_mut()
        .ok_or_else(|| "jgrep stdin was not available".to_owned())
        .and_then(|stdin| {
            stdin
                .write_all(case.text.as_bytes())
                .and_then(|()| stdin.write_all(b"\n"))
                .map_err(|error| format!("could not write case input: {error}"))
        });
    drop(child.stdin.take());

    if let Err(error) = write_result {
        let _ = child.kill();
        let _ = child.wait();
        return CaseResult::Error(error);
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => return CaseResult::Error(format!("could not wait for jgrep: {error}")),
    };

    match output.status.code() {
        Some(0) => CaseResult::Match(true),
        Some(1) => CaseResult::Match(false),
        Some(status) => CaseResult::Error(format!(
            "jgrep exited with status {status}{}",
            diagnostic_suffix(&output.stderr)
        )),
        None => CaseResult::Error(format!(
            "jgrep ended without an exit status{}",
            diagnostic_suffix(&output.stderr)
        )),
    }
}

fn diagnostic_suffix(stderr: &[u8]) -> String {
    const LIMIT: usize = 240;
    let rendered = String::from_utf8_lossy(stderr);
    let excerpt = rendered
        .chars()
        .flat_map(char::escape_default)
        .take(LIMIT)
        .collect::<String>();
    if excerpt.is_empty() {
        String::new()
    } else {
        format!(" ({excerpt})")
    }
}

#[derive(Debug, Default)]
struct EvaluationSummary {
    total: usize,
    true_positive: usize,
    false_positive: usize,
    true_negative: usize,
    false_negative: usize,
    errors: usize,
    failed_ids: Vec<String>,
}

impl EvaluationSummary {
    fn record(&mut self, case: &EvaluationCase, result: CaseResult) {
        self.total += 1;
        match result {
            CaseResult::Match(actual) if actual && case.expected_match => self.true_positive += 1,
            CaseResult::Match(actual) if actual && !case.expected_match => {
                self.false_positive += 1;
                self.failed_ids
                    .push(format!("{} (false positive)", case.id));
            }
            CaseResult::Match(actual) if !actual && case.expected_match => {
                self.false_negative += 1;
                self.failed_ids
                    .push(format!("{} (false negative)", case.id));
            }
            CaseResult::Match(_) => self.true_negative += 1,
            CaseResult::Error(reason) => {
                self.errors += 1;
                self.failed_ids
                    .push(format!("{} (error: {reason})", case.id));
            }
        }
    }

    fn precision(&self) -> Option<f64> {
        let denominator = self.true_positive + self.false_positive;
        (denominator != 0).then(|| self.true_positive as f64 / denominator as f64)
    }

    fn recall(&self) -> Option<f64> {
        let denominator = self.true_positive + self.false_negative;
        (denominator != 0).then(|| self.true_positive as f64 / denominator as f64)
    }

    fn f1(&self) -> Option<f64> {
        let precision = self.precision()?;
        let recall = self.recall()?;
        let denominator = precision + recall;
        (denominator != 0.0).then(|| 2.0 * precision * recall / denominator)
    }

    fn has_failures(&self) -> bool {
        !self.failed_ids.is_empty()
    }
}

fn format_metric(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".to_owned(), |value| format!("{value:.4}"))
}

fn print_summary(summary: &EvaluationSummary, elapsed: Duration, options: &EvalOptions) {
    let seconds = elapsed.as_secs_f64();
    let throughput = if seconds == 0.0 {
        None
    } else {
        Some(summary.total as f64 / seconds)
    };

    println!("Semantic evaluation");
    println!("  binary: {}", options.binary.display());
    println!("  fixture: {}", options.fixture.display());
    println!(
        "  model: {}",
        options.model.as_ref().map_or_else(
            || "standard jgrep cache".to_owned(),
            |path| path.display().to_string()
        )
    );
    println!("  threshold: {:.6}", options.threshold);
    println!("  device: {}", options.device.as_str());
    println!("  cases: {}", summary.total);
    println!("  TP: {}", summary.true_positive);
    println!("  FP: {}", summary.false_positive);
    println!("  TN: {}", summary.true_negative);
    println!("  FN: {}", summary.false_negative);
    println!("  errors: {}", summary.errors);
    println!("  precision: {}", format_metric(summary.precision()));
    println!("  recall: {}", format_metric(summary.recall()));
    println!("  F1: {}", format_metric(summary.f1()));
    println!("  wall time: {:.3} s", seconds);
    println!(
        "  throughput: {} cases/s",
        throughput.map_or_else(|| "n/a".to_owned(), |value| format!("{value:.3}"))
    );
    if summary.failed_ids.is_empty() {
        println!("  failed IDs: none");
    } else {
        println!("  failed IDs:");
        for id in &summary.failed_ids {
            println!("    - {id}");
        }
    }
}

fn evaluate(options: EvalOptions) -> Result<(), String> {
    if !options.binary.is_file() {
        return Err(format!(
            "jgrep executable is missing: {}. Run `cargo build --release` or set JGREP_BIN.",
            options.binary.display()
        ));
    }

    let cases = read_fixture(&options.fixture)?;
    let started = Instant::now();
    let mut summary = EvaluationSummary::default();
    for case in &cases {
        summary.record(case, evaluate_case(case, &options));
    }
    print_summary(&summary, started.elapsed(), &options);
    std::io::stdout()
        .flush()
        .map_err(|error| format!("could not flush evaluation summary: {error}"))?;

    if summary.has_failures() {
        Err(format!(
            "semantic evaluation found {} failed case(s)",
            summary.failed_ids.len()
        ))
    } else {
        Ok(())
    }
}

fn eval_command(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let args = args.into_iter().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", eval_usage());
        return Ok(());
    }
    EvalOptions::from_args(args).and_then(evaluate)
}

fn run(args: &[&str]) -> Result<(), String> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .map_err(|error| format!("could not run cargo {}: {error}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed with {status}", args.join(" ")))
    }
}

const READMES: &[&str] = &[
    "README.md",
    "README.ja.md",
    "README.zh-CN.md",
    "README.ko.md",
    "README.es.md",
    "README.de.md",
    "README.ru.md",
    "README.fr.md",
    "README.pt-BR.md",
    "README.it.md",
    "README.ar.md",
    "README.hi.md",
];

const REQUIRED_DOCUMENTS: &[&str] = &[
    "CHANGELOG.md",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "LICENSE",
    "LICENSES/Apache-2.0.txt",
    "LICENSES/MIT.txt",
    "NOTICE",
    "RELEASING.md",
    "SECURITY.md",
    "THIRD_PARTY_NOTICES.md",
    "docs/README.md",
    "docs/evaluation.md",
    "docs/model.md",
    "docs/portability.md",
    "docs/semantic-search.md",
    "docs/translations.md",
    "eval/README.md",
    "eval/RESULTS-v0.1.0.md",
    "eval/semantic-v1.jsonl",
    "eval/semantic-v1.schema.json",
];

const MODEL_REVISION: &str = "6dd44a1fb35d11b5d1b28902876ce3cc9e882d0e";
const MODEL_SHA256: &str = "ca59ca7f13d0e15a8cfa77bd17e65d24f6844b554a7b6c12e07a5f89ff76844e";

fn check_docs() -> Result<(), String> {
    for readme in READMES {
        if !Path::new(readme).is_file() {
            return Err(format!("required documentation file is missing: {readme}"));
        }
        let contents = read_text(readme)?;
        for required in [
            "Bash",
            "PowerShell",
            "--download-model",
            "--offline",
            "GPL-3.0-or-later",
        ] {
            require_contains(readme, &contents, required)?;
        }
    }

    for document in REQUIRED_DOCUMENTS {
        if !Path::new(document).is_file() {
            return Err(format!(
                "required documentation file is missing: {document}"
            ));
        }
    }

    for document in [
        "README.md",
        "docs/model.md",
        "THIRD_PARTY_NOTICES.md",
        "eval/RESULTS-v0.1.0.md",
    ] {
        let contents = read_text(document)?;
        require_contains(document, &contents, MODEL_REVISION)?;
        require_contains(document, &contents, MODEL_SHA256)?;
    }

    let cargo_toml = read_text("Cargo.toml")?;
    require_contains("Cargo.toml", &cargo_toml, "version = \"0.1.0\"")?;
    let changelog = read_text("CHANGELOG.md")?;
    require_contains("CHANGELOG.md", &changelog, "## [0.1.0]")?;
    let results = read_text("eval/RESULTS-v0.1.0.md")?;
    require_contains("eval/RESULTS-v0.1.0.md", &results, "localjev-grep v0.1.0")?;

    let mit = read_text("LICENSES/MIT.txt")?;
    if mit.contains("<year>") || mit.contains("<copyright holders>") {
        return Err("LICENSES/MIT.txt still contains a license placeholder".to_owned());
    }
    require_contains(
        "LICENSES/MIT.txt",
        &mit,
        "Copyright (c) 2023 Georgi Gerganov",
    )?;

    for document in markdown_documents()? {
        let contents = fs::read_to_string(&document)
            .map_err(|error| format!("could not read {}: {error}", document.display()))?;
        validate_markdown_links(&document, &contents)?;
    }

    Ok(())
}

fn read_text(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("could not read {path}: {error}"))
}

fn require_contains(path: &str, contents: &str, required: &str) -> Result<(), String> {
    if contents.contains(required) {
        Ok(())
    } else {
        Err(format!("{path} is missing required text {required:?}"))
    }
}

fn markdown_documents() -> Result<Vec<PathBuf>, String> {
    let mut documents = Vec::new();
    for entry in
        fs::read_dir(".").map_err(|error| format!("could not list repository root: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("could not inspect repository entry: {error}"))?;
        let path = entry.path();
        if is_markdown_file(&path) {
            documents.push(path);
        }
    }
    for directory in [Path::new("docs"), Path::new("eval")] {
        collect_markdown_documents(directory, &mut documents)?;
    }
    documents.sort();
    Ok(documents)
}

fn collect_markdown_documents(
    directory: &Path,
    documents: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("could not list {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "could not inspect an entry in {}: {error}",
                directory.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_documents(&path, documents)?;
        } else if is_markdown_file(&path) {
            documents.push(path);
        }
    }
    Ok(())
}

fn is_markdown_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn validate_markdown_links(path: &Path, contents: &str) -> Result<(), String> {
    for target in markdown_link_targets(contents) {
        if is_external_or_anchor_link(&target) {
            continue;
        }
        let target = target
            .split_once('#')
            .map_or(target.as_str(), |(file, _)| file);
        if target.is_empty() {
            continue;
        }
        let candidate = path.parent().unwrap_or_else(|| Path::new(".")).join(target);
        if !candidate.exists() {
            return Err(format!(
                "{} has a local Markdown link to missing target {}",
                path.display(),
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn markdown_link_targets(contents: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut remaining = contents;
    while let Some(start) = remaining.find("](") {
        let after_open = &remaining[start + 2..];
        let Some(end) = after_open.find(')') else {
            break;
        };
        let raw = after_open[..end].trim();
        let raw = raw.trim_matches(['<', '>']);
        let target = raw.split_ascii_whitespace().next().unwrap_or_default();
        if !target.is_empty() {
            targets.push(target.to_owned());
        }
        remaining = &after_open[end + 1..];
    }
    targets
}

fn is_external_or_anchor_link(target: &str) -> bool {
    target.starts_with('#')
        || target.contains("://")
        || target.starts_with("mailto:")
        || target.starts_with("data:")
}

fn ci() -> Result<(), String> {
    run(&["fmt", "--all", "--check"])?;
    run(&[
        "clippy",
        "--locked",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    run(&["test", "--locked", "--workspace"])?;
    check_docs()
}

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next();
    let command = args.next().unwrap_or_else(|| "help".to_owned());
    let result = match command.as_str() {
        "ci" => ci(),
        "check-docs" => check_docs(),
        "eval" => eval_command(args),
        "help" | "--help" | "-h" => {
            println!("Usage: cargo xtask <ci|check-docs|eval>");
            Ok(())
        }
        _ => Err(format!("unknown xtask command: {command}")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_options_accept_explicit_values() {
        let options = EvalOptions::from_args([
            "--bin".to_owned(),
            "bin/jgrep".to_owned(),
            "--fixture".to_owned(),
            "fixture.jsonl".to_owned(),
            "--model".to_owned(),
            "model.gguf".to_owned(),
            "--threshold".to_owned(),
            "0.25".to_owned(),
            "--device".to_owned(),
            "auto".to_owned(),
        ])
        .expect("options should parse");

        assert_eq!(options.binary, PathBuf::from("bin/jgrep"));
        assert_eq!(options.fixture, PathBuf::from("fixture.jsonl"));
        assert_eq!(options.model, Some(PathBuf::from("model.gguf")));
        assert_eq!(options.threshold, 0.25);
        assert_eq!(options.device, Device::Auto);
    }

    #[test]
    fn eval_options_reject_invalid_threshold() {
        let error = EvalOptions::from_args(["--threshold".to_owned(), "1.5".to_owned()])
            .expect_err("threshold above one must fail");

        assert!(error.contains("invalid --threshold"));
    }

    #[test]
    fn fixture_parser_rejects_duplicate_ids() {
        let fixture = concat!(
            r#"{"id":"one","query":"query","text":"text","expected_match":true}"#,
            "\n",
            r#"{"id":"one","query":"query","text":"text","expected_match":false}"#,
            "\n"
        );
        let error = parse_fixture(fixture, Path::new("fixture.jsonl"))
            .expect_err("duplicate case identifiers must fail");

        assert!(error.contains("duplicate id"));
    }

    #[test]
    fn metrics_cover_all_confusion_matrix_outcomes() {
        let mut summary = EvaluationSummary::default();
        let positive = EvaluationCase {
            id: "positive".to_owned(),
            query: "query".to_owned(),
            text: "text".to_owned(),
            expected_match: true,
        };
        let negative = EvaluationCase {
            id: "negative".to_owned(),
            query: "query".to_owned(),
            text: "text".to_owned(),
            expected_match: false,
        };

        summary.record(&positive, CaseResult::Match(true));
        summary.record(&positive, CaseResult::Match(false));
        summary.record(&negative, CaseResult::Match(true));
        summary.record(&negative, CaseResult::Match(false));
        summary.record(&positive, CaseResult::Error("failed".to_owned()));

        assert_eq!(summary.total, 5);
        assert_eq!(summary.true_positive, 1);
        assert_eq!(summary.false_negative, 1);
        assert_eq!(summary.false_positive, 1);
        assert_eq!(summary.true_negative, 1);
        assert_eq!(summary.errors, 1);
        assert_eq!(summary.precision(), Some(0.5));
        assert_eq!(summary.recall(), Some(0.5));
        assert_eq!(summary.f1(), Some(0.5));
        assert_eq!(summary.failed_ids.len(), 3);
    }
}

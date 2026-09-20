//! Streaming search, traversal, and output rendering.
//!
//! This module deliberately does not know how a model is loaded.  Semantic
//! scoring is represented by [`SemanticScorer`], which keeps all lexical
//! functionality usable in a minimal build and makes the engine easy to test.

use std::{
    collections::VecDeque,
    ffi::OsStr,
    fmt::Write as _,
    fs::{self, File},
    io::{self, BufRead, BufReader, IsTerminal, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

/// Search implementation selected by the command line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchMode {
    Semantic,
    Regex,
    Fixed,
}

/// Device preference passed to the model runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Device {
    Auto,
    Cpu,
}

/// Options that control model acquisition and semantic scoring.
#[derive(Clone, Debug)]
pub struct SemanticOptions {
    pub model_path: Option<PathBuf>,
    pub offline: bool,
    pub device: Device,
    pub threshold: f32,
    pub report_score: bool,
}

/// A model implementation that returns an uncalibrated semantic relevance
/// score in the inclusive range 0.0 through 1.0.
pub trait SemanticScorer {
    /// Score a single candidate line against a search context.
    ///
    /// # Errors
    ///
    /// Returns an error when the local runtime cannot score the candidate.
    fn score(&mut self, query: &str, line: &str) -> Result<f32>;
}

/// ANSI color behavior for regular line output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

/// The output style selected by grep-compatible flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Lines,
    /// Compact path-and-line locations intended as a first-pass result for a
    /// coding agent. Records contain no source text, so a caller can fetch
    /// only the small ranges it needs next.
    AiRecords,
    Count,
    FilesWithMatches,
    FilesWithoutMatches,
    Quiet,
}

/// Default total location budget for `--ai`. This applies to the full
/// invocation, including recursive and multi-file searches, rather than to
/// each input as grep's `-m` does.
pub const DEFAULT_AI_MAX_RESULTS: usize = 50;

/// A normalized command that the streaming engine can execute.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct SearchConfig {
    pub mode: SearchMode,
    pub patterns: Vec<String>,
    pub paths: Vec<PathBuf>,
    pub invert_match: bool,
    pub line_number: bool,
    pub with_filename: bool,
    pub no_filename: bool,
    pub output_mode: OutputMode,
    pub max_count: Option<usize>,
    pub recursive: bool,
    pub before_context: usize,
    pub after_context: usize,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub color: ColorMode,
    pub line_buffered: bool,
    /// A global location budget for compact agent output. `None` preserves
    /// normal grep behavior without a cross-input result cap.
    pub ai_max_results: Option<usize>,
    pub ignore_case: bool,
    pub semantic: SemanticOptions,
    /// Request model acquisition before a search. A command with no patterns
    /// is a download-only invocation and is handled by the application layer.
    pub download_model: bool,
}

/// The observable status of a completed run.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RunSummary {
    pub selected_any: bool,
    pub had_error: bool,
    pub output_broken: bool,
    pub ai_limit_reached: bool,
}

impl RunSummary {
    /// grep-compatible process status.
    pub const fn exit_code(self) -> u8 {
        if self.had_error {
            2
        } else if self.selected_any {
            0
        } else {
            1
        }
    }
}

/// Execute a normalized search. The scorer factory is held until the first
/// non-empty semantic candidate is evaluated, so lexical searches, empty
/// input, and `-m 0` never initialize a model.
///
/// # Errors
///
/// Returns an error for invalid matcher configuration, malformed globs, or a
/// semantic scorer initialization failure. Per-input I/O failures are emitted
/// as diagnostics and recorded in the returned summary.
pub fn run<W, E, F>(
    config: &SearchConfig,
    stdout: &mut W,
    stderr: &mut E,
    scorer_factory: F,
) -> Result<RunSummary>
where
    W: Write + IsTerminal,
    E: Write,
    F: FnOnce(&SemanticOptions) -> Result<Box<dyn SemanticScorer>>,
{
    validate_runtime_config(config)?;
    let selector = PathSelector::new(&config.include, &config.exclude)?;
    let (sources, mut summary) = collect_sources(config, &selector, stderr);
    // Decide this from operands rather than surviving readable sources. This
    // mirrors grep when one of several requested paths fails to open.
    let show_filename = config.with_filename
        || (!config.no_filename && (config.paths.len() > 1 || config.recursive));
    let stdout_is_terminal = stdout.is_terminal();
    let color_enabled = match config.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => stdout_is_terminal,
    };
    // A pipe may close immediately after receiving a selected record. Flush
    // it even without `--line-buffered` so the next read from an open stdin
    // producer cannot leave this process waiting forever before it notices
    // EPIPE. Terminals retain their usual newline buffering unless the user
    // explicitly requests `--line-buffered`.
    let mut sink = OutputSink::new(
        stdout,
        color_enabled,
        config.line_buffered || !stdout_is_terminal,
    );

    // `-m 0` is intentionally resolved before even constructing a matcher.
    // This guarantees semantic invocations do not initialize a model and lets
    // `-L` retain its useful "all readable inputs" behavior.
    if config.max_count == Some(0) {
        for source in &sources {
            if sink.broken_pipe {
                summary.output_broken = true;
                break;
            }
            match process_zero_source(config, source, show_filename, &mut sink) {
                Ok(selected) => summary.selected_any |= selected,
                Err(error) => {
                    let non_text = error.downcast_ref::<NonTextInputError>().is_some();
                    if !(source.recursive && non_text) {
                        summary.had_error = true;
                    }
                    write_diagnostic(stderr, &source.label, &error.to_string());
                }
            }
        }
        summary.output_broken |= sink.broken_pipe;
        return Ok(summary);
    }

    let mut matcher = Matcher::new(config, scorer_factory)?;
    let mut ai_remaining = config.ai_max_results;

    for (source_index, source) in sources.iter().enumerate() {
        if sink.broken_pipe {
            summary.output_broken = true;
            break;
        }
        if ai_remaining == Some(0) {
            summary.ai_limit_reached = true;
            summary.had_error |= validate_sources_after_ai_limit(&sources[source_index..], stderr);
            break;
        }
        let result = process_source(
            config,
            source,
            show_filename,
            &mut matcher,
            &mut sink,
            &mut ai_remaining,
        );
        match result {
            Ok(source_result) => {
                summary.selected_any |= source_result.selected_any;
                if config.output_mode == OutputMode::Quiet && source_result.selected_any {
                    // grep's quiet mode is successful when it finds a match,
                    // even if an earlier operand could not be read.
                    summary.had_error = false;
                    break;
                }
                if ai_remaining == Some(0) {
                    summary.ai_limit_reached = true;
                    summary.had_error |=
                        validate_sources_after_ai_limit(&sources[source_index..], stderr);
                    break;
                }
            }
            Err(error) => {
                let non_text = error.downcast_ref::<NonTextInputError>().is_some();
                if !(source.recursive && non_text) {
                    summary.had_error = true;
                }
                write_diagnostic(stderr, &source.label, &error.to_string());
            }
        }
    }

    summary.output_broken |= sink.broken_pipe;
    Ok(summary)
}

fn validate_runtime_config(config: &SearchConfig) -> Result<()> {
    if config.patterns.is_empty() && !config.download_model {
        bail!("a search context is required");
    }
    if config.patterns.is_empty() {
        return Ok(());
    }
    if config.mode == SearchMode::Semantic && config.ignore_case {
        bail!("-i/--ignore-case is only available with -E or -F");
    }
    if config.semantic.threshold.is_nan() || !(0.0..=1.0).contains(&config.semantic.threshold) {
        bail!("--threshold must be a finite number from 0.0 through 1.0");
    }
    Ok(())
}

#[derive(Debug)]
struct InputSource {
    path: Option<PathBuf>,
    label: String,
    recursive: bool,
}

impl InputSource {
    fn ai_label(&self) -> &str {
        if self.path.is_some() {
            &self.label
        } else {
            // `-` is already the reserved command-line spelling for standard
            // input. Keeping it in compact output makes the locator distinct
            // from a real relative file named `stdin`.
            "-"
        }
    }
}

fn collect_sources<E: Write>(
    config: &SearchConfig,
    selector: &PathSelector,
    stderr: &mut E,
) -> (Vec<InputSource>, RunSummary) {
    if config.paths.is_empty() {
        return (
            vec![InputSource {
                path: None,
                label: "(standard input)".to_owned(),
                recursive: false,
            }],
            RunSummary::default(),
        );
    }

    let mut sources = Vec::new();
    let mut summary = RunSummary::default();
    for operand in &config.paths {
        if operand.as_os_str() == OsStr::new("-") {
            sources.push(InputSource {
                path: None,
                label: "(standard input)".to_owned(),
                recursive: false,
            });
            continue;
        }

        // `metadata` follows links, which is useful for ordinary explicit
        // file operands. Recursive traversal has a different contract: no
        // symlink operand is a traversal root, including a symlink supplied
        // directly on the command line.
        if config.recursive {
            match fs::symlink_metadata(operand) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    write_diagnostic(
                        stderr,
                        &display_path(operand),
                        "skipping symbolic link during recursive search",
                    );
                    continue;
                }
                Ok(_) => {}
                Err(error) => {
                    summary.had_error = true;
                    write_diagnostic(stderr, &display_path(operand), &error.to_string());
                    continue;
                }
            }
        }

        match fs::metadata(operand) {
            Ok(metadata) if metadata.is_dir() => {
                if !config.recursive {
                    summary.had_error = true;
                    write_diagnostic(
                        stderr,
                        &display_path(operand),
                        "is a directory (use -r to recurse)",
                    );
                    continue;
                }
                let (mut found, traversal_failed) = collect_directory(operand, selector, stderr);
                sources.append(&mut found);
                summary.had_error |= traversal_failed;
            }
            Ok(_) => sources.push(InputSource {
                path: Some(operand.clone()),
                label: display_path(operand),
                recursive: false,
            }),
            Err(error) => {
                summary.had_error = true;
                write_diagnostic(stderr, &display_path(operand), &error.to_string());
            }
        }
    }
    (sources, summary)
}

fn collect_directory<E: Write>(
    root: &Path,
    selector: &PathSelector,
    stderr: &mut E,
) -> (Vec<InputSource>, bool) {
    let mut paths = Vec::new();
    let mut had_error = false;
    for entry in WalkDir::new(root)
        .follow_links(false)
        .follow_root_links(false)
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                had_error = true;
                let label = error
                    .path()
                    .map_or_else(|| display_path(root), display_path);
                write_diagnostic(stderr, &label, &format!("unable to traverse: {error}"));
                continue;
            }
        };
        let file_type = entry.file_type();
        if file_type.is_symlink() || !file_type.is_file() {
            continue;
        }
        let path = entry.into_path();
        if selector.matches(&path, root) {
            paths.push(path);
        }
    }
    paths.sort_by_key(|path| normalized_path(path));
    (
        paths
            .into_iter()
            .map(|path| InputSource {
                label: display_path(&path),
                path: Some(path),
                recursive: true,
            })
            .collect(),
        had_error,
    )
}

struct PathSelector {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl PathSelector {
    fn new(include: &[String], exclude: &[String]) -> Result<Self> {
        Ok(Self {
            include: build_glob_set(include, "--include")?,
            exclude: build_glob_set(exclude, "--exclude")?,
        })
    }

    fn matches(&self, path: &Path, root: &Path) -> bool {
        let relative = path.strip_prefix(root).unwrap_or(path);
        let name = path.file_name().unwrap_or_else(|| OsStr::new(""));
        // Normalize slash spelling before the glob engine sees recursive paths;
        // users write `nested/*.log` on every platform.
        let normalized_relative = normalized_path(relative);
        let normalized_filename = normalized_path(Path::new(name));
        let included = self.include.as_ref().is_none_or(|set| {
            set.is_match(&normalized_relative) || set.is_match(&normalized_filename)
        });
        let excluded = self.exclude.as_ref().is_some_and(|set| {
            set.is_match(&normalized_relative) || set.is_match(&normalized_filename)
        });
        included && !excluded
    }
}

fn build_glob_set(patterns: &[String], option: &str) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob =
            Glob::new(pattern).with_context(|| format!("invalid {option} glob {pattern:?}"))?;
        builder.add(glob);
    }
    Ok(Some(builder.build()?))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_diagnostic<E: Write>(stderr: &mut E, label: &str, message: &str) {
    let _ = writeln!(stderr, "jgrep: {label}: {message}");
}

enum Matcher<F> {
    Semantic {
        patterns: Vec<String>,
        threshold: f32,
        scorer: Option<Box<dyn SemanticScorer>>,
        factory: Option<F>,
        options: SemanticOptions,
    },
    Regex(Vec<Regex>),
    Fixed(Vec<Regex>),
}

#[derive(Clone, Copy, Debug)]
struct MatchResult {
    selected: bool,
    score: Option<f32>,
}

impl<F> Matcher<F>
where
    F: FnOnce(&SemanticOptions) -> Result<Box<dyn SemanticScorer>>,
{
    fn new(config: &SearchConfig, factory: F) -> Result<Self> {
        match config.mode {
            SearchMode::Semantic => Ok(Self::Semantic {
                patterns: config.patterns.clone(),
                threshold: config.semantic.threshold,
                scorer: None,
                factory: Some(factory),
                options: config.semantic.clone(),
            }),
            SearchMode::Regex => Ok(Self::Regex(compile_regexes(
                &config.patterns,
                config.ignore_case,
                false,
            )?)),
            SearchMode::Fixed => Ok(Self::Fixed(compile_regexes(
                &config.patterns,
                config.ignore_case,
                true,
            )?)),
        }
    }

    fn evaluate(&mut self, line: &str, invert_match: bool) -> Result<MatchResult> {
        let (matched, score) = match self {
            Self::Regex(regexes) | Self::Fixed(regexes) => {
                (regexes.iter().any(|regex| regex.is_match(line)), None)
            }
            Self::Semantic {
                patterns,
                threshold,
                scorer,
                factory,
                options,
            } => {
                // A blank candidate cannot contain meaningful semantic content.
                // Avoiding scorer creation here also keeps a truly empty input
                // independent of model download/loading.
                if line.trim().is_empty() {
                    (false, None)
                } else {
                    if scorer.is_none() {
                        let loader = factory.take().ok_or_else(|| {
                            anyhow!("semantic scorer was unavailable after initialization")
                        })?;
                        *scorer = Some(loader(options)?);
                    }
                    let scorer = scorer.as_deref_mut().expect("scorer was initialized");
                    let mut best = f32::NEG_INFINITY;
                    for pattern in patterns {
                        let candidate = scorer.score(pattern, line)?;
                        if !candidate.is_finite() || !(0.0..=1.0).contains(&candidate) {
                            bail!("semantic scorer returned an invalid relevance score");
                        }
                        best = best.max(candidate);
                    }
                    (best >= *threshold, Some(best))
                }
            }
        };
        Ok(MatchResult {
            selected: matched ^ invert_match,
            score,
        })
    }
}

fn compile_regexes(patterns: &[String], ignore_case: bool, fixed: bool) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|pattern| {
            let source = if fixed {
                regex::escape(pattern)
            } else {
                pattern.clone()
            };
            RegexBuilder::new(&source)
                .case_insensitive(ignore_case)
                .build()
                .with_context(|| format!("invalid regex {pattern:?}"))
        })
        .collect()
}

#[derive(Debug)]
struct SourceResult {
    selected_any: bool,
}

fn process_source<W, F>(
    config: &SearchConfig,
    source: &InputSource,
    show_filename: bool,
    matcher: &mut Matcher<F>,
    sink: &mut OutputSink<'_, W>,
    ai_remaining: &mut Option<usize>,
) -> Result<SourceResult>
where
    W: Write,
    F: FnOnce(&SemanticOptions) -> Result<Box<dyn SemanticScorer>>,
{
    if let Some(path) = &source.path {
        let mut file =
            File::open(path).with_context(|| format!("unable to open {}", source.label))?;
        // A recursive path is discovered before it is opened. Preflight its
        // complete byte stream before emitting anything, so a NUL or invalid
        // UTF-8 sequence near the end cannot leave partial search output.
        // Rewind the same descriptor rather than reopening by path.
        if source.recursive {
            preflight_text_file(&mut file, source)?;
            file.seek(SeekFrom::Start(0))
                .with_context(|| format!("unable to rewind {}", source.label))?;
        }
        let reader = BufReader::new(file);
        process_reader(
            config,
            source,
            show_filename,
            matcher,
            sink,
            ai_remaining,
            reader,
        )
    } else {
        let stdin = io::stdin();
        process_reader(
            config,
            source,
            show_filename,
            matcher,
            sink,
            ai_remaining,
            stdin.lock(),
        )
    }
}

#[allow(clippy::too_many_lines)]
fn process_reader<R, W, F>(
    config: &SearchConfig,
    source: &InputSource,
    show_filename: bool,
    matcher: &mut Matcher<F>,
    sink: &mut OutputSink<'_, W>,
    ai_remaining: &mut Option<usize>,
    mut reader: R,
) -> Result<SourceResult>
where
    R: BufRead,
    W: Write,
    F: FnOnce(&SemanticOptions) -> Result<Box<dyn SemanticScorer>>,
{
    // Looking at the buffered prefix detects normal binary files before any
    // output. A NUL encountered later is still handled below.
    if reader.fill_buf()?.contains(&0) {
        return binary_input_error(source);
    }

    // `-B`/`-C` accept a grep-sized unsigned value. Do not allocate that
    // amount up front: a large requested window should grow only as input
    // lines actually arrive, rather than panic before the first read.
    let mut before = VecDeque::new();
    let mut selected_count = 0usize;
    let mut selected_any = false;
    let mut after_until = 0usize;
    let mut stop_after = None;
    let mut last_emitted = None;
    let mut emitted_group = false;
    let mut raw = Vec::new();
    let mut line_number = 0usize;

    loop {
        // `write_record` marks a closed downstream pipe rather than failing
        // the command. Check before reading again so an open stdin/FIFO
        // producer cannot keep this process blocked after EPIPE.
        if sink.broken_pipe {
            break;
        }
        raw.clear();
        let bytes = reader.read_until(b'\n', &mut raw)?;
        if bytes == 0 {
            break;
        }
        line_number += 1;
        if raw.contains(&0) {
            return binary_input_error(source);
        }
        if raw.last() == Some(&b'\n') {
            raw.pop();
        }
        if raw.last() == Some(&b'\r') {
            raw.pop();
        }
        let line = std::str::from_utf8(&raw).map_err(|_| NonTextInputError::invalid_utf8())?;

        let can_select = stop_after.is_none() && ai_remaining.is_none_or(|remaining| remaining > 0);
        let result = if can_select {
            matcher.evaluate(line, config.invert_match)?
        } else {
            MatchResult {
                selected: false,
                score: None,
            }
        };

        if result.selected {
            selected_any = true;
            selected_count += 1;
            match config.output_mode {
                OutputMode::Lines => {
                    for record in &before {
                        emit_context_record(
                            sink,
                            source,
                            show_filename,
                            config,
                            record,
                            &mut last_emitted,
                            &mut emitted_group,
                        )?;
                    }
                    let record = LineRecord {
                        number: line_number,
                        text: line.to_owned(),
                        selected: true,
                        score: result.score,
                    };
                    emit_context_record(
                        sink,
                        source,
                        show_filename,
                        config,
                        &record,
                        &mut last_emitted,
                        &mut emitted_group,
                    )?;
                    after_until = after_until.max(line_number.saturating_add(config.after_context));
                }
                OutputMode::AiRecords => {
                    sink.ai_record(source.ai_label(), line_number)?;
                    // A closed downstream consumer is a normal grep case.
                    // Do not consume the AI budget after EPIPE, or the caller
                    // would receive a misleading "limit reached" diagnostic.
                    if !sink.broken_pipe {
                        if let Some(remaining) = ai_remaining {
                            *remaining = remaining.saturating_sub(1);
                        }
                    }
                }
                OutputMode::Quiet => return Ok(SourceResult { selected_any: true }),
                OutputMode::FilesWithMatches => {
                    sink.file_name(&source.label)?;
                    return Ok(SourceResult { selected_any: true });
                }
                OutputMode::FilesWithoutMatches | OutputMode::Count => {}
            }

            if config
                .max_count
                .is_some_and(|limit| selected_count >= limit)
            {
                stop_after = Some(line_number.saturating_add(config.after_context));
                if !matches!(config.output_mode, OutputMode::Lines) || config.after_context == 0 {
                    break;
                }
            }
            if *ai_remaining == Some(0) {
                break;
            }
        } else if config.output_mode == OutputMode::Lines && line_number <= after_until {
            let record = LineRecord {
                number: line_number,
                text: line.to_owned(),
                selected: false,
                score: None,
            };
            emit_context_record(
                sink,
                source,
                show_filename,
                config,
                &record,
                &mut last_emitted,
                &mut emitted_group,
            )?;
        }

        if config.before_context > 0 {
            before.try_reserve(1).map_err(|error| {
                anyhow!(
                    "could not grow the --before-context buffer for {}: {error}",
                    source.label
                )
            })?;
            before.push_back(LineRecord {
                number: line_number,
                text: line.to_owned(),
                selected: false,
                score: None,
            });
            while before.len() > config.before_context {
                before.pop_front();
            }
        }

        if stop_after.is_some_and(|last| line_number >= last) {
            break;
        }
    }

    match config.output_mode {
        OutputMode::Count => sink.count(&source.label, show_filename, selected_count)?,
        OutputMode::FilesWithoutMatches if !selected_any => sink.file_name(&source.label)?,
        _ => {}
    }
    Ok(SourceResult {
        // The process status follows selected lines after `-v`, rather than
        // whether `-L` happened to print a file name.
        selected_any,
    })
}

fn process_zero_source<W: Write>(
    config: &SearchConfig,
    source: &InputSource,
    show_filename: bool,
    sink: &mut OutputSink<'_, W>,
) -> Result<bool> {
    // Validate named files even though zero matches are requested. That keeps
    // the explicit non-text input contract intact, while stdin remains
    // untouched and no semantic scorer is constructed.
    if let Some(path) = &source.path {
        let mut file =
            File::open(path).with_context(|| format!("unable to open {}", source.label))?;
        preflight_text_file(&mut file, source)?;
    }
    match config.output_mode {
        OutputMode::Count => sink.count(&source.label, show_filename, 0)?,
        OutputMode::FilesWithoutMatches => sink.file_name(&source.label)?,
        _ => {}
    }
    // `-m 0` cannot select a line. `-L` may print a filename, but that does
    // not change grep's selected-line status.
    Ok(false)
}

/// After compact output reaches its budget, stop matching immediately but
/// retain the explicit regular-file contract: a named binary or invalid UTF-8
/// file is still an error. This deliberately does no model work and never
/// consumes an unread stdin stream, which could be an open-ended producer.
fn validate_sources_after_ai_limit<E: Write>(sources: &[InputSource], stderr: &mut E) -> bool {
    let mut had_error = false;
    for source in sources {
        let Some(path) = &source.path else {
            continue;
        };
        // Regular files can be safely reopened and preflighted without
        // compromising a prompt result. Do not reopen named FIFOs, devices,
        // or other streams here: a writer may intentionally remain open after
        // the compact result limit is reached.
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                had_error = true;
                write_diagnostic(stderr, &source.label, &error.to_string());
                continue;
            }
        };
        if !metadata.is_file() {
            continue;
        }
        let result = (|| -> Result<()> {
            let mut file =
                File::open(path).with_context(|| format!("unable to open {}", source.label))?;
            preflight_text_file(&mut file, source)
        })();
        if let Err(error) = result {
            let non_text = error.downcast_ref::<NonTextInputError>().is_some();
            if !(source.recursive && non_text) {
                had_error = true;
            }
            write_diagnostic(stderr, &source.label, &error.to_string());
        }
    }
    had_error
}

/// Validate a file as text without retaining its contents. This is used as a
/// preflight for recursive sources and for `-m 0` named operands. Retaining
/// only an unfinished UTF-8 sequence keeps memory bounded by the read buffer
/// rather than the file size.
fn preflight_text_file(file: &mut File, source: &InputSource) -> Result<()> {
    const BUFFER_SIZE: usize = 8 * 1024;

    let mut buffer = [0_u8; BUFFER_SIZE];
    // Holds at most the unfinished suffix of a multi-byte UTF-8 sequence
    // between reads. Its allocation may grow to one buffer, then is reused.
    let mut pending = Vec::with_capacity(4);

    loop {
        let bytes = match file.read(&mut buffer) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("unable to read {}", source.label));
            }
        };
        if bytes == 0 {
            break;
        }

        let chunk = &buffer[..bytes];
        if chunk.contains(&0) {
            return binary_input_error(source);
        }
        pending.extend_from_slice(chunk);
        match std::str::from_utf8(&pending) {
            Ok(_) => pending.clear(),
            Err(error) if error.error_len().is_some() => {
                return Err(NonTextInputError::invalid_utf8().into());
            }
            Err(error) => {
                // `error_len == None` means the only non-valid bytes are an
                // incomplete sequence at the end. Keep that suffix for the
                // next buffer; it is at most three bytes for UTF-8.
                pending.drain(..error.valid_up_to());
            }
        }
    }

    if pending.is_empty() {
        Ok(())
    } else {
        Err(NonTextInputError::invalid_utf8().into())
    }
}

#[derive(Debug)]
struct NonTextInputError {
    binary: bool,
}

impl NonTextInputError {
    const fn binary() -> Self {
        Self { binary: true }
    }

    const fn invalid_utf8() -> Self {
        Self { binary: false }
    }
}

impl std::fmt::Display for NonTextInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.binary {
            formatter.write_str("binary input is not supported")
        } else {
            formatter.write_str("input is not valid UTF-8 text")
        }
    }
}

impl std::error::Error for NonTextInputError {}

fn binary_input_error<T>(_source: &InputSource) -> Result<T> {
    Err(NonTextInputError::binary().into())
}

#[derive(Clone, Debug)]
struct LineRecord {
    number: usize,
    text: String,
    selected: bool,
    score: Option<f32>,
}

#[allow(clippy::too_many_arguments)]
fn emit_context_record<W: Write>(
    sink: &mut OutputSink<'_, W>,
    source: &InputSource,
    show_filename: bool,
    config: &SearchConfig,
    record: &LineRecord,
    last_emitted: &mut Option<usize>,
    emitted_group: &mut bool,
) -> Result<()> {
    if last_emitted.is_some_and(|last| record.number <= last) {
        return Ok(());
    }
    if *emitted_group
        && (config.before_context > 0 || config.after_context > 0)
        && last_emitted.is_some_and(|last| record.number > last + 1)
    {
        sink.context_separator()?;
    }
    sink.line(
        &source.label,
        show_filename,
        config.line_number,
        record.selected,
        record.number,
        &record.text,
        if config.semantic.report_score && record.selected {
            record.score
        } else {
            None
        },
    )?;
    *last_emitted = Some(record.number);
    *emitted_group = true;
    Ok(())
}

struct OutputSink<'a, W> {
    writer: &'a mut W,
    color: bool,
    flush_after_write: bool,
    broken_pipe: bool,
}

impl<'a, W: Write> OutputSink<'a, W> {
    fn new(writer: &'a mut W, color: bool, flush_after_write: bool) -> Self {
        Self {
            writer,
            color,
            flush_after_write,
            broken_pipe: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn line(
        &mut self,
        label: &str,
        show_filename: bool,
        line_number: bool,
        selected: bool,
        number: usize,
        text: &str,
        score: Option<f32>,
    ) -> Result<()> {
        let separator = if selected { ':' } else { '-' };
        let mut record = String::new();
        if show_filename {
            record.push_str(label);
            record.push(separator);
        }
        if line_number {
            record.push_str(&number.to_string());
            record.push(separator);
        }
        if let Some(score) = score {
            write!(&mut record, "score={score:.3} ").expect("writing to a String cannot fail");
        }
        if self.color && selected {
            record.push_str("\x1b[1;31m");
            record.push_str(text);
            record.push_str("\x1b[0m");
        } else {
            record.push_str(text);
        }
        self.write_record(&record)
    }

    fn ai_record(&mut self, label: &str, number: usize) -> Result<()> {
        // Preserve ordinary paths verbatim, including Windows backslashes.
        // A line break would make the one-record-per-line protocol ambiguous,
        // so report it instead of emitting an unrecoverable location.
        if label.contains(['\n', '\r']) {
            bail!("--ai cannot emit a path containing a line break");
        }
        self.write_record(&format!("{label}:{number}"))
    }

    fn count(&mut self, label: &str, show_filename: bool, count: usize) -> Result<()> {
        let record = if show_filename {
            format!("{label}:{count}")
        } else {
            count.to_string()
        };
        self.write_record(&record)
    }

    fn file_name(&mut self, label: &str) -> Result<()> {
        self.write_record(label)
    }

    fn context_separator(&mut self) -> Result<()> {
        self.write_record("--")
    }

    fn write_record(&mut self, record: &str) -> Result<()> {
        if self.broken_pipe {
            return Ok(());
        }
        match writeln!(self.writer, "{record}") {
            Ok(()) if self.flush_after_write => match self.writer.flush() {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
                    self.broken_pipe = true;
                    Ok(())
                }
                Err(error) => Err(error.into()),
            },
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
                self.broken_pipe = true;
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    }
}

//! Command-line parsing and validation for `jgrep`.
//!
//! The parser deliberately keeps all positional values in one collection.
//! `grep` has two supported forms: a positional pattern, or one or more `-e`
//! patterns.  Splitting the operands after Clap has parsed the flags makes
//! those forms unambiguous and keeps options usable before or after a pattern.

use std::{ffi::OsString, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use clap::{ArgAction, ArgMatches, CommandFactory, FromArgMatches, Parser, ValueEnum};

use crate::engine::{ColorMode, Device, OutputMode, SearchConfig, SearchMode, SemanticOptions};

/// The raw, user-facing command line.
#[derive(Debug, Parser)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "jgrep",
    version,
    about = "Search text with local semantic matching or familiar lexical grep modes",
    override_usage = "jgrep [OPTIONS] <CONTEXT> [FILE ...]\n       jgrep [OPTIONS] -e <CONTEXT> [-e <CONTEXT> ...] [FILE ...]",
    disable_help_flag = true
)]
pub struct Cli {
    /// Show this help message.
    #[arg(long, action = ArgAction::Help)]
    _help: Option<bool>,

    /// Add a search context. Any supplied context may select a line.
    #[arg(
        short = 'e',
        long = "regexp",
        value_name = "CONTEXT",
        action = ArgAction::Append
    )]
    patterns: Vec<String>,

    /// Use Rust regular expressions instead of semantic matching.
    #[arg(
        short = 'E',
        long = "extended-regexp",
        conflicts_with = "fixed_strings"
    )]
    extended_regexp: bool,

    /// Search literal strings instead of semantic matching.
    #[arg(
        short = 'F',
        long = "fixed-strings",
        conflicts_with = "extended_regexp"
    )]
    fixed_strings: bool,

    /// Select non-matching lines.
    #[arg(short = 'v', long = "invert-match")]
    invert_match: bool,

    /// Prefix selected lines with their line number.
    #[arg(short = 'n', long = "line-number")]
    line_number: bool,

    /// Always print a filename prefix.
    #[arg(short = 'H', long = "with-filename", conflicts_with = "no_filename")]
    with_filename: bool,

    /// Never print a filename prefix. `--help` is used for help because `-h`
    /// has the same meaning as grep's `--no-filename`.
    #[arg(short = 'h', long = "no-filename", conflicts_with = "with_filename")]
    no_filename: bool,

    /// Print the number of selected lines for each input.
    #[arg(short = 'c', long = "count")]
    count: bool,

    /// Print only names of inputs containing a selected line.
    #[arg(
        short = 'l',
        long = "files-with-matches",
        conflicts_with = "files_without_match"
    )]
    files_with_matches: bool,

    /// Print only names of inputs containing no selected line.
    #[arg(
        short = 'L',
        long = "files-without-match",
        conflicts_with = "files_with_matches"
    )]
    files_without_match: bool,

    /// Stop after the first selected line and suppress normal output.
    #[arg(short = 'q', long = "quiet", conflicts_with_all = ["count", "files_with_matches", "files_without_match"])]
    quiet: bool,

    /// Stop after NUM selected lines in each input.
    #[arg(short = 'm', long = "max-count", value_name = "NUM")]
    max_count: Option<usize>,

    /// Recursively search directories without following symbolic links.
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,

    /// Print NUM lines of trailing context.
    #[arg(
        short = 'A',
        long = "after-context",
        value_name = "NUM",
        action = ArgAction::Append
    )]
    after_context: Vec<usize>,

    /// Print NUM lines of leading context.
    #[arg(
        short = 'B',
        long = "before-context",
        value_name = "NUM",
        action = ArgAction::Append
    )]
    before_context: Vec<usize>,

    /// Print NUM lines of both leading and trailing context.
    #[arg(
        short = 'C',
        long = "context",
        value_name = "NUM",
        action = ArgAction::Append
    )]
    context: Vec<usize>,

    #[arg(skip)]
    resolved_before_context: usize,

    #[arg(skip)]
    resolved_after_context: usize,

    /// Search recursively only in paths matching GLOB.
    #[arg(long = "include", value_name = "GLOB", action = ArgAction::Append)]
    include: Vec<String>,

    /// Skip recursively discovered paths matching GLOB.
    #[arg(long = "exclude", value_name = "GLOB", action = ArgAction::Append)]
    exclude: Vec<String>,

    /// When to use ANSI color in regular line output.
    #[arg(long, value_enum, default_value_t = ColorArgument::Auto)]
    color: ColorArgument,

    /// Flush stdout after every output record.
    #[arg(long = "line-buffered")]
    line_buffered: bool,

    /// Ignore case in regular-expression and fixed-string modes.
    #[arg(short = 'i', long = "ignore-case")]
    ignore_case: bool,

    /// Semantic relevance score required to select a line (0.0 through 1.0).
    #[arg(long, value_name = "SCORE")]
    threshold: Option<f32>,

    /// Include the semantic relevance score with selected lines.
    #[arg(long)]
    score: bool,

    /// Use a local GGUF model instead of the cache.
    #[arg(long, value_name = "PATH")]
    model: Option<PathBuf>,

    /// Do not download a missing semantic model.
    #[arg(long)]
    offline: bool,

    /// Download the configured semantic model, then exit when no context is supplied.
    #[arg(long = "download-model", conflicts_with = "offline")]
    download_model: bool,

    /// Semantic inference device.
    #[arg(long, value_enum, value_name = "DEVICE")]
    device: Option<DeviceArgument>,

    /// A context followed by zero or more files, or only files when `-e` is
    /// used. `-` denotes standard input.
    #[arg(value_name = "CONTEXT_OR_FILE", num_args = 0..)]
    operands: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum ColorArgument {
    Auto,
    Always,
    Never,
}

impl From<ColorArgument> for ColorMode {
    fn from(value: ColorArgument) -> Self {
        match value {
            ColorArgument::Auto => Self::Auto,
            ColorArgument::Always => Self::Always,
            ColorArgument::Never => Self::Never,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum DeviceArgument {
    Auto,
    Cpu,
}

impl From<DeviceArgument> for Device {
    fn from(value: DeviceArgument) -> Self {
        match value {
            DeviceArgument::Auto => Self::Auto,
            DeviceArgument::Cpu => Self::Cpu,
        }
    }
}

#[derive(Clone, Copy)]
enum ContextOption {
    Before,
    After,
    Both,
}

/// Parse command-line arguments while retaining the occurrence order of
/// `-A`, `-B`, and `-C`. Clap exposes each option's value index through
/// `ArgMatches`; using those indices gives the same last-occurrence behavior
/// as grep even when these options are interleaved or repeated.
pub fn parse() -> Cli {
    parse_from(std::env::args_os()).unwrap_or_else(|error| error.exit())
}

fn parse_from<I, T>(arguments: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = Cli::command().try_get_matches_from(arguments)?;
    let mut cli = Cli::from_arg_matches(&matches)?;
    cli.apply_context_order(&matches);
    Ok(cli)
}

impl Cli {
    /// Convert parsed values into the execution configuration and reject mode
    /// combinations which would have surprising semantics.
    ///
    /// # Errors
    ///
    /// Returns an error when positional operands are ambiguous or missing, or
    /// when flags select incompatible search modes.
    pub fn into_config(self) -> Result<SearchConfig> {
        let before_context = self.resolved_before_context;
        let after_context = self.resolved_after_context;
        let mode = if self.extended_regexp {
            SearchMode::Regex
        } else if self.fixed_strings {
            SearchMode::Fixed
        } else {
            SearchMode::Semantic
        };

        self.validate_mode(mode)?;

        let mut patterns = self.patterns;
        let paths = if patterns.is_empty() {
            match self.operands.split_first() {
                Some((context, files)) => {
                    patterns.push(context.clone());
                    files.iter().map(PathBuf::from).collect()
                }
                None if self.download_model => Vec::new(),
                None => {
                    return Err(anyhow!(
                        "a search context is required (or use --download-model)"
                    ));
                }
            }
        } else {
            self.operands.into_iter().map(PathBuf::from).collect()
        };

        if mode == SearchMode::Semantic && patterns.iter().any(String::is_empty) {
            bail!("search contexts must not be empty");
        }

        let threshold = self.threshold.unwrap_or(0.5);
        if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
            bail!("--threshold must be a finite number from 0.0 through 1.0");
        }

        let output_mode = if self.quiet {
            OutputMode::Quiet
        } else if self.files_with_matches {
            OutputMode::FilesWithMatches
        } else if self.files_without_match {
            OutputMode::FilesWithoutMatches
        } else if self.count {
            OutputMode::Count
        } else {
            OutputMode::Lines
        };

        Ok(SearchConfig {
            mode,
            patterns,
            paths,
            invert_match: self.invert_match,
            line_number: self.line_number,
            with_filename: self.with_filename,
            no_filename: self.no_filename,
            output_mode,
            max_count: self.max_count,
            recursive: self.recursive,
            before_context,
            after_context,
            include: self.include,
            exclude: self.exclude,
            color: self.color.into(),
            line_buffered: self.line_buffered,
            ignore_case: self.ignore_case,
            semantic: SemanticOptions {
                model_path: self.model,
                offline: self.offline,
                device: self.device.map_or(Device::Auto, Into::into),
                threshold,
                report_score: self.score,
            },
            download_model: self.download_model,
        })
    }

    fn validate_mode(&self, mode: SearchMode) -> Result<()> {
        if mode != SearchMode::Semantic {
            if self.threshold.is_some() {
                bail!("--threshold is only available in semantic mode");
            }
            if self.score {
                bail!("--score is only available in semantic mode");
            }
            if self.model.is_some() || self.offline || self.download_model || self.device.is_some()
            {
                bail!("model options are only available in semantic mode");
            }
        } else if self.ignore_case {
            bail!("-i/--ignore-case is only available with -E or -F");
        }

        Ok(())
    }

    fn apply_context_order(&mut self, matches: &ArgMatches) {
        let mut occurrences = Vec::new();
        append_context_occurrences(
            matches,
            "after_context",
            ContextOption::After,
            &mut occurrences,
        );
        append_context_occurrences(
            matches,
            "before_context",
            ContextOption::Before,
            &mut occurrences,
        );
        append_context_occurrences(matches, "context", ContextOption::Both, &mut occurrences);
        occurrences.sort_by_key(|(index, _, _)| *index);

        let mut before = 0;
        let mut after = 0;
        for (_, option, value) in occurrences {
            match option {
                ContextOption::Before => before = value,
                ContextOption::After => after = value,
                ContextOption::Both => {
                    before = value;
                    after = value;
                }
            }
        }
        self.resolved_before_context = before;
        self.resolved_after_context = after;
    }
}

fn append_context_occurrences(
    matches: &ArgMatches,
    id: &str,
    option: ContextOption,
    occurrences: &mut Vec<(usize, ContextOption, usize)>,
) {
    let indices = matches.indices_of(id).into_iter().flatten();
    let values = matches.get_many::<usize>(id).into_iter().flatten();
    for (index, value) in indices.zip(values) {
        occurrences.push((index, option, *value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_context(arguments: &[&str]) -> (usize, usize) {
        let config = parse_from(arguments.iter().copied())
            .expect("CLI should parse")
            .into_config()
            .expect("CLI should normalize");
        (config.before_context, config.after_context)
    }

    #[test]
    fn context_options_follow_their_last_occurrence() {
        assert_eq!(
            parse_context(&["jgrep", "-F", "-B", "1", "-C", "2", "needle"]),
            (2, 2)
        );
        assert_eq!(
            parse_context(&["jgrep", "-F", "-C", "2", "-B", "1", "needle"]),
            (1, 2)
        );
        assert_eq!(
            parse_context(&["jgrep", "-F", "-A", "1", "-A", "3", "-B", "2", "needle"]),
            (2, 3)
        );
    }
}

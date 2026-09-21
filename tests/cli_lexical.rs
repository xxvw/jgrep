//! End-to-end coverage for the lexical modes.
//!
//! These tests intentionally use `-F` or `-E` so they never download or load
//! the semantic model.  They document the grep-compatible contract shared by
//! the native and release builds.

use assert_cmd::Command as AssertCommand;
use jgrep::model::DEFAULT_MODEL_FILE;
use predicates::prelude::*;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command as ProcessCommand, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;
use tempfile::TempDir;

fn jgrep() -> AssertCommand {
    AssertCommand::new(
        std::env::var_os("CARGO_BIN_EXE_jgrep")
            .expect("Cargo should expose the jgrep binary to integration tests"),
    )
}

#[test]
fn line_buffered_stdin_emits_a_match_before_the_producer_closes() {
    let executable = std::env::var_os("CARGO_BIN_EXE_jgrep")
        .expect("Cargo should expose the jgrep binary to integration tests");
    let mut child = ProcessCommand::new(executable)
        .args(["--color=never", "--line-buffered", "-F", "needle"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start jgrep");
    let mut input = child.stdin.take().expect("piped stdin");
    let output = child.stdout.take().expect("piped stdout");
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(output).read_line(&mut line).map(|_| line);
        sender
            .send(result)
            .expect("test receiver should remain open");
    });

    input
        .write_all(b"needle from an open producer\n")
        .expect("write stdin fixture");
    input.flush().expect("flush stdin fixture");

    // `input` remains open while waiting. A whole-stdin implementation would
    // time out here instead of returning the first selected line.
    let first_line = receiver.recv_timeout(Duration::from_secs(5));
    drop(input);
    let status = child.wait().expect("wait for jgrep");
    reader.join().expect("join stdout reader");

    assert!(
        status.success(),
        "jgrep should finish successfully: {status}"
    );
    assert_eq!(
        first_line
            .expect("jgrep should emit before stdin is closed")
            .expect("read stdout line"),
        "needle from an open producer\n"
    );
}

#[test]
fn broken_piped_stdout_after_a_match_exits_without_line_buffering() {
    let executable = std::env::var_os("CARGO_BIN_EXE_jgrep")
        .expect("Cargo should expose the jgrep binary to integration tests");
    let mut child = ProcessCommand::new(executable)
        .args(["--color=never", "-F", "needle"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start jgrep");

    // Close the read end before giving jgrep a matching line. This avoids a
    // timing race: the child cannot write before its stdout consumer is gone.
    drop(child.stdout.take().expect("piped stdout"));

    let mut input = child.stdin.take().expect("piped stdin");
    input
        .write_all(b"needle delivered after stdout closes\n")
        .expect("write stdin fixture");
    input.flush().expect("flush stdin fixture");

    let (sender, receiver) = mpsc::channel();
    let waiter = thread::spawn(move || {
        sender
            .send(child.wait_with_output())
            .expect("test receiver should remain open");
    });
    // Keep stdin open here. Without `--line-buffered`, a pipe destination
    // must still be flushed so EPIPE is observed before jgrep asks for a
    // second input line.
    let output = receiver.recv_timeout(Duration::from_secs(5));
    drop(input);
    waiter.join().expect("join jgrep waiter");
    let output = output
        .expect("jgrep should exit after a broken stdout pipe")
        .expect("wait for jgrep");
    assert!(
        output.status.success(),
        "a closed stdout pipe after a match should not be an error: {output:?}"
    );
    assert!(
        output.stderr.is_empty(),
        "a closed stdout pipe should not produce a diagnostic: {output:?}"
    );
}

#[test]
fn ai_output_handles_a_broken_pipe_without_a_false_limit_notice() {
    let executable = std::env::var_os("CARGO_BIN_EXE_jgrep")
        .expect("Cargo should expose the jgrep binary to integration tests");
    let mut child = ProcessCommand::new(executable)
        .args(["--ai", "-F", "needle"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start jgrep");

    drop(child.stdout.take().expect("piped stdout"));
    let mut input = child.stdin.take().expect("piped stdin");
    input
        .write_all(b"needle delivered after stdout closes\n")
        .expect("write stdin fixture");
    input.flush().expect("flush stdin fixture");

    let (sender, receiver) = mpsc::channel();
    let waiter = thread::spawn(move || {
        sender
            .send(child.wait_with_output())
            .expect("test receiver should remain open");
    });
    let output = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("jgrep should exit after a broken stdout pipe")
        .expect("wait for jgrep");
    drop(input);
    waiter.join().expect("join jgrep waiter");

    assert!(
        output.status.success(),
        "AI mode should tolerate EPIPE: {output:?}"
    );
    assert!(
        output.stderr.is_empty(),
        "a closed stdout pipe must not look like an AI result cap: {output:?}"
    );
}

#[cfg(unix)]
#[test]
fn ai_cap_does_not_reopen_a_named_fifo() {
    let root = tempfile::tempdir().expect("temporary FIFO fixture directory");
    let fifo = root.path().join("stream.fifo");
    let status = ProcessCommand::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("create FIFO fixture");
    assert!(
        status.success(),
        "mkfifo should create the fixture: {status}"
    );

    let (release_writer, wait_for_release) = mpsc::channel();
    let writer_fifo = fifo.clone();
    let writer = thread::spawn(move || {
        let mut stream = fs::OpenOptions::new()
            .write(true)
            .open(writer_fifo)
            .expect("open FIFO writer once jgrep starts reading");
        stream
            .write_all(b"needle from an open FIFO writer\n")
            .expect("write FIFO fixture");
        stream.flush().expect("flush FIFO fixture");
        // Keep the producer open. A post-cap reopen of the FIFO would block
        // here until this test releases it.
        let _ = wait_for_release.recv_timeout(Duration::from_secs(10));
    });

    let executable = std::env::var_os("CARGO_BIN_EXE_jgrep")
        .expect("Cargo should expose the jgrep binary to integration tests");
    let mut child = ProcessCommand::new(executable)
        .args(["--ai", "--ai-max-results", "1", "-F", "needle"])
        .arg(&fifo)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start jgrep against FIFO");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut completed = None;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("poll jgrep process") {
            completed = Some(status);
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }

    if completed.is_none() {
        let _ = child.kill();
    }
    let _ = release_writer.send(());
    let _ = child.wait();
    writer.join().expect("join FIFO writer");

    assert!(
        completed.is_some_and(|status| status.success()),
        "jgrep must finish after its AI cap without reopening an open FIFO"
    );
}

fn write_file(root: &TempDir, relative_path: &str, contents: impl AsRef<[u8]>) {
    let path = root.path().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture directory");
    }
    fs::write(path, contents).expect("write fixture file");
}

#[test]
fn fixed_string_search_reads_stdin_and_uses_grep_exit_codes() {
    jgrep()
        .args(["--color=never", "-F", "needle"])
        .write_stdin("haystack\nneedle\nneedle again\n")
        .assert()
        .success()
        .stdout("needle\nneedle again\n")
        .stderr(predicate::str::is_empty());

    jgrep()
        .args(["--color=never", "-F", "missing"])
        .write_stdin("haystack\nneedle\n")
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());
}

#[test]
fn help_is_available_without_a_context_or_model() {
    jgrep()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: jgrep"));
}

#[test]
fn fixed_string_search_handles_files_and_filename_controls() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "one.txt", "first\nneedle in one\n");
    write_file(&root, "two.txt", "needle in two\nlast\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-F", "needle", "one.txt", "two.txt"])
        .assert()
        .success()
        .stdout("one.txt:needle in one\ntwo.txt:needle in two\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-H", "-F", "needle", "one.txt"])
        .assert()
        .success()
        .stdout("one.txt:needle in one\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-h", "-F", "needle", "one.txt", "two.txt"])
        .assert()
        .success()
        .stdout("needle in one\nneedle in two\n");
}

#[test]
fn ai_mode_emits_compact_locations_with_a_global_budget() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "one.txt", "needle first\nordinary\nneedle second\n");
    write_file(&root, "two.txt", "needle third\nneedle fourth\n");

    jgrep()
        .current_dir(root.path())
        .args([
            "--ai",
            "--ai-max-results",
            "3",
            "-F",
            "needle",
            "one.txt",
            "two.txt",
        ])
        .assert()
        .success()
        .stdout("one.txt:1\none.txt:3\ntwo.txt:1\n")
        .stderr(predicate::str::contains(
            "--ai stopped after 3 results; output may be incomplete",
        ));
}

#[test]
fn ai_limit_keeps_later_explicit_binary_inputs_as_errors() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "first.txt", "needle first\n");
    write_file(&root, "binary.txt", b"needle\0not text\n");

    jgrep()
        .current_dir(root.path())
        .args([
            "--ai",
            "--ai-max-results",
            "1",
            "-F",
            "needle",
            "first.txt",
            "binary.txt",
        ])
        .assert()
        .code(2)
        .stdout("first.txt:1\n")
        .stderr(
            predicate::str::contains("binary input")
                .and(predicate::str::contains("--ai stopped after 1 results")),
        );

    let mut late_binary = b"needle first\n".to_vec();
    late_binary.extend(std::iter::repeat_n(b'x', 9_000));
    late_binary.extend_from_slice(b"\0late binary marker\n");
    write_file(&root, "late-binary.txt", late_binary);
    jgrep()
        .current_dir(root.path())
        .args([
            "--ai",
            "--ai-max-results",
            "1",
            "-F",
            "needle",
            "late-binary.txt",
        ])
        .assert()
        .code(2)
        .stdout("late-binary.txt:1\n")
        .stderr(predicate::str::contains("binary input"));
}

#[test]
fn ai_mode_supports_lexical_filters_and_stdin_without_source_text() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "events.txt", "Needle details\nordinary line\n");

    jgrep()
        .current_dir(root.path())
        .args(["--ai", "-i", "-F", "needle", "events.txt"])
        .assert()
        .success()
        .stdout("events.txt:1\n")
        .stderr(predicate::str::is_empty());

    jgrep()
        .current_dir(root.path())
        .args(["--ai", "-E", "Needle|ordinary", "events.txt"])
        .assert()
        .success()
        .stdout("events.txt:1\nevents.txt:2\n")
        .stderr(predicate::str::is_empty());

    jgrep()
        .args(["--ai", "-F", "needle"])
        .write_stdin("ordinary\nneedle content that must not be printed\n")
        .assert()
        .success()
        .stdout("-:2\n")
        .stderr(predicate::str::is_empty());

    // The reserved stdin label must not collide with a normal file operand
    // named `stdin`.
    write_file(&root, "stdin", "needle from file\n");
    jgrep()
        .current_dir(root.path())
        .args(["--ai", "-F", "needle", "stdin", "-"])
        .write_stdin("needle from pipe\n")
        .assert()
        .success()
        .stdout("stdin:1\n-:1\n")
        .stderr(predicate::str::is_empty());

    jgrep()
        .current_dir(root.path())
        .args(["--ai", "-v", "-i", "-F", "needle", "events.txt"])
        .assert()
        .success()
        .stdout("events.txt:2\n")
        .stderr(predicate::str::is_empty());

    write_file(&root, "hyphen-context.txt", "-needle context\n");
    jgrep()
        .current_dir(root.path())
        .args(["--ai", "-F", "-e", "-needle", "hyphen-context.txt"])
        .assert()
        .success()
        .stdout("hyphen-context.txt:1\n")
        .stderr(predicate::str::is_empty());
}

#[cfg(unix)]
#[test]
fn ai_mode_rejects_line_breaks_in_file_labels() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    let path = "line\nbreak.txt";
    write_file(&root, path, "needle\n");

    jgrep()
        .current_dir(root.path())
        .args(["--ai", "-F", "needle", path])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "cannot emit a path containing a line break",
        ));
}

#[test]
fn line_numbers_counts_file_lists_and_quiet_mode_match_grep_expectations() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "has-match.txt", "first\nneedle\nneedle again\n");
    write_file(&root, "no-match.txt", "first\nlast\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-n", "-F", "needle", "has-match.txt"])
        .assert()
        .success()
        .stdout("2:needle\n3:needle again\n");

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "-c",
            "-F",
            "needle",
            "has-match.txt",
            "no-match.txt",
        ])
        .assert()
        .success()
        .stdout("has-match.txt:2\nno-match.txt:0\n");

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "-l",
            "-F",
            "needle",
            "has-match.txt",
            "no-match.txt",
        ])
        .assert()
        .success()
        .stdout("has-match.txt\n");

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "-L",
            "-F",
            "needle",
            "has-match.txt",
            "no-match.txt",
        ])
        .assert()
        .success()
        .stdout("no-match.txt\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-L", "-F", "needle", "has-match.txt"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-L", "-F", "needle", "no-match.txt"])
        .assert()
        .code(1)
        .stdout("no-match.txt\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-q", "-F", "needle", "has-match.txt"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-q", "-F", "needle", "no-match.txt"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());
}

#[test]
fn repeated_patterns_are_or_conditions_and_invert_applies_after_matching() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(
        &root,
        "patterns.txt",
        "alpha first\nneedle second\nother third\n",
    );

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "-F",
            "-e",
            "alpha",
            "-e",
            "needle",
            "patterns.txt",
        ])
        .assert()
        .success()
        .stdout("alpha first\nneedle second\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-v", "-F", "needle", "patterns.txt"])
        .assert()
        .success()
        .stdout("alpha first\nother third\n");
}

#[test]
fn max_count_limits_selection_and_zero_selects_nothing() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "many.txt", "needle one\nneedle two\nneedle three\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-m", "1", "-F", "needle", "many.txt"])
        .assert()
        .success()
        .stdout("needle one\n");

    // This path is also important for semantic mode: `-m 0` must complete
    // without initializing a model.  `-F` keeps the test independent of it.
    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-m", "0", "-F", "needle", "many.txt"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-m", "0", "-L", "-F", "needle", "many.txt"])
        .assert()
        .code(1)
        .stdout("many.txt\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-m", "0", "-c", "-F", "needle", "many.txt"])
        .assert()
        .code(1)
        .stdout("0\n");

    // The default mode is semantic, but this particular invocation must still
    // short-circuit before looking for a model or touching the network.
    jgrep()
        .args([
            "--color=never",
            "--offline",
            "-m",
            "0",
            "a semantic query that has no local model",
        ])
        .write_stdin("any input is sufficient\n")
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());

    // `--download-model` normally warms a cache before a search. `-m 0`
    // takes precedence and must not even inspect an explicitly invalid model.
    jgrep()
        .args([
            "--color=never",
            "--download-model",
            "--model",
            "not-a-gguf.gguf",
            "-m",
            "0",
            "a semantic query",
        ])
        .write_stdin("any input is sufficient\n")
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn recursive_search_filters_and_orders_paths_deterministically() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "tree/z.txt", "needle ignored by include\n");
    write_file(&root, "tree/b.log", "needle beta\n");
    write_file(&root, "tree/a.log", "needle alpha\n");
    write_file(&root, "tree/nested/skip.log", "needle skipped\n");
    write_file(&root, "tree/nested/c.log", "needle charlie\n");

    let separator = std::path::MAIN_SEPARATOR;
    let expected = format!(
        "tree{separator}a.log:needle alpha\n\
         tree{separator}b.log:needle beta\n\
         tree{separator}nested{separator}c.log:needle charlie\n"
    );

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "-r",
            "--include",
            "*.log",
            "--exclude",
            "skip*",
            "-F",
            "needle",
            "tree",
        ])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn before_after_and_context_windows_do_not_duplicate_lines() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(
        &root,
        "context.txt",
        "zero\nneedle one\nafter one\ngap\nbefore two\nneedle two\ntail\n",
    );

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-A", "1", "-F", "needle", "context.txt"])
        .assert()
        .success()
        .stdout("needle one\nafter one\n--\nneedle two\ntail\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-B", "1", "-F", "needle", "context.txt"])
        .assert()
        .success()
        .stdout("zero\nneedle one\n--\nbefore two\nneedle two\n");

    write_file(
        &root,
        "overlap.txt",
        "first\nneedle one\nshared\nneedle two\nlast\n",
    );
    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-C", "1", "-F", "needle", "overlap.txt"])
        .assert()
        .success()
        .stdout("first\nneedle one\nshared\nneedle two\nlast\n");
}

#[test]
fn huge_context_window_is_a_normal_search_not_a_capacity_panic() {
    let enormous = usize::MAX.to_string();

    jgrep()
        .args(["--color=never", "-C", enormous.as_str(), "-F", "needle"])
        .write_stdin("ordinary input\n")
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn regex_fixed_and_case_insensitive_modes_are_distinct() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(
        &root,
        "events.txt",
        "warning lowercase\nWARN upper\nERROR upper\nliteral ERROR|WARN\n",
    );

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-E", "WARN|ERROR", "events.txt"])
        .assert()
        .success()
        .stdout("WARN upper\nERROR upper\nliteral ERROR|WARN\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-F", "ERROR|WARN", "events.txt"])
        .assert()
        .success()
        .stdout("literal ERROR|WARN\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-i", "-F", "WARNING", "events.txt"])
        .assert()
        .success()
        .stdout("warning lowercase\n");
}

#[test]
fn explicit_dash_reads_stdin_alongside_files() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "on-disk.txt", "needle from file\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-F", "needle", "on-disk.txt", "-"])
        .write_stdin("needle from stdin\n")
        .assert()
        .success()
        .stdout("on-disk.txt:needle from file\n(standard input):needle from stdin\n");
}

#[test]
fn crlf_unicode_and_space_containing_paths_are_handled_as_text() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    let path = format!(
        "日本語 directory{}space file.txt",
        std::path::MAIN_SEPARATOR
    );
    write_file(
        &root,
        path.as_str(),
        "first line\r\nNEEDLE 東京\r\nlast line\r\n",
    );

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "-H",
            "-i",
            "-F",
            "needle 東京",
            path.as_str(),
        ])
        .assert()
        .success()
        .stdout(format!("{path}:NEEDLE 東京\n"));
}

#[test]
fn explicit_binary_input_is_an_error_but_recursive_binary_input_is_skipped() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "binary.dat", b"needle\0not text\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-F", "needle", "binary.dat"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("binary input"));

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-m", "0", "-F", "needle", "binary.dat"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("binary input"));

    write_file(&root, "tree/a-binary.dat", b"needle\0not text\n");
    write_file(&root, "tree/z-text.txt", "needle from text\n");
    let separator = std::path::MAIN_SEPARATOR;

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-r", "-F", "needle", "tree"])
        .assert()
        .success()
        .stdout(format!("tree{separator}z-text.txt:needle from text\n"))
        .stderr(predicate::str::contains("binary input"));

    let mut late_binary = b"needle emitted only if binary preflight is broken\n".to_vec();
    late_binary.extend(std::iter::repeat_n(b'x', 9_000));
    late_binary.extend_from_slice(b"\0late binary marker\n");
    write_file(&root, "late-tree/late-binary.dat", late_binary);

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-r", "-F", "needle", "late-tree"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("binary input"));
}

#[cfg(unix)]
#[test]
fn recursive_search_skips_an_explicit_symbolic_link_root() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "real/needle.txt", "needle behind a link\n");
    symlink(root.path().join("real"), root.path().join("linked-root"))
        .expect("create directory symlink");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-r", "-F", "needle", "linked-root"])
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("symbolic link"));
}

#[test]
fn offline_semantic_search_with_a_missing_explicit_model_fails_without_output() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(
        &root,
        "semantic-input.txt",
        "The connection failed while uploading.\n",
    );

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "--offline",
            "--model",
            "missing-model.gguf",
            "network connection failure",
            "semantic-input.txt",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("model"));
}

#[test]
fn offline_semantic_search_rejects_a_corrupt_default_cached_model_without_download() {
    let cache = tempfile::tempdir().expect("temporary model cache");
    let model_path = cache.path().join(DEFAULT_MODEL_FILE);
    let corrupt_model = b"this intentionally is not the pinned Qwen GGUF model";
    fs::write(&model_path, corrupt_model).expect("write corrupt cached model");

    jgrep()
        .env("JGREP_MODEL_DIR", cache.path())
        .args(["--color=never", "--offline", "network connection failure"])
        .write_stdin("The connection failed while uploading.\n")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("cached default model is invalid")
                .and(predicate::str::contains("--offline forbids replacement")),
        );

    assert_eq!(
        fs::read(&model_path).expect("read corrupt cached model"),
        corrupt_model,
        "offline mode must not replace a corrupt cached model"
    );
    let has_partial_download = fs::read_dir(cache.path())
        .expect("read model cache")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .any(|name| name.to_string_lossy().ends_with(".part"));
    assert!(
        !has_partial_download,
        "offline mode must not create a partial model download"
    );
}

#[test]
fn invalid_modes_and_operands_report_a_command_error() {
    let root = tempfile::tempdir().expect("temporary fixture directory");
    write_file(&root, "input.txt", "needle\n");
    write_file(&root, "-starts-with-a-dash.txt", "needle from dash path\n");

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-E", "-F", "needle", "input.txt"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("cannot"));

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-i", "needle", "input.txt"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("-i"));

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "--threshold",
            "0.75",
            "-F",
            "needle",
            "input.txt",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--threshold"));

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "--score", "-F", "needle", "input.txt"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--score"));

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-E", "[", "input.txt"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("regex"));

    jgrep()
        .current_dir(root.path())
        .args(["--color=never", "-F", "needle", "does-not-exist.txt"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("does-not-exist.txt"));

    jgrep()
        .current_dir(root.path())
        .args([
            "--color=never",
            "-F",
            "needle",
            "--",
            "-starts-with-a-dash.txt",
        ])
        .assert()
        .success()
        .stdout("needle from dash path\n");
}

<p align="center">
  <img src="assets/jgrep-logo.png" alt="jgrep" width="680">
</p>

<p align="center">
  <a href="https://github.com/xxvw/jgrep/actions/workflows/ci.yml"><img src="https://github.com/xxvw/jgrep/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/xxvw/jgrep/releases/latest"><img src="https://img.shields.io/github/v/release/xxvw/jgrep" alt="Latest release"></a>
  <a href="https://github.com/xxvw/jgrep/blob/main/LICENSE"><img src="https://img.shields.io/github/license/xxvw/jgrep" alt="License"></a>
  <a href="https://github.com/xxvw/jgrep/releases"><img src="https://img.shields.io/github/downloads/xxvw/jgrep/total" alt="Downloads"></a>
</p>

`jgrep` is a local, semantic grep-style command. It prints input lines whose
meaning matches a natural-language query, while preserving the familiar
file-and-pipe workflow of `grep`.

> **Status:** Versioned native archives are published through
> [GitHub Releases](https://github.com/xxvw/jgrep/releases). This
> project makes no benchmark or accuracy guarantees.

```sh
jgrep "network connection failure" app.log
cat app.log | jgrep "network connection failure"
jgrep -n -r --include '*.log' "authentication failed" logs/
jgrep -e "timeout" -e "connection refused" a.log b.log
jgrep -E 'ERROR|WARN' app.log
jgrep -F 'connection refused' app.log
```

## Quickstart for Codex

Install the `jgrep` executable from [GitHub Releases](https://github.com/xxvw/jgrep/releases),
then add this repository as a Codex plugin marketplace and install `jgrep-agent`:

```sh
codex plugin marketplace add xxvw/jgrep --ref main \
  --sparse .agents/plugins --sparse plugins/jgrep-agent
codex plugin add jgrep-agent@jgrep
```

Start a new Codex session so the plugin is loaded. It teaches Codex to use the
compact `jgrep --ai` location protocol first and to read only the source ranges
that matter. The executable and plugin are separate: the process running Codex
must be able to find `jgrep` on `PATH`. See the
[plugin guide](plugins/jgrep-agent/README.md) and OpenAI's
[plugin documentation](https://developers.openai.com/plugins/build/plugins)
for details.

## What it is

By default, `jgrep` treats the pattern as a semantic context rather than a
literal string. For every input line, it asks a local
[Qwen2.5-0.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct)
model for a binary relevance decision and prints matching source text. It does
not send searched text to a hosted model, require Python or Ollama, or run a
background service.

`jgrep` also has conventional lexical modes: `-E` selects Rust regular
expressions and `-F` selects fixed-string matching. These modes never download
or load a model.

The project is independently developed and is **not affiliated with, endorsed
by, or a distribution of** Jev, TypeSafe, Qwen, Hugging Face, or llama.cpp.
Jev is an inspiration for the binary-decision interaction model; see
[TypeSafe's System One documentation](https://docs.typesafe.ai/concepts/system-one).

## Read this in your language

The English README is canonical. Translations are maintained as concise usage
guides and may lag behind it.

| Language | README |
| --- | --- |
| English | [README.md](README.md) |
| 日本語 | [README.ja.md](README.ja.md) |
| 简体中文 | [README.zh-CN.md](README.zh-CN.md) |
| 한국어 | [README.ko.md](README.ko.md) |
| Español | [README.es.md](README.es.md) |
| Deutsch | [README.de.md](README.de.md) |
| Русский | [README.ru.md](README.ru.md) |
| Français | [README.fr.md](README.fr.md) |
| Português (Brasil) | [README.pt-BR.md](README.pt-BR.md) |
| Italiano | [README.it.md](README.it.md) |
| العربية | [README.ar.md](README.ar.md) |
| हिन्दी | [README.hi.md](README.hi.md) |

## Install

Download the archive for macOS on Apple Silicon or Intel, Windows x64, or Linux
x64 (glibc 2.35 or later) from
[GitHub Releases](https://github.com/xxvw/jgrep/releases).

For a clone-free, version-pinned installation, copy the verified Bash/zsh or
PowerShell command in [installation and agent integration](docs/installation-and-agents.md).
It downloads the `v0.1.1` installer bundle, checks its SHA-256 file before
extracting it, and then runs a local installer. The guide also has one-paste
commands for every supported README language, offline asset installation, and
coding-agent setup.

The `v0.1.1` release predates the repository rename, so its immutable archive
and installer directory names still begin with `localjev-grep`. Repository
URLs, source builds, and the installed command use `jgrep`.

To build from source:

```sh
git clone https://github.com/xxvw/jgrep.git
cd jgrep
cargo build --release
./target/release/jgrep --help
```

Windows PowerShell:

```powershell
git clone https://github.com/xxvw/jgrep.git
Set-Location jgrep
cargo build --release
.\target\release\jgrep.exe --help
```

Source builds use the pinned Rust toolchain in `rust-toolchain.toml`, CMake,
and a C++ compiler that can build the embedded llama.cpp dependency. On Apple Silicon, the
automatic device setting may use Metal; CPU execution is available on every
supported platform. Release builds intentionally avoid host-specific CPU
instructions. The Windows x64 archive uses the standard Microsoft Visual C++
runtime; install the current Visual C++ Redistributable if it is not already
available on the machine.

## Usage

The normal operand order follows `grep`:

```text
jgrep [OPTIONS] <context> [FILE ...]
```

With no file operands, or with `-` as a file operand, `jgrep` reads standard
input. A query supplied with `-e` may be used instead of the positional
`<context>`; repeated `-e` patterns are ORed.

Bash / zsh:

```sh
# Semantic search (the default)
jgrep 'a database login was rejected' service.log

# Search a pipeline without buffering the whole input
journalctl -f | jgrep --line-buffered 'connection was reset'

# Search recursively and show line numbers
jgrep -n -r --include '*.rs' 'handling a filesystem error' src/

# Context output
jgrep -C 2 'request timed out' app.log

# Conventional matching, with no model initialization
jgrep -E -i 'error|warning' app.log
jgrep -F 'connection refused' app.log
```

### Search modes

| Mode | Selection | Meaning |
| --- | --- | --- |
| Semantic | default | Local model judges whether each line is relevant to the context. |
| Extended regex | `-E` | Rust `regex` syntax. This is not GNU BRE, PCRE, or backreference compatible. |
| Fixed string | `-F` | Literal substring matching. |

`-E` and `-F` cannot be combined. `-i` is supported only in lexical modes.
Semantic-only flags such as `--threshold` and `--score` are rejected in
lexical modes, so a typo cannot silently change how a query is interpreted.

### Frequently used options

| Option | Purpose |
| --- | --- |
| `-e <context>` | Add a context; any matching context selects the line. |
| `-n` | Prefix output with line numbers. |
| `-H`, `-h` | Always show, or suppress, file names. |
| `-c` | Print a selected-line count per input. |
| `-l`, `-L` | Print names with a match, or names without a match. |
| `-q` | Stop after the first selected line and suppress normal output. |
| `-m <NUM>` | Stop after NUM selected lines per input. `-m 0` does no model work. |
| `-v` | Invert the final selection. |
| `-r` | Search directories recursively in deterministic path order. |
| `-A`, `-B`, `-C` | Include trailing, leading, or surrounding context lines. |
| `--include <GLOB>` | Limit recursive candidates to matching paths. |
| `--exclude <GLOB>` | Skip matching recursive paths. |
| `--color <auto\|always\|never>` | Control ANSI highlighting. `auto` disables it when stdout is a pipe. |
| `--line-buffered` | Flush each output line for streaming pipelines. |
| `--ai` | Emit compact `path:line` locations for coding agents, without source text. |
| `--ai-max-results <NUM>` | Set the positive, whole-invocation location limit for `--ai`; default `50`. |
| `--threshold <0..1>` | Semantic relevance cutoff; default `0.5`. |
| `--score` | Add the semantic relevance score to selected output. |
| `--model <PATH>` | Use an explicit local GGUF model file. |
| `--download-model` | Download the pinned default model first; with no context, exit after the download. |
| `--offline` | Never use the network; fail if no valid local model is available. |
| `--device <auto\|cpu>` | Select the local inference device. |

Use `--` before a pattern or path that begins with `-`.

### AI-agent output

`--ai` is a token-efficient first pass for coding agents and tool calls. It
keeps the normal matcher and exit codes, but emits one UTF-8 record per
selected line in this form:

```text
path/to/file.rs:42
-:7
```

Codex users can install the ready-made
[`jgrep-agent` plugin](plugins/jgrep-agent/README.md), including its localized
agent guides.

Records contain neither source text nor ANSI color, semantic scores, or
context lines. The default is at most **50 locations across the entire
invocation**, including recursive and multi-file searches. Increase it only
when needed with `--ai-max-results <NUM>`; it accepts positive integers. When
the limit is reached, stderr says that the output may be incomplete. Split a
record at the final colon before its decimal line number so Windows drive
prefixes remain valid. `-:LINE` identifies a reproducible pipeline rather
than retaining its input for a later read. To preserve one-record-per-line
output, `--ai` rejects a file path containing CR or LF.

Use it to locate candidates, then read only narrow ranges around the returned
locations. For example:

```sh
jgrep --ai -r --include '*.rs' -F 'resolve_config' src/
# Then inspect only the cited source range:
sed -n '38,48p' src/config.rs
```

`--ai` supports semantic, `-E`, and `-F` matching; lexical `-i`, `-v`,
`-r`, `--include`, and `--exclude` remain available. It rejects `-c`, `-l`,
`-L`, `-q`, `-m`, `-h`, `-A`, `-B`, `-C`, and `--score`, because those flags
would hide, expand, or change the compact location protocol. `-H`, `-n`, and
`--line-buffered` are accepted but redundant. `--color=always` is rejected;
agent output never contains ANSI escapes. See
[installation and agent integration](docs/installation-and-agents.md) for
installer and template usage.

## Codex token benchmark

On a controlled three-task location-search benchmark, `jgrep --ai -F` used
**16.4% fewer total Codex tokens** than `rg -F`. Non-cached input tokens were
54.9% lower, and search-tool output was 97.1% smaller.

| Search tool | Total tokens | Non-cached input tokens | Tool output |
| --- | ---: | ---: | ---: |
| `rg` | 91,278 | 34,703 | 95,421 characters |
| `jgrep --ai` | 76,334 | 15,662 | 2,739 characters |

This is a one-repetition fixed-string microbenchmark, not a general performance
or accuracy guarantee. The location lists agreed for two tasks; in the broad
project-name task, the `rg`-arm Codex response skipped one location that was
present in its raw tool output. See the
[methodology and limitations](benchmark/README.md), the
[result summary](benchmark/RESULTS-2026-09-22.md), the
[per-run data](benchmark/results/2026-09-22.json), and the
[raw execution logs](benchmark/logs/2026-09-22/README.md).

## Model, privacy, and scores

Semantic mode uses the official Qwen2.5-0.5B-Instruct GGUF **Q8_0** artifact
(about 676 MB). The program pins the model revision and SHA-256 in source,
downloads it only when semantic inference is first needed (or when
`--download-model` is used), verifies it before atomically placing it in the
per-user application cache, and reuses it afterward. The model file and cache
are intentionally excluded from Git and release source archives.

The pinned artifact is revision
`6dd44a1fb35d11b5d1b28902876ce3cc9e882d0e`, file
`qwen2.5-0.5b-instruct-q8_0.gguf`, SHA-256
`ca59ca7f13d0e15a8cfa77bd17e65d24f6844b554a7b6c12e07a5f89ff76844e`.
See [docs/model.md](docs/model.md) for the immutable URL and cache details.

With a context, `--download-model` warms the cache before searching. With no
context, it is a download-only command. If `--model <PATH>` is supplied, it
validates that local GGUF file instead of downloading or replacing it.

Use `--model /path/to/model.gguf` for an existing local file, or `--offline`
to guarantee that an invocation makes no network request. `--help`, lexical
searches, empty input, and `-m 0` do not initialize the model.

`--offline` and `--download-model` conflict. To inspect a score in ordinary
line output, use `--score`; selected records receive a `score=0.743` prefix,
followed by a space and the original line.

The semantic score is computed from the difference between the model's `Yes`
and `No` next-token logits:

```text
sigmoid(logit(Yes) - logit(No))
```

It is a relevance score, not a calibrated probability or guarantee of
correctness. The default threshold is a starting point, not an accuracy claim.
Text is processed locally after the optional model download, but semantic
results can be affected by ambiguous wording, negation, languages, long lines,
and adversarial content in searched files. Do not use it as the sole decision
maker for safety-, legal-, medical-, or security-critical work.

## I/O and compatibility

`jgrep` reads incrementally, emits selected lines in input order, supports
UTF-8 text with LF or CRLF line endings, and accepts Unicode paths. It does not
silently truncate a line that exceeds the semantic prompt's 4,096-token limit.
Recursive search
does not follow directory symlinks; detected binary files encountered during
recursion are skipped with a diagnostic, while an explicitly named non-text
file is an error. Diagnostics and download progress go to stderr.

Like `grep`, its exit status is `0` when a selection is found, `1` when none is
found, and `2` for an error. Broken output pipes are handled quietly where the
platform permits. `jgrep` intentionally does not implement GNU basic regular
expressions, PCRE, shell-glob operand expansion, or every GNU grep flag.

## Development

Run the common local check entry point before opening a pull request:

```sh
cargo xtask ci
```

It runs formatting, linting, unit/integration tests, and documentation checks.
The test suite separates ordinary CLI behavior from model-backed smoke tests.
The small versioned fixture and its local reference results are documented in
[eval/](eval/README.md) and [RESULTS-v0.1.0.md](eval/RESULTS-v0.1.0.md); they
do not establish a general accuracy, throughput, or latency guarantee. See
[CONTRIBUTING.md](CONTRIBUTING.md), [RELEASING.md](RELEASING.md), and
[docs/](docs/).

## License and notices

Project source code is licensed under **GPL-3.0-or-later**. Read
[LICENSE](LICENSE), [NOTICE](NOTICE), and
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) before redistributing a build.
The default Qwen model is a separately downloaded Apache-2.0 asset and is not
licensed as project source code. Release archives also generate and include a
locked Cargo dependency license inventory with the copied upstream notices.

Security reports are described in [SECURITY.md](SECURITY.md). Community
expectations are in [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

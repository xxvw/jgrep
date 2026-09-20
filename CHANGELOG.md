# Changelog

All notable changes to localjev-grep will be documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
released versions will use [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-20

### Added

- The `jgrep` streaming semantic-search CLI with grep-style lexical modes,
  files, recursive traversal, stdin pipelines, context output, and
  grep-compatible exit codes.
- Local Qwen2.5-0.5B-Instruct Q8_0 acquisition with a pinned revision and
  SHA-256, atomic cache installation, offline mode, and CPU/Metal selection.
- Native CI and release packaging for macOS Apple Silicon, macOS Intel,
  Windows x64, and Linux x64, including real-model smoke gates and checksums.
- GPL-3.0-or-later licensing, notices, contribution and security guidance,
  multilingual READMEs, and the initial multilingual semantic evaluation set.
- A reproducible local result record for the versioned semantic fixture,
  including threshold-specific confusion matrices and measurement limits.
- `--ai` compact `path:line` output for coding agents, with a global result
  budget and guidance for fetching only the cited source ranges afterward.
- Bash and PowerShell installers plus a portable `AGENTS.md` template for
  integrating compact `jgrep` searches into coding-agent workflows.

# Raw Codex logs — 2026-09-22

These are the unmodified streams produced by the benchmark run summarized in
[`../../RESULTS-2026-09-22.md`](../../RESULTS-2026-09-22.md). Each ordinary
`.log` file is JSON Lines from `codex exec --json`; each `.stderr.log` file is
the corresponding standard-error stream. The structured result links every
run to these files in [`../../results/2026-09-22.json`](../../results/2026-09-22.json).

| Task | `rg` baseline | `jgrep --ai` treatment |
| --- | --- | --- |
| Repository rename audit | [events](repository-rename-audit.rg.1.log) · [stderr](repository-rename-audit.rg.1.stderr.log) | [events](repository-rename-audit.jgrep.1.log) · [stderr](repository-rename-audit.jgrep.1.stderr.log) |
| Project-name locations | [events](project-name-locations.rg.1.log) · [stderr](project-name-locations.rg.1.stderr.log) | [events](project-name-locations.jgrep.1.log) · [stderr](project-name-locations.jgrep.1.stderr.log) |
| Semantic-feature locations | [events](semantic-feature-locations.rg.1.log) · [stderr](semantic-feature-locations.rg.1.stderr.log) | [events](semantic-feature-locations.jgrep.1.log) · [stderr](semantic-feature-locations.jgrep.1.stderr.log) |

The stderr streams include local Codex startup warnings, including unavailable
optional MCP authentication. They contain no benchmark search results and did
not affect the successful read-only turns. All six event logs were validated as
JSON Lines and parsed successfully with `token-compressor`.

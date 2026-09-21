# Codex token benchmark

This benchmark measures whether `jgrep --ai` reduces the tokens consumed by a
Codex code-location microbenchmark. It runs the same three read-only tasks
twice on the same archived repository commit:

- the baseline uses `rg --hidden --sort path -n -F`, whose matches include
  source text;
- the treatment uses `jgrep --ai -F`, whose matches contain only `path:line`
  locations and are capped at 25 records.

Both arms use the same Codex model, reasoning effort, repository snapshot, and
task wording. The benchmark removes `AGENTS.md` from the temporary snapshot so
repository instructions do not select a search tool before the controlled
prompt does. Each arm performs exactly one search command and returns the first
five locations. Each Codex run is ephemeral and uses a read-only sandbox.

## Run it

Requirements: authenticated Codex CLI, `jq`, `rg`, `jgrep`, Git, and tar.

```sh
BENCHMARK_OUTPUT=benchmark/results/$(date +%F).json \
  CODEX_MODEL=gpt-5.6-luna \
  CODEX_REASONING=low \
  BENCHMARK_REPETITIONS=1 \
  ./benchmark/run.sh
```

Raw Codex JSONL and stderr logs are written below `benchmark/.runs/` and are
ignored by Git. The requested result file contains the prompts' answers,
commands, token usage from Codex's `turn.completed.usage`, tool-output character
counts, and aggregates.

## What the numbers mean

`total_tokens` is `input_tokens + output_tokens` as reported by Codex. The
report also shows non-cached input tokens (`input_tokens - cached_input_tokens`)
and raw tool-output characters. The latter directly measures the compact
protocol; token counts also include Codex's fixed instructions and reasoning.

This is a small, single-repository fixed-string location-search benchmark, not
a claim about semantic-search accuracy, latency, or every coding task. A single
repetition is especially sensitive to model variance. The checked-in report
records the exact model, CLI versions, commit, tasks, commands, answers, and
repetition count so the result can be audited and rerun.

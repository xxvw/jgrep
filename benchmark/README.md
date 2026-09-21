# Codex token benchmark

This benchmark measures whether `jgrep --ai` reduces the tokens consumed by a
Codex code-location microbenchmark. It runs the same three read-only tasks
twice on the same archived repository commit. Generated artifacts under
`benchmark/**` are excluded from both search arms:

- the baseline uses `rg --hidden --sort path -n -F`, whose matches include
  source text;
- the treatment uses `jgrep --ai -F`, whose matches contain only `path:line`
  locations and are capped at 25 records.

Both arms use the same Codex model, reasoning effort, repository snapshot, and
task wording. The benchmark removes `AGENTS.md` from the temporary snapshot so
repository instructions do not select a search tool before the controlled
prompt does. Each arm performs exactly one search command and returns the first
five locations. Each Codex run is ephemeral and uses a read-only sandbox.

The latest checked-in run measured a **16.4% reduction in total Codex tokens**,
a 54.9% reduction in non-cached input tokens, and a 97.1% reduction in search
output characters. See the [result summary](RESULTS-2026-09-22.md),
[per-run data](results/2026-09-22.json), and
[raw execution logs](logs/2026-09-22/README.md).

## Run it

Requirements: authenticated Codex CLI, `jq`, `rg`, `jgrep`, Git, and tar.

```sh
BENCHMARK_OUTPUT=benchmark/results/$(date +%F).json \
  BENCHMARK_LOG_DIR=benchmark/logs/$(date +%F) \
  CODEX_MODEL=gpt-5.6-luna \
  CODEX_REASONING=low \
  BENCHMARK_REPETITIONS=1 \
  ./benchmark/run.sh
```

Raw Codex JSONL and stderr logs are always written below `benchmark/.runs/`
and ignored by Git. When `BENCHMARK_LOG_DIR` is set, the same unmodified streams
are copied there with `.log` and `.stderr.log` suffixes so an auditable run can
be checked in. The requested result file links each run to those logs and
contains its exact prompt, answer, commands, token usage from Codex's
`turn.completed.usage`, tool-output character counts, and aggregates.

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

## Result translations

Localized README summaries link back to the same auditable result:
[日本語](../README.ja.md), [Deutsch](../README.de.md),
[Español](../README.es.md), [Français](../README.fr.md),
[Italiano](../README.it.md), [Português (Brasil)](../README.pt-BR.md),
[Русский](../README.ru.md), [한국어](../README.ko.md),
[简体中文](../README.zh-CN.md), [العربية](../README.ar.md), and
[हिन्दी](../README.hi.md).

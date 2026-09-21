# v0.1.0 semantic fixture results

This is a reproducible local reference measurement for the versioned
`semantic-v1` fixture. It is deliberately a small, hand-authored fixture, not
a general benchmark, an accuracy guarantee, or a recommended threshold for a
user's corpus.

## Measurement record

| Property | Value |
| --- | --- |
| Date | 2026-09-20 |
| Source / binary | jgrep v0.1.0 release build |
| Fixture | `eval/semantic-v1.jsonl` (24 cases) |
| Model | Qwen2.5-0.5B-Instruct Q8_0 GGUF |
| Model revision | `6dd44a1fb35d11b5d1b28902876ce3cc9e882d0e` |
| Model SHA-256 | `ca59ca7f13d0e15a8cfa77bd17e65d24f6844b554a7b6c12e07a5f89ff76844e` |
| Platform | macOS 26.5, arm64, Apple M5 Max (18 reported CPUs) |
| Device | `--device cpu` |
| Cache state | Model cache was warm; every fixture case starts a fresh `jgrep` process and loads the model again. |

The executable was built with `cargo build --locked --release`. The runner
passed each record's exact text plus one newline through standard input and
used `--offline -q`; no model was downloaded during the measurement.

```sh
cargo xtask eval --device cpu --threshold 0.5
cargo xtask eval --device cpu --threshold 0.299
```

## Results

| Threshold | TP | FP | TN | FN | Errors | Precision | Recall | F1 | Wall time | Throughput |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0.500 (the CLI default) | 3 | 0 | 12 | 9 | 0 | 1.0000 | 0.2500 | 0.4000 | 38.219 s | 0.628 cases/s |
| 0.299 (fixture-specific comparison) | 9 | 0 | 12 | 3 | 0 | 1.0000 | 0.7500 | 0.8571 | 38.262 s | 0.627 cases/s |

At the default threshold, the false negatives were
`network.en.positive`, `network.ja.positive`,
`network.zh-cn.positive`, `network.ko.positive`, `auth.es.positive`,
`auth.ru.positive`, `data.pt-br.positive`, `data.it.positive`, and
`data.hi.positive`. At `0.299`, the false negatives were
`network.ja.positive`, `data.it.positive`, and `data.hi.positive`.

The lower threshold is included to make the recall/threshold tradeoff visible
on this exact fixture. It is not a calibration result and does not change the
default threshold of `0.5`. Scores are relevance scores derived from the
model's `Yes` and `No` logits, not probabilities. The fixture is too small and
too narrowly authored to support a claim about general multilingual quality,
real-world precision, latency, or long-session throughput. In particular,
each record pays process startup and model-load cost, so the throughput above
is not a streaming-search benchmark.

Re-run the commands above after changing the model runtime, prompt, score
calculation, or fixture. The runner returns a nonzero exit status whenever a
case differs from its annotation; that is expected for the reported results
and should trigger review rather than a silent metric change.

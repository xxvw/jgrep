# Semantic evaluation fixture

`semantic-v1.jsonl` is a small, fixed set of manually annotated semantic
matching cases. Each non-empty line is one UTF-8 JSON object that conforms to
`semantic-v1.schema.json`.

It covers three query groups, positive matches, explicitly negated statements,
and lexical false-positive challenges. The input lines cover English, Japanese,
Simplified Chinese, Korean, Spanish, German, Russian, French, Brazilian
Portuguese, Italian, Arabic, and Hindi. Query languages intentionally vary
between English, Japanese, and Spanish so consumers can exercise cross-language
matching as well as cross-language input.

`expected_match` is a human judgment of whether a single input line expresses
the query's meaning. `negated` cases must remain false even when they contain
keywords from the query. `false_positive_challenge` cases contain related terms
or failure language while describing a different event.

## Runner contract

The stable minimum contract for a runner is only these fields:

| Field | Contract |
| --- | --- |
| `id` | Preserve it as the case identifier in any result. |
| `query` | Pass it as the semantic context. |
| `text` | Send this exact UTF-8 text plus one newline to standard input. |
| `expected_match` | Compare it with the actual Boolean result. |

The canonical per-record invocation is:

```text
jgrep --offline --model <MODEL_PATH> -q -- <QUERY>
```

The runner supplies `<TEXT>\n` on standard input. Exit status `0` represents
`true`, `1` represents `false`, and `2` is an evaluation error. A runner may
pass an explicit semantic threshold or other supported semantic flags, but it
must record those choices outside this fixture. The remaining fields are useful
for filtering and stratifying cases and are not required to execute a record.

This fixture is intended for optional model-backed smoke or regression
evaluation. Run each record independently, compare the emitted boolean with
`expected_match`, and retain any per-case failures for review. It defines no
required threshold, aggregate score, latency target, or quality claim. It does
not download a model or include model outputs.

## Portable runner

Build the release binary and fetch the pinned model once from the repository
root:

```sh
cargo build --locked --release
target/release/jgrep --download-model
cargo xtask eval --device cpu --threshold 0.5
```

In PowerShell:

```powershell
cargo build --locked --release
.\target\release\jgrep.exe --download-model
cargo xtask eval --device cpu --threshold 0.5
```

`cargo xtask eval` reads `eval/semantic-v1.jsonl`, runs each case in a separate
`jgrep` process, supplies the exact fixture `text` followed by one newline on
standard input, and invokes `jgrep --offline --device <cpu|auto> --threshold
<value> -q -- <query>`. It prints TP, FP, TN, FN, process errors, precision,
recall, F1, total wall time, throughput, and every mismatched or errored case
identifier. It returns a nonzero status if any case disagrees with the fixture
or fails to execute, so it is suitable for an explicit regression gate after
reviewing the reported metrics.

The default executable is `target/release/jgrep` (`jgrep.exe` on Windows).
Set `JGREP_BIN` to use another already-built binary, or pass `--bin <path>`.
Use `--model <path>` to select an explicit local GGUF model; otherwise `jgrep`
uses its normal user cache. The runner never downloads a model because every
invocation includes `--offline`. `--device cpu` is the default for comparable
portable measurements; `--device auto` permits the binary to use its automatic
device choice. `--fixture <path>` permits a separate JSONL fixture using the
same required fields.

Each fixture record starts a new process, so its wall time includes model load
and process startup for every case. This intentionally isolates inference state
but is not a throughput benchmark for a long-running model session. Results
are local measurements only: report the exact command, model revision and
SHA-256, platform, device, threshold, and cold/warm cache state before making
any public quality or performance claim.

The short sentences were written for this repository as original evaluation
material. They are distributed under the repository's GPL-3.0-or-later license
and contain no copied corpus, downloaded model, or third-party benchmark data.

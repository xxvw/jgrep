# Default model

Semantic mode uses a local GGUF model. The default is the official
Qwen2.5-0.5B-Instruct Q8_0 artifact, selected for a small local footprint
while keeping the instruction-tuned Qwen2.5 model family.

| Property | Value |
| --- | --- |
| Upstream repository | `Qwen/Qwen2.5-0.5B-Instruct-GGUF` |
| Revision | `6dd44a1fb35d11b5d1b28902876ce3cc9e882d0e` |
| File | `qwen2.5-0.5b-instruct-q8_0.gguf` |
| Expected size | 675,710,816 bytes |
| SHA-256 | `ca59ca7f13d0e15a8cfa77bd17e65d24f6844b554a7b6c12e07a5f89ff76844e` |
| Upstream license | Apache-2.0 |

The immutable download address is:

```text
https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/6dd44a1fb35d11b5d1b28902876ce3cc9e882d0e/qwen2.5-0.5b-instruct-q8_0.gguf?download=true
```

## Acquisition and cache

`jgrep` downloads the default model only when a semantic invocation needs it,
or when `--download-model` is requested. With a context, that flag warms the
cache before searching; with no context, it is a download-only command. It
downloads to a temporary `.part`
file, verifies both byte length and SHA-256, then atomically renames the file
into the application cache. An incomplete or mismatched download is never
used.

By default the cache is the OS-specific application cache directory for the
organization/application identifiers `org`, `localjev`, and `jgrep`, followed
by `models/qwen2.5-0.5b-instruct-q8_0.gguf`. Set `JGREP_MODEL_DIR` to name the
model directory directly, or set `JGREP_CACHE_DIR` to name a cache root under
which `models/` is used. Use `--model <PATH>` for a specific GGUF file, or pass
`--offline` to prohibit network access. Concurrent invocations lock the
per-artifact cache entry and publish only a complete verified file.

An explicit `--model` file is never overwritten or downloaded. `jgrep` checks
that it is a non-empty regular file with a GGUF header, but cannot verify it
against the default model's size or checksum because it is user-selected.

The source repository deliberately excludes GGUF files and local caches. A
redistributor that bundles a model must keep the upstream Apache-2.0 terms,
model card, and notices; see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

## Device selection

`--device auto` is the default. It can use Metal on Apple Silicon when the
embedded backend initializes it successfully and falls back to CPU otherwise.
`--device cpu` forces CPU execution. CPU semantic search is the portability
baseline for macOS, Windows x64, and Linux x64.

## What the score means

For each line and context, `jgrep` prepares a fresh binary-decision prompt and
uses the next-token logits for `Yes` and `No`. It requires each label to be
one exact, round-trippable model token during initialization, then computes:

```text
sigmoid(logit(Yes) - logit(No))
```

This is an uncalibrated relevance score. It is not an estimate of a real-world
probability, a safety decision, or a claim of Jev-equivalent behavior. The
default `--threshold 0.5` is a practical starting point and should be assessed
against the user's own corpus.

The prompt is capped at 4,096 tokens, even if the selected model supports a
larger training context. An over-limit prompt is an error; no query or line is
silently shortened.

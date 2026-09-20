# Semantic search behavior

Semantic mode is selected when neither `-E` nor `-F` is present. Each input
line is evaluated independently against one or more natural-language contexts.
Contexts supplied with repeated `-e` options are combined with OR semantics;
`-v` inverts the resulting selection.

The model produces a binary relevance score for every line/context pair. A
line is selected when any score reaches `--threshold` (default `0.5`). `--score`
is an opt-in display aid and does not turn the score into calibrated
confidence. A fresh inference state is used for each evaluation so prior lines
cannot affect a later decision.

With `--score`, a selected ordinary output record starts with a value such as
`score=0.743 ` before the original line. This display value is still the same
uncalibrated relevance score. `--offline` and `--download-model` conflict.

Semantic mode is intentionally conservative about unsupported combinations:

- `-E` and `-F` are mutually exclusive lexical modes.
- `-i` applies to lexical matching only.
- `--threshold` and `--score` apply to semantic matching only.
- A line that cannot fit safely within the model's supported prompt context
  produces an error; this build caps one semantic prompt at 4,096 tokens and
  does not silently shorten it.

The searched content remains local after any optional model acquisition, but
the model can still misunderstand wording, including multilingual text,
negation, quotations, and instructions embedded in a file. Treat results as a
developer-search aid and review consequential matches yourself.

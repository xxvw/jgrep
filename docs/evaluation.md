# Evaluation policy

The versioned v0.1.0 local measurement is published in
[`eval/RESULTS-v0.1.0.md`](../eval/RESULTS-v0.1.0.md). It records the exact
fixture, model revision and hash, platform, device, thresholds, command, and
per-case mismatches. It is a small reference measurement rather than a
general accuracy, latency, or throughput claim.

The repository's versioned
[semantic fixture](../eval/README.md) supplies small, manually annotated
positive and negative examples for model-backed smoke and regression work. A
separate CI inference smoke may validate model acquisition and execution
without making an accuracy assertion. Neither is a benchmark.

Before publishing a quality claim, maintainers should extend or run a fixed
evaluation set containing at least:

- positive and negative examples for each query;
- English, Japanese, Simplified Chinese, Korean, Spanish, German, Russian, and
  other represented languages;
- negation, quoted text, ambiguous phrasing, and closely related non-matches;
- inputs near the model context boundary.

Report precision, recall, F1, fixture version, model file SHA-256, score
threshold, operating system/CPU or Metal setting, cold/warm startup condition,
and measurement command. Keep evaluation records separate from user logs and
never add proprietary search content to the repository.

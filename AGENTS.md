# Repository agent instructions

## Locate code with `jgrep`

When `jgrep` is available, use its compact agent mode as the first search for
code locations. It keeps tool output small and lets you inspect only the
source ranges that matter next.

```sh
# Known identifier or exact text: model-free.
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/

# Conceptual behavior: local semantic matching.
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'where authentication failures are handled' src/
```

`--ai` emits bounded `path:line` records without source text. Parse the final
colon followed by the decimal line number, since Windows paths can contain a
drive-letter colon. A notice that the result cap was reached means the search
is incomplete; narrow the query or scope before increasing the cap. Read only
the returned local ranges before broadening the search.

The reusable, fuller guidance is in
[`templates/AGENTS.jgrep.md`](templates/AGENTS.jgrep.md). Use `rg` when
`jgrep` is not installed or when its compact search cannot express the task.

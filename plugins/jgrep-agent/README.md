# jgrep Agent

`jgrep-agent` is a skills-only Agent Plugin for Codex. It teaches coding
agents to use [`jgrep`](https://github.com/xxvw/jgrep)'s compact
`--ai` mode to find code locations, then inspect only the source ranges that
matter. It adds no network service, background process, or model of its own.

Use the current `jgrep` installer before enabling the plugin so that `jgrep`
is available on the coding environment's `PATH`. See the
[installation and agent integration guide](https://github.com/xxvw/jgrep/blob/main/docs/installation-and-agents.md).

## Install in Codex

This command registers the repository marketplace, installs the plugin, and
does not require cloning the repository:

```sh
codex plugin marketplace add xxvw/jgrep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@jgrep
```

Start a new Codex session after installation so the skill is available. In the
ChatGPT desktop app, select the `jgrep` marketplace in the Plugins
Directory and install `jgrep-agent`.

## What the skill changes

For a conceptual search, it prefers semantic `jgrep --ai`:

```sh
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'where authentication failures are handled' src/
```

For a known literal or pattern, it prefers `-F` or `-E`. Those lexical modes
do not download or load the local model:

```sh
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/
jgrep --ai --ai-max-results 25 -r -E 'Auth(Error|Failure)' src/
```

The output is one locator per selected line:

```text
src/auth/session.rs:42
-:7
```

Parse the final colon before the decimal line number, since Windows paths can
contain a drive-letter colon. The default cap is 50 locations. If `jgrep`
reports that the cap was reached on stderr, the result is incomplete; narrow
the search before raising `--ai-max-results`. The skill treats exit status `1`
as a normal no-result outcome and falls back to `rg` when `jgrep` is not
available.

`--ai` is deliberately incompatible with output modes that hide or expand
locations, including `-c`, `-l`, `-L`, `-q`, `-m`, context flags, `--score`,
and `--color=always`. The full rules are in
[`skills/jgrep-agent/SKILL.md`](skills/jgrep-agent/SKILL.md).

## Read this plugin in your language

| Language | README |
| --- | --- |
| English | [README.md](README.md) |
| 日本語 | [README.ja.md](README.ja.md) |
| 简体中文 | [README.zh-CN.md](README.zh-CN.md) |
| 한국어 | [README.ko.md](README.ko.md) |
| Español | [README.es.md](README.es.md) |
| Deutsch | [README.de.md](README.de.md) |
| Русский | [README.ru.md](README.ru.md) |
| Français | [README.fr.md](README.fr.md) |
| Português (Brasil) | [README.pt-BR.md](README.pt-BR.md) |
| Italiano | [README.it.md](README.it.md) |
| العربية | [README.ar.md](README.ar.md) |
| हिन्दी | [README.hi.md](README.hi.md) |

## License

This plugin is part of jgrep and is licensed under
[GPL-3.0-or-later](https://github.com/xxvw/jgrep/blob/main/LICENSE).

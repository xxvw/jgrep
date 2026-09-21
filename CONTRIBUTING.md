# Contributing to jgrep

Thanks for helping improve `jgrep`, the local semantic-search CLI in this
repository. By contributing, you agree to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Getting started

1. Fork the repository and create a focused branch.
2. Make a small, reviewable change with tests or documentation that demonstrate
   the intended behavior.
3. Run the project's documented local checks before opening a pull request.
4. Explain the user-facing behavior, platforms tested, and any limitations in
   the pull request template.

`jgrep` aims to work consistently on macOS, Windows, and Linux. Keep command
output suitable for pipelines: selected input lines belong on standard output;
diagnostics belong on standard error. Preserve grep-compatible exit behavior
where the project documents it, and do not make model downloads or network
access implicit in unrelated commands.

## Changes and tests

- Keep Rust code formatted and free of new compiler or linter warnings.
- Add focused tests for changed command-line behavior, including error and exit
  cases when relevant.
- Test both file input and standard input when changing search or output code.
- Avoid committing model files, caches, generated binaries, credentials, or
  personal data.
- Update the English README when public behavior changes. Translations should
  retain the same technical meaning; an incomplete translation is preferable
  to a misleading one.

## Pull requests

Use a descriptive title and keep unrelated refactors separate. Maintainers may
ask for changes to keep the project portable, secure, or compatible with its
documented CLI behavior. Please disclose security-sensitive issues privately as
described in [SECURITY.md](SECURITY.md), rather than in a public issue or pull
request.

## License

Contributions are submitted under the repository's GPL-3.0-or-later license,
unless the maintainers explicitly agree otherwise in writing.

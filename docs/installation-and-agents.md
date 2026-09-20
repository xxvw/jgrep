# Installation and coding-agent integration

`localjev-grep` publishes native, versioned release archives for macOS Apple
Silicon, macOS Intel, Windows x64, and Linux x64. The installers in this
repository select the matching archive, verify its SHA-256 manifest before
using its executable, require its `jgrep --version` output to match the
selected tag, and install only that executable into a per-user directory. They
do not download the Qwen model; semantic mode retrieves the pinned model only
when it is first needed.

Release archives are checksummed but the initial `v0.1.0` release is neither
signed nor notarized. A checksum downloaded from the same release protects
against transfer corruption and accidental asset mismatch; it is not an
independent publisher-identity proof. Prefer a reviewed checkout or a pinned,
reviewed source URL for the installer itself. Do not pipe a downloaded script
straight into a shell.

## macOS and Linux

From a reviewed clone, run:

```sh
git clone https://github.com/xxvw/localjev-grep.git
cd localjev-grep
./scripts/install.sh
```

With no version option, the script resolves the latest published GitHub
Release. Pin a release for reproducible installations:

```sh
./scripts/install.sh --version v0.1.0
```

The script selects one of these native release targets:

| Host | Target | Archive |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `.tar.gz` |
| Linux x64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |

It installs `jgrep` into `$XDG_BIN_HOME` when that variable is set, otherwise
`$HOME/.local/bin`. The installer does not edit shell startup files. If that
directory is not already on `PATH`, add it in your shell configuration:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Pass `--install-dir <directory>` (or the `JGREP_INSTALL_DIR` environment
variable) to choose another directory. The installer refuses to replace an
existing executable unless `--force` is supplied. `--bin-dir` is an alias for
`--install-dir`.

For a local release build, air-gapped validation, or an internal mirror that
preserves release filenames and checksum files, use `--asset-dir`. This makes
no HTTP request and requires an explicit version:

```sh
./scripts/install.sh --asset-dir ./dist --version v0.1.0
```

The asset directory must contain the target-qualified `.tar.gz` archive and
either its `.tar.gz.sha256` file or `SHA256SUMS`. `JGREP_VERSION`,
`JGREP_ASSET_DIR`, `JGREP_TARGET`, and `JGREP_REPOSITORY` provide corresponding
environment overrides. `--target` is intended for controlled build or test
environments; it does not make an incompatible binary executable on another
CPU or operating system.

The Linux x64 archive requires glibc 2.35 or newer. Build from source on an
older Linux system or an unsupported architecture. macOS releases are not
notarized; follow your organization’s normal review and platform policy rather
than disabling platform security controls in an installer.

## Windows PowerShell

From a reviewed clone in PowerShell:

```powershell
git clone https://github.com/xxvw/localjev-grep.git
Set-Location localjev-grep
.\scripts\install.ps1
```

The default resolves the latest published release and installs `jgrep.exe` to
`$env:LOCALAPPDATA\Programs\jgrep\bin`. Pin an installation or install from
already-produced assets with:

```powershell
.\scripts\install.ps1 -Version v0.1.0
.\scripts\install.ps1 -AssetDirectory .\dist -Version v0.1.0
```

The PowerShell installer supports the `x86_64-pc-windows-msvc` release only;
it rejects unsupported Windows architectures. Use `-InstallDir` or
`JGREP_INSTALL_DIR` to change the destination. It refuses to replace an
existing `jgrep.exe` unless `-Force` is supplied. Add the selected directory
to the current and future user `PATH` with the opt-in `-AddToPath` switch:

```powershell
.\scripts\install.ps1 -Version v0.1.0 -AddToPath
```

The Windows release uses the standard Microsoft Visual C++ runtime. Install
the current Visual C++ Redistributable when that runtime is not already
present. The script does not download or alter system runtimes.

For CI-style offline verification, `-AssetDirectory` requires the versioned
`.zip` archive and its `.zip.sha256` file (or a `SHA256SUMS` file) from the
same release. It uses the same checksum and safe single-entry extraction path
as an online install. `JGREP_VERSION`, `JGREP_ASSET_DIR`, `JGREP_REPOSITORY`,
and `JGREP_TARGET` can also be set as environment variables.

## Verify the installation and warm the model

Both installers run `jgrep --version` after placing the executable. Check it
again from a new shell after updating `PATH`:

```sh
jgrep --version
jgrep --download-model
jgrep --offline 'network connection failure' app.log
```

`--download-model` obtains the pinned Qwen GGUF file without searching input.
For a machine that must not use the network, transfer a validated model and
use `--model <path> --offline`; see [the model guide](model.md). Fixed-string
and regex searches (`-F` and `-E`) do not need a model.

## Tell coding agents to use compact search

The reusable agent-instruction template is
[`templates/AGENTS.jgrep.md`](../templates/AGENTS.jgrep.md). Copy its guidance
into the repository-level instruction file understood by the coding agent
(for example `AGENTS.md` for Codex-compatible agents or `CLAUDE.md` for Claude
Code), merging it with the repository’s existing instructions rather than
replacing them.

The template teaches agents to use:

```sh
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'where authorization errors are handled' src/
```

`--ai` prints bounded `INPUT_LABEL:LINE` locators rather than source text. It
reduces tool-output tokens by letting an agent fetch only cited line ranges.
Agents must parse the final `:decimal-line-number` suffix, because a Windows
path can contain an earlier colon. They should treat a cap notice on stderr as
an incomplete result and narrow the next query. The usual grep statuses still
apply: `0` means locations were found, `1` is a normal no-location result, and
`2` identifies an invocation error. `-:LINE` results are best used only
with reproducible pipelines.

Use semantic `--ai` searches for concepts and behavior. Combine `--ai` with
`-F` or `-E` for known symbols or patterns when model-free lexical matching is
more precise. Normal formatting, count, file-list, quiet, score, and context
flags are deliberately incompatible with the compact agent format; the
template lists the exact restrictions.

## Script validation

Run the repository-provided static validation after changing installer or
template artifacts:

```sh
./scripts/validate-installers.sh
```

It always checks Bash syntax and the shared artifact contract, uses ShellCheck
when available, and parses the PowerShell script when `pwsh` is available.
The offline `--asset-dir` mode lets release jobs or maintainers exercise the
full download-free installation path against a freshly produced archive before
publication.

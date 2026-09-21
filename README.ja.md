# localjev-grep

`jgrep` は、自然言語の文脈に意味が合う入力行を表示する、ローカル実行の
grep 風コマンドです。Python、Ollama、常駐サーバーは必要ありません。既存の
`grep` と同じようにファイルとパイプを扱えます。

Jev の「文章を生成するのではなく、意味に対する判定を返す」という考え方を
参考にしています。ただし本プロジェクトは Jev、TypeSafe、Qwen、Hugging Face、
llama.cpp とは提携・承認・配布関係にありません。完全な仕様は英語版の
[README.md](README.md) を参照してください。

**言語:** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## インストール

`git clone` を使わず、次の一つのコマンドブロックを一度貼り付けるだけで
インストールできます。バージョン固定のインストーラーバンドルをダウンロードし、
展開前に SHA-256 を検証してからローカルの日本語ラッパーを実行します。macOS
（Apple Silicon / Intel）、Windows x64、Linux x64（glibc 2.35 以降）に対応します。
ダウンロード元と同じリリースから得るチェックサムは転送破損や取り違えを検出しますが、
署名による発行元の証明ではありません。
日本語ラッパーはローカライズされた開始メッセージを表示し、共有の検証済みコア
インストーラーへ処理とすべての安全確認を委譲します。

Bash / zsh に一度貼り付ける場合:

```sh
(
  set -e
  version=v0.1.1
  archive="localjev-grep-installers-${version}.tar.gz"
  workdir="$(mktemp -d)"
  trap 'rm -rf "$workdir"' EXIT
  base="https://github.com/xxvw/localjev-grep/releases/download/${version}"
  curl --fail --silent --show-error --location --proto '=https' \
    --proto-redir '=https' -o "$workdir/$archive" "$base/$archive"
  curl --fail --silent --show-error --location --proto '=https' \
    --proto-redir '=https' -o "$workdir/$archive.sha256" "$base/$archive.sha256"
  (cd "$workdir" && if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "$archive.sha256"
  else
    sha256sum -c "$archive.sha256"
  fi)
  tar -xzf "$workdir/$archive" -C "$workdir"
  bash "$workdir/localjev-grep-installers-${version}/installers/ja/install.sh" \
    --version "$version"
)
```

Windows PowerShell に一度貼り付ける場合:

```powershell
& {
  $ErrorActionPreference = 'Stop'
  $version = 'v0.1.1'
  $archive = "localjev-grep-installers-$version.zip"
  $workdir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
  New-Item -ItemType Directory -Path $workdir | Out-Null
  try {
    $base = "https://github.com/xxvw/localjev-grep/releases/download/$version"
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile (Join-Path $workdir $archive)
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile (Join-Path $workdir "$archive.sha256")
    $manifest = (Get-Content -LiteralPath (Join-Path $workdir "$archive.sha256") -Raw).Trim()
    $manifestPattern = '^[A-Fa-f0-9]{64}  ' + [regex]::Escape($archive) + '$'
    if ($manifest -notmatch $manifestPattern) { throw 'installer bundle checksum manifest is invalid' }
    $expected = $manifest.Substring(0, 64).ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath (Join-Path $workdir $archive) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw 'installer bundle checksum mismatch' }
    Expand-Archive -LiteralPath (Join-Path $workdir $archive) -DestinationPath $workdir -Force
    & (Join-Path $workdir "localjev-grep-installers-$version\installers\ja\install.ps1") -Version $version
  } finally {
    Remove-Item -LiteralPath $workdir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
```

この手順は `curl | sh` や `Invoke-Expression` を使いません。インストール先などの
オプションとオフライン用アセットの指定は、[インストールとコーディングエージェントの
ガイド](docs/installation-and-agents.md) を参照してください。コーディングエージェント用の
設定は [日本語の `jgrep-agent` プラグインガイド](plugins/jgrep-agent/README.ja.md) にあります。
Unix の既定のインストール先は `$HOME/.local/bin` です。PowerShell では最後の
`-Version $version` を `-Version $version -AddToPath` にすると、選択したインストール先を
ユーザーの `PATH` に追加できます。

### ソースからビルドする場合（任意）

ソースからビルドする場合は、固定された Rust ツールチェーン、CMake、C++
コンパイラが必要です。未対応アーキテクチャや glibc 2.35 未満の Linux では、
ソースビルドを選べます。

```sh
git clone https://github.com/xxvw/localjev-grep.git
cd localjev-grep
cargo build --release
./target/release/jgrep --help
```

Windows PowerShell:

```powershell
git clone https://github.com/xxvw/localjev-grep.git
Set-Location localjev-grep
cargo build --release
.\target\release\jgrep.exe --help
```

## 最初のモデル取得と使用例

意味検索では公式の Qwen2.5-0.5B-Instruct GGUF Q8_0（約 676 MB）を使います。
最初の検索時に自動取得されますが、先にキャッシュを準備するには次を実行します。

```sh
jgrep --download-model
```

取得物はユーザーごとのアプリケーションキャッシュへ保存され、ダウンロード中は
ハッシュ検証後に原子的に配置されます。モデルやキャッシュはリポジトリおよび
ソース配布物に含まれません。

Bash / zsh:

```sh
jgrep 'network connection failure' app.log
cat app.log | jgrep 'network connection failure'
jgrep -n -r --include '*.log' '認証に失敗している' logs/
jgrep -E -i 'error|warning' app.log
jgrep -F 'connection refused' app.log
```

Windows PowerShell:

```powershell
.\jgrep.exe 'network connection failure' .\app.log
Get-Content .\app.log | .\jgrep.exe 'network connection failure'
.\jgrep.exe -n -r --include '*.log' '認証に失敗している' .\logs
.\jgrep.exe -E -i 'error|warning' .\app.log
```

ファイルを省略するか `-` を指定すると標準入力を読みます。複数の `-e` は OR
条件です。通常は選択された元の行だけを標準出力へ書き、診断とダウンロード進捗は
標準エラーへ書きます。

## オフライン利用

すでにキャッシュ済みの既定モデル、または明示的に指定したローカル GGUF を
ネットワークに接続せず使用できます。

```sh
jgrep --offline 'network connection failure' app.log
jgrep --model /path/to/model.gguf --offline 'network connection failure' app.log
```

`--offline` は有効なローカルモデルがなければ失敗し、取得は行いません。
`--download-model` とは併用できません。

## AI エージェント向けのコンパクト出力

`--ai` は、コーディングエージェントのツール呼び出しで使うために、一致位置だけを
`path:line` 形式で出力します。一致した元の行、ANSI 色、スコア、前後コンテキストは
出力しません。既定では呼び出し全体で最大 50 件に制限されます。必要に応じて
`--ai-max-results <NUM>` で上限を変更してください。エージェントは位置を受け取った後、
必要な狭い行範囲だけを別の読み取り操作で取得できます。

## 検索モードと主なオプション

標準の意味検索では、モデルが各行と文脈の関連性を Yes/No として判定します。
`-E` は Rust `regex` の正規表現、`-F` は固定文字列です。この二つのモードでは
モデルを取得もロードもせず、`-i` も利用できます。`-E` と `-F` は同時に使えず、
GNU BRE、後方参照、PCRE との完全互換はありません。

| オプション | 内容 |
| --- | --- |
| `-e <context>` | 文脈を追加する。どれかが一致すれば選択する。 |
| `-n`、`-H`、`-h` | 行番号、ファイル名の常時表示、ファイル名の抑制。 |
| `-c`、`-l`、`-L`、`-q`、`-m` | 件数、ファイル名、静かな早期終了、入力ごとの最大件数。 |
| `-v` | 最終選択を反転する。 |
| `-r`、`--include`、`--exclude` | パス順の再帰検索と対象パスの絞り込み。 |
| `-A`、`-B`、`-C` | 後方、前方、前後のコンテキスト行。 |
| `--color`、`--line-buffered` | 色表示とパイプ向けの行単位フラッシュ。 |
| `--ai`、`--ai-max-results <NUM>` | エージェント向けの `path:line` 出力と、呼び出し全体の最大結果数（既定 50）。 |
| `--threshold`、`--score` | 意味検索のしきい値（既定 0.5）とスコア表示。 |
| `--model`、`--offline`、`--device auto\|cpu` | ローカルモデル、ネットワーク禁止、推論デバイス。 |

`--` は `-` で始まる文脈やパスの前に使います。意味検索専用のオプションを
`-E` / `-F` に組み合わせるとエラーになります。

## 制約と互換性

意味スコアは `Yes` と `No` の次トークン logit 差から得る順位付けの値であり、
校正済みの確率でも正確性の保証でもありません。曖昧な表現、否定、多言語、長い行、
入力中の敵対的な内容は結果に影響します。安全、法務、医療、セキュリティに関わる
判断をこれだけに委ねないでください。精度、起動時間、処理速度については公的な
保証をしていません。

`jgrep` は UTF-8、LF / CRLF、Unicode パスに対応し、行単位で処理して入力順を
維持します。再帰検索ではディレクトリ symlink を追跡せず、検出したバイナリは
診断してスキップします。明示指定の非テキスト入力はエラーです。意味プロンプトの
4,096 トークン上限を超える行は黙って切り詰めません。終了コードは一致あり `0`、
一致なし `1`、エラー `2` です。Apple Silicon では `--device auto` が利用可能な
Metal を使えますが、すべての対応 OS で CPU 実行を利用できます。

## 開発とライセンス

共通のローカル検証は次で実行します。

```sh
cargo xtask ci
```

プロジェクトのソースコードは **GPL-3.0-or-later** です。
[LICENSE](LICENSE)、[NOTICE](NOTICE)、[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
を確認してください。既定の Qwen モデルは別途ダウンロードされる Apache-2.0 の
資産で、プロジェクトのソースコードとして GPL に再許諾されるものではありません。

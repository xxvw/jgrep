# Codex 用 jgrep-agent

`jgrep-agent` は、Codex に `jgrep --ai` を最初のコード検索として使わせる
ローカルプラグインです。候補の場所だけを返すため、検索結果のツール出力を小さく
保てます。

## インストール

最初に、実行するエージェントの `PATH` から `jgrep` を呼び出せるようにします。
対応 OS、1 コマンドインストール、手動・オフライン導入は、プロジェクトの
[README](https://github.com/xxvw/localjev-grep/blob/main/README.md) と
[インストールおよびエージェント統合ガイド](https://github.com/xxvw/localjev-grep/blob/main/docs/installation-and-agents.md)
を参照してください。

次に、Codex へマーケットプレイスとプラグインを追加し、新しい Codex セッションを
開始します。

```sh
codex plugin marketplace add xxvw/localjev-grep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@localjev-grep
```

プラグインを有効にしたプロセスにも `jgrep` のインストール先が `PATH` として渡る
必要があります。

## 使い方

概念や振る舞いを探すときは、意味検索を使います。

```sh
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  '認証失敗を処理する場所' src/
```

既知の識別子や固定テキストには、モデルを使わない `-F` を使います。正規表現が
必要な場合は `-E` を使います。

```sh
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/
jgrep --ai --ai-max-results 25 -r -E 'ERROR|WARN' src/
```

出力はソース本文を含まない `path:line` 形式です。Windows のドライブ文字にも
コロンが含まれるため、**末尾の `:LINE`**（10 進の行番号）を解析してください。
件数上限に達したという通知が出た場合、結果は完全ではありません。対象を狭めて
から再検索し、必要なときだけ上限を増やします。返された近い行だけを読みます。

`jgrep` が未導入、またはこの検索形式で表せない場合は、`rg` を代替として使います。

```sh
rg -n -F 'validate_session' src/
```

# 面向 Codex 的 jgrep-agent

`jgrep-agent` 是一个本地 Codex 插件，它让 Codex 优先使用 `jgrep --ai` 查找代码。
该模式只返回候选位置，因此可以减少搜索工具的输出量。

## 安装

先安装 `jgrep`，并确保运行代理的进程可以通过 `PATH` 调用它。支持的平台、单命令
安装、手动安装和离线安装请参阅项目的
[README](https://github.com/xxvw/jgrep/blob/main/README.md) 和
[安装与代理集成指南](https://github.com/xxvw/jgrep/blob/main/docs/installation-and-agents.md)。

然后将市场和插件加入 Codex，并启动新的 Codex 会话：

```sh
codex plugin marketplace add xxvw/jgrep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@jgrep
```

启用插件的进程也必须在其 `PATH` 中包含 `jgrep` 的安装目录。

## 使用方法

查找概念或行为时，使用语义搜索：

```sh
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  '处理身份验证失败的位置' src/
```

对于已知的标识符或固定文本，使用不加载模型的 `-F`。需要正则表达式时使用 `-E`：

```sh
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/
jgrep --ai --ai-max-results 25 -r -E 'ERROR|WARN' src/
```

输出为不含源代码正文的 `path:line` 记录。Windows 驱动器号本身带有冒号，因此请解析
**最后一个 `:LINE`**（十进制行号）后缀。若出现达到结果上限的提示，结果并不完整；
请先缩小路径或条件，再仅在必要时提高上限。随后只读取返回位置附近的代码。

如果尚未安装 `jgrep`，或当前搜索不能用这种形式表达，请改用 `rg`：

```sh
rg -n -F 'validate_session' src/
```

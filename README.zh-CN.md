# localjev-grep

`jgrep` 是一个本地运行的、用于语义搜索的 grep 风格命令。它根据自然语言上下文筛选输入行，同时保留 `grep` 熟悉的文件与管道工作方式。

> **v0.1.0：** 可从 [GitHub Releases](https://github.com/xxvw/localjev-grep/releases) 下载带版本号的原生归档包。本项目不对基准测试结果、准确率、吞吐量或延迟作出保证。

默认的语义模式使用本地的 [Qwen2.5-0.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct) 模型，对每一行是否与上下文相关作二元判断。搜索内容不会发送给托管模型；不需要 Python、Ollama 或常驻服务。`-E` 为 Rust 正则表达式模式，`-F` 为固定字符串模式，两者均不会下载或加载模型。

本项目独立开发，与 Jev、TypeSafe、Qwen、Hugging Face 或 llama.cpp 没有隶属、背书或发行关系。Jev 仅为二元判断交互方式的灵感来源。

## 安装

从 [GitHub Releases](https://github.com/xxvw/localjev-grep/releases) 下载适用于 macOS（Apple Silicon 或 Intel）、Windows x64 或 Linux x64（glibc 2.35 及更高版本）的归档包，解压后将其中的可执行文件加入 `PATH`。

也可以从源码构建：

```sh
git clone https://github.com/xxvw/localjev-grep.git
cd localjev-grep
cargo build --release
./target/release/jgrep --help
```

Windows PowerShell：

```powershell
git clone https://github.com/xxvw/localjev-grep.git
Set-Location localjev-grep
cargo build --release
.\target\release\jgrep.exe --help
```

源码构建需要 `rust-toolchain.toml` 固定的 Rust 工具链、CMake，以及可构建内嵌 llama.cpp 依赖的 C++ 编译器。所有支持的平台均可使用 CPU；Apple Silicon 上的自动设备设置可使用 Metal。

## 使用

常规参数顺序为：

```text
jgrep [OPTIONS] <context> [FILE ...]
```

没有文件参数或文件参数为 `-` 时，`jgrep` 从标准输入读取。`-e` 可代替位置参数中的 `<context>`；多个 `-e` 上下文之间为 OR 关系。

### Bash

```sh
# 预先下载默认模型；不带上下文时下载后退出
jgrep --download-model

# 默认的本地语义搜索
jgrep '数据库登录被拒绝' service.log

# 管道、递归搜索和上下文行
journalctl -f | jgrep --line-buffered '连接被重置'
jgrep -n -r --include '*.rs' '处理文件系统错误' src/
jgrep -C 2 '请求超时' app.log

# 不联网地搜索；本地没有有效模型时失败
jgrep --offline '身份验证失败' service.log

# 常规文本匹配，不初始化模型
jgrep -E -i 'error|warning' app.log
jgrep -F 'connection refused' app.log
```

### PowerShell

```powershell
# 解压发行版归档后在该目录运行
.\jgrep.exe --download-model
Get-Content .\service.log | .\jgrep.exe '数据库登录被拒绝'
.\jgrep.exe --offline '身份验证失败' .\service.log
.\jgrep.exe -E -i 'error|warning' .\app.log
```

## 常用选项

| 选项 | 作用 |
| --- | --- |
| `-e <context>` | 添加上下文；任一上下文匹配即可选中该行。 |
| `-E` / `-F` | 使用 Rust 正则表达式 / 固定字符串模式；二者不能同时使用。 |
| `-i` | 忽略大小写；仅适用于 `-E` 和 `-F`。 |
| `-n`、`-H` / `-h`、`-c` | 显示行号、强制 / 隐藏文件名、按输入打印选中行计数。 |
| `-l` / `-L`、`-q`、`-m <NUM>`、`-v` | 输出有 / 无匹配的文件名、静默并在首个匹配停止、限制选中行数、反转最终选择。 |
| `-r`、`-A` / `-B` / `-C` | 递归搜索，以及显示后方 / 前方 / 周围的上下文行。 |
| `--include <GLOB>` / `--exclude <GLOB>` | 限制或跳过递归搜索的路径。 |
| `--color <auto|always|never>`、`--line-buffered` | 控制 ANSI 高亮；为流式管道逐行刷新输出。 |
| `--threshold <0..1>`、`--score` | 设置语义相关性阈值（默认 `0.5`）；在输出中显示语义分数。 |
| `--model <PATH>`、`--download-model`、`--offline`、`--device <auto|cpu>` | 指定本地 GGUF 模型、下载默认模型、禁止网络访问、选择本地推理设备。 |

语义专用选项不能用于词法模式；`--offline` 与 `--download-model` 不能同时使用。以 `-` 开头的模式或路径前请使用 `--`。

## 模型、隐私与限制

语义模式使用官方 Qwen2.5-0.5B-Instruct GGUF **Q8_0** 文件（约 676 MB）。程序固定模型版本与 SHA-256，在首次需要语义推理时或执行 `--download-model` 时下载；验证后以原子方式写入每位用户的应用缓存，并在之后复用。使用 `--model /path/to/model.gguf` 可以改用已有的本地 GGUF 文件。

可在下载模型后完全离线搜索。`--help`、词法模式、空输入和 `-m 0` 不会初始化模型。语义分数由模型的 `Yes` 与 `No` 下一个 token 的 logits 差计算：

```text
sigmoid(logit(Yes) - logit(No))
```

该分数是相关性分数，不是经过校准的概率，也不保证正确。歧义表述、否定、语言、长行和搜索文件中的对抗性内容都会影响结果；语义提示的上限为 4,096 个 token，过长的行不会被静默截断。请勿将它作为人身安全、法律、医疗或信息安全关键决策的唯一依据。项目不承诺与 `grep` 相同的速度，也不提供已测量的准确率、吞吐量或延迟保证。

`jgrep` 增量读取输入、按输入顺序输出选中行，并支持 UTF-8、LF / CRLF 和 Unicode 路径。递归搜索不跟随目录符号链接，递归遇到的二进制文件会跳过并输出诊断；明确指定的非文本输入会报错。退出码与 `grep` 一致：找到选中项为 `0`，未找到为 `1`，错误为 `2`。

## 许可证

项目源代码采用 **GPL-3.0-or-later** 许可。重新分发构建前，请阅读 [LICENSE](LICENSE)、[NOTICE](NOTICE) 和 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。默认 Qwen 模型是单独下载的 Apache-2.0 资产，并不作为项目源代码许可。

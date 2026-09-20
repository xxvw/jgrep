# localjev-grep

`jgrep` é um comando local de busca semântica no estilo `grep`. Ele imprime
linhas cujo significado corresponde a uma consulta em linguagem natural e
preserva o fluxo habitual de arquivos e pipelines. Não requer Python, Ollama
nem um serviço em segundo plano.

A interação foi inspirada na ideia de Jev de retornar uma decisão binária de
relevância em vez de gerar texto. O projeto não é afiliado, endossado ou uma
distribuição de Jev, TypeSafe, Qwen, Hugging Face ou llama.cpp. A documentação
canônica é o [README.md](README.md) em inglês.

**Idiomas:** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## Instalação

Arquivos nativos para macOS (Apple Silicon e Intel), Windows x64 e Linux x64
(glibc 2.35 ou mais recente) estão em
[GitHub Releases](https://github.com/xxvw/localjev-grep/releases). Extraia o
arquivo e coloque `jgrep` ou `jgrep.exe` no `PATH`.

Para compilar o código-fonte, são necessários a ferramenta Rust definida em
`rust-toolchain.toml`, CMake e um compilador C++ para o llama.cpp incorporado:

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

No Apple Silicon, `--device auto` pode usar Metal. A execução por CPU está
disponível em todas as plataformas compatíveis, e os binários de distribuição
não usam instruções de CPU específicas da máquina que os compilou.

## Primeiro modelo e exemplos

A busca semântica usa o GGUF Q8_0 oficial do Qwen2.5-0.5B-Instruct (cerca de
676 MB). Ele é baixado no primeiro uso semântico; para preparar o cache antes:

```sh
jgrep --download-model
```

A revisão e o SHA-256 do artefato são fixados no código. O arquivo é verificado,
instalado atomicamente no cache de aplicativo do usuário e reutilizado. Modelo
e cache não entram no Git nem nos arquivos de fonte.

Bash / zsh:

```sh
jgrep 'network connection failure' app.log
cat app.log | jgrep 'network connection failure'
jgrep -n -r --include '*.log' 'authentication failed' logs/
jgrep -E -i 'error|warning' app.log
jgrep -F 'connection refused' app.log
```

Windows PowerShell:

```powershell
.\jgrep.exe 'network connection failure' .\app.log
Get-Content .\app.log | .\jgrep.exe 'network connection failure'
.\jgrep.exe -n -r --include '*.log' 'authentication failed' .\logs
.\jgrep.exe -E -i 'error|warning' .\app.log
```

Sem arquivos, ou com `-`, `jgrep` lê stdin. Vários `-e` são combinados por
OU. As linhas originais selecionadas vão para stdout; diagnósticos e progresso
de download vão para stderr.

## Uso offline e opções

Um modelo padrão já armazenado em cache, ou um GGUF local explícito, pode ser
usado sem acessar a rede:

```sh
jgrep --offline 'network connection failure' app.log
jgrep --model /path/to/model.gguf --offline 'network connection failure' app.log
```

`--offline` nunca usa a rede e falha sem um modelo local válido. Ele não pode
ser combinado com `--download-model`. Com contexto, `--download-model`
prepara o cache e então busca; sem contexto, apenas baixa o modelo.

No modo padrão, o modelo local avalia cada linha usando os tokens `Yes` e
`No`. `-E` usa expressões regulares Rust e `-F` busca uma substring literal;
esses modos léxicos não baixam nem carregam o modelo. `-E` e `-F` conflitam,
e `-i` só funciona neles. GNU BRE, PCRE e retroreferências não têm
compatibilidade completa.

| Opção | Finalidade |
| --- | --- |
| `-e <contexto>` | Adiciona um contexto; qualquer correspondência basta. |
| `-n`, `-H`, `-h` | Números de linha e exibir ou suprimir nomes de arquivo. |
| `-c`, `-l`, `-L`, `-q`, `-m` | Contagem, nomes, saída silenciosa e limite por entrada. |
| `-v`, `-r`, `-A/-B/-C` | Inversão, pesquisa recursiva e linhas de contexto. |
| `--include`, `--exclude`, `--color`, `--line-buffered` | Filtros de caminho, cor e flush por linha. |
| `--threshold`, `--score` | Limiar semântico (padrão `0.5`) e exibição da pontuação. |
| `--ai`, `--ai-max-results <N>` | Saída compacta de locais para agentes de programação e seu limite. |
| `--model`, `--download-model`, `--offline`, `--device auto\|cpu` | Modelo, download, rede e dispositivo. |

Use `--` antes de um contexto ou caminho que comece com `-`. Opções
exclusivas da semântica, como `--threshold` e `--score`, são rejeitadas nos
modos léxicos. `-m 0`, entrada vazia, ajuda e buscas léxicas não inicializam
o modelo.

`--ai` emite apenas locais compactos no formato `caminho:linha`: sem texto
fonte correspondente, cor ANSI, pontuação ou linhas de contexto. O padrão é
no máximo 50 locais em toda a execução; ajuste o limite com
`--ai-max-results`. Em seguida, o agente de programação pode buscar somente
os intervalos estreitos de linhas necessários, reduzindo o uso de tokens em
chamadas de ferramentas.

## Limites e compatibilidade

A pontuação é `sigmoid(logit(Yes) - logit(No))`. Ela indica relevância, não
uma probabilidade calibrada nem uma garantia de correção. Ambiguidade, negação,
idioma, linhas longas e conteúdo adversarial podem alterar resultados. Não há
garantias de precisão, desempenho ou latência; não use a saída como única base
para decisões de segurança, jurídicas, médicas ou de segurança da informação.

`jgrep` processa dados incrementalmente e mantém a ordem de entrada. Suporta
UTF-8, finais de linha LF/CRLF e caminhos Unicode. Linhas acima do limite
semântico de 4.096 tokens não são truncadas silenciosamente. A busca recursiva
não segue links simbólicos de diretórios e ignora binários detectados com um
diagnóstico; um arquivo não textual indicado explicitamente é um erro. Códigos
de saída: `0` encontrou seleção, `1` não encontrou e `2` indica erro.
Pipelines de saída rompidos são tratados silenciosamente quando possível.

## Desenvolvimento e licença

A verificação local comum é:

```sh
cargo xtask ci
```

O código-fonte é **GPL-3.0-or-later**. Veja [LICENSE](LICENSE),
[NOTICE](NOTICE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
[SECURITY.md](SECURITY.md) e [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
O modelo Qwen padrão é um artefato Apache-2.0 baixado separadamente, e não
código-fonte do projeto sob GPL.

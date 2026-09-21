# Plugin Codex `jgrep-agent`

Este plugin ensina o Codex a usar `jgrep` como a primeira busca por locais no
código. Ele prioriza a saída compacta para que o agente leia apenas os trechos
de código necessários. Instale também o executável `jgrep`: o plugin não o
inclui.

## Instalação

Em um terminal que tenha o CLI do Codex, adicione o marketplace do projeto e
instale o plugin:

```sh
codex plugin marketplace add xxvw/localjev-grep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@localjev-grep
```

Inicie uma nova sessão do Codex para carregar o plugin. Instale o `jgrep`
seguindo o [README principal](https://github.com/xxvw/localjev-grep/blob/main/README.md) e o
[guia de instalação e integração de agentes](https://github.com/xxvw/localjev-grep/blob/main/docs/installation-and-agents.md).

## Busca compacta para agentes

Comece com `--ai`. Ele mostra apenas locais no formato `caminho:linha`, sem o
texto-fonte, e limita a saída a 50 resultados por padrão:

```sh
# Busca semântica de um comportamento
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'onde as falhas de autenticação são tratadas' src/

# Identificador ou texto conhecido: correspondência literal sem modelo
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/

# Padrão conhecido: expressão regular Rust sem modelo
jgrep --ai --ai-max-results 25 -r -E 'validate_(session|token)' src/
```

Sempre interprete o sufixo final que corresponde a `:[0-9]+$` em cada resultado. Um
caminho do Windows pode conter dois-pontos antes dele, por exemplo
`C:\work\src\auth.rs:57`. Leia apenas os intervalos curtos apontados antes de
ampliar a busca.

Se stderr avisar que o limite foi atingido, o resultado está incompleto:
restrinja a consulta ou o diretório antes de aumentar `--ai-max-results`.
Use `rg` se `jgrep` não estiver instalado ou se a busca compacta não puder
expressar a tarefa.

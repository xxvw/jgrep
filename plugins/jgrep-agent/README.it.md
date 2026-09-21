# Plug-in Codex `jgrep-agent`

Questo plug-in insegna a Codex a usare `jgrep` come prima ricerca delle
posizioni nel codice. Privilegia l’output compatto, così l’agente legge solo
gli intervalli di sorgente necessari. Installare anche l’eseguibile `jgrep`:
il plug-in non lo include.

## Installazione

Da un terminale con il CLI di Codex disponibile, aggiungere il marketplace del
progetto e installare il plug-in:

```sh
codex plugin marketplace add xxvw/localjev-grep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@localjev-grep
```

Avviare una nuova sessione Codex per caricare il plug-in. Installare `jgrep`
seguendo il [README principale](https://github.com/xxvw/localjev-grep/blob/main/README.md) e la
[guida all’installazione e all’integrazione degli agenti](https://github.com/xxvw/localjev-grep/blob/main/docs/installation-and-agents.md).

## Ricerca compatta per agenti

Iniziare con `--ai`. Restituisce solo posizioni `percorso:riga`, senza testo
sorgente, e per impostazione predefinita limita l’output a 50 risultati:

```sh
# Ricerca semantica di un comportamento
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'dove vengono gestiti gli errori di autenticazione' src/

# Identificatore o testo noto: corrispondenza letterale senza modello
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/

# Motivo noto: espressione regolare Rust senza modello
jgrep --ai --ai-max-results 25 -r -E 'validate_(session|token)' src/
```

Interpretare sempre il suffisso finale che corrisponde a `:[0-9]+$` in un risultato. Un
percorso Windows può contenere un due punti precedente, per esempio
`C:\work\src\auth.rs:57`. Leggere solo gli intervalli stretti indicati prima di
ampliare la ricerca.

Se stderr avvisa che è stato raggiunto il limite, il risultato è incompleto:
restringere la query o la directory prima di aumentare `--ai-max-results`.
Usare `rg` se `jgrep` non è installato o se la ricerca compatta non può
esprimere l’attività.

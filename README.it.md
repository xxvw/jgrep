# jgrep

`jgrep` è un comando locale di ricerca semantica nello stile di `grep`.
Stampa le righe il cui significato corrisponde a una richiesta in linguaggio
naturale e conserva il normale flusso con file e pipe. Non richiede Python,
Ollama né un servizio in background.

L’interazione si ispira all’idea di Jev di restituire una decisione binaria di
rilevanza anziché generare testo. Il progetto non è affiliato a Jev, TypeSafe,
Qwen, Hugging Face o llama.cpp, non è da essi approvato e non è una loro
distribuzione. La documentazione canonica è il [README.md](README.md) inglese.

**Lingue:** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## Installazione

Installare senza clonare il repository: copiare integralmente il blocco della
propria piattaforma come un solo comando. Scarica il pacchetto di installazione
fissato a `v0.1.1`, ne verifica lo SHA-256 prima dell’estrazione ed esegue un
wrapper con messaggio iniziale in italiano. Il wrapper delega all’installatore
centrale condiviso e verificato, che seleziona e verifica nuovamente l’archivio
nativo per macOS (Apple Silicon e Intel), Windows x64 o Linux x64 (glibc 2.35
o successiva).

La versione `v0.1.1` precede la rinomina del repository; perciò i nomi
immutabili dell’archivio e della directory dell’installer iniziano ancora con
`localjev-grep`. Gli URL del repository, il sorgente attuale e il comando usano `jgrep`.

### macOS e Linux

Incollare integralmente questo singolo comando composto in Bash o zsh:

```sh
(
  set -e
  version=v0.1.1
  archive="localjev-grep-installers-${version}.tar.gz"
  workdir="$(mktemp -d)"
  trap 'rm -rf "$workdir"' EXIT
  base="https://github.com/xxvw/jgrep/releases/download/${version}"
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
  JGREP_REPOSITORY=xxvw/jgrep \
    bash "$workdir/localjev-grep-installers-${version}/installers/it/install.sh" \
    --version "$version"
)
```

### Windows PowerShell

Incollare integralmente questo singolo blocco PowerShell:

```powershell
& {
  $ErrorActionPreference = 'Stop'
  $version = 'v0.1.1'
  $archive = "localjev-grep-installers-$version.zip"
  $workdir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
  New-Item -ItemType Directory -Path $workdir | Out-Null
  try {
    $base = "https://github.com/xxvw/jgrep/releases/download/$version"
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile (Join-Path $workdir $archive)
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile (Join-Path $workdir "$archive.sha256")
    $manifest = (Get-Content -LiteralPath (Join-Path $workdir "$archive.sha256") -Raw).Trim()
    $manifestPattern = '^[A-Fa-f0-9]{64}  ' + [regex]::Escape($archive) + '$'
    if ($manifest -notmatch $manifestPattern) { throw 'installer bundle checksum manifest is invalid' }
    $expected = $manifest.Substring(0, 64).ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath (Join-Path $workdir $archive) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw 'installer bundle checksum mismatch' }
    Expand-Archive -LiteralPath (Join-Path $workdir $archive) -DestinationPath $workdir -Force
    & (Join-Path $workdir "localjev-grep-installers-$version\installers\it\install.ps1") -Version $version -Repository 'xxvw/jgrep'
  } finally {
    Remove-Item -LiteralPath $workdir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
```

Questi comandi non usano `git clone`, `curl | sh` né `Invoke-Expression`. Su
macOS/Linux la destinazione predefinita è `$HOME/.local/bin` (o `$XDG_BIN_HOME`
se impostato); aggiungerla a `PATH` se necessario. In PowerShell, aggiungere
`-AddToPath` alla chiamata finale dell’installatore per aggiungere la directory
al `PATH` utente. Per opzioni, installazioni offline e directory di destinazione,
consultare la [guida all’installazione](docs/installation-and-agents.md). Per
configurare un agente di programmazione, consultare la
[guida al plugin per agenti jgrep](plugins/jgrep-agent/README.it.md).

### Compilazione facoltativa dai sorgenti

Per lo sviluppo, un’architettura non supportata o un Linux meno recente,
compilare dai sorgenti con la toolchain Rust fissata in `rust-toolchain.toml`,
CMake e un compilatore C++ per llama.cpp integrato:

```sh
git clone https://github.com/xxvw/jgrep.git
cd jgrep
cargo build --release
./target/release/jgrep --help
```

Windows PowerShell:

```powershell
git clone https://github.com/xxvw/jgrep.git
Set-Location jgrep
cargo build --release
.\target\release\jgrep.exe --help
```

Su Apple Silicon `--device auto` può usare Metal. L’esecuzione CPU è
disponibile su tutte le piattaforme supportate e gli archivi di rilascio non
contengono istruzioni CPU specifiche della macchina di compilazione.

## Primo modello ed esempi

La ricerca semantica usa il GGUF Q8_0 ufficiale di Qwen2.5-0.5B-Instruct
(circa 676 MB). Viene ottenuto al primo uso semantico; per preparare la cache
in anticipo:

```sh
jgrep --download-model
```

La revisione e lo SHA-256 dell’artefatto sono fissati nel codice. Il file viene
verificato, collocato atomicamente nella cache applicativa dell’utente e poi
riutilizzato. Modello e cache non sono inclusi in Git né negli archivi sorgente.

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

Senza file, o con `-`, `jgrep` legge stdin. Più `-e` sono uniti da OR. Le
righe originali selezionate vanno su stdout; diagnostica e avanzamento del
download vanno su stderr.

## Uso offline e opzioni

Un modello predefinito nella cache, oppure un GGUF locale esplicito, può essere
usato senza richieste di rete:

```sh
jgrep --offline 'network connection failure' app.log
jgrep --model /path/to/model.gguf --offline 'network connection failure' app.log
```

`--offline` non usa mai la rete e fallisce senza un modello locale valido. Non
può essere combinato con `--download-model`. Con un contesto,
`--download-model` prepara la cache e poi cerca; senza contesto scarica solo
il modello.

Per impostazione predefinita il modello locale valuta ogni riga usando i token
`Yes` e `No`. `-E` usa le espressioni regolari Rust e `-F` una
sottostringa letterale; questi modi lessicali non scaricano né caricano il
modello. `-E` e `-F` sono incompatibili e `-i` vale solo per essi. GNU
BRE, PCRE e backreference non sono completamente compatibili.

`--ai` è pensato per gli agenti di programmazione: restituisce soltanto
posizioni compatte `percorso:riga`, senza testo sorgente corrispondente,
colore ANSI, punteggio o contesto. Per impostazione predefinita limita l’intera
invocazione a 50 posizioni; usare `--ai-max-results <n>` per modificare il
limite. L’agente può quindi recuperare solo gli intervalli stretti necessari.

| Opzione | Scopo |
| --- | --- |
| `-e <contesto>` | Aggiunge un contesto; basta una corrispondenza. |
| `-n`, `-H`, `-h` | Numeri di riga e visualizzazione o soppressione dei file. |
| `-c`, `-l`, `-L`, `-q`, `-m` | Conteggio, nomi, uscita silenziosa e limite per input. |
| `-v`, `-r`, `-A/-B/-C` | Inversione, ricerca ricorsiva e righe di contesto. |
| `--include`, `--exclude`, `--color`, `--line-buffered` | Filtri di percorso, colore e flush per riga. |
| `--threshold`, `--score` | Soglia semantica (predefinita `0.5`) e punteggio mostrato. |
| `--ai`, `--ai-max-results <n>` | Output compatto per agenti e limite di posizioni. |
| `--model`, `--download-model`, `--offline`, `--device auto\|cpu` | Modello, download, rete e dispositivo. |

Usare `--` prima di un contesto o percorso che inizia con `-`. Le opzioni
semantiche, come `--threshold` e `--score`, sono rifiutate nei modi
lessicali. `-m 0`, input vuoto, guida e ricerche lessicali non inizializzano
il modello.

## Benchmark dei token Codex

In un benchmark controllato di tre attività di localizzazione del codice,
`jgrep --ai -F` ha usato **il 16,4% di token Codex totali in meno** rispetto a
`rg -F`. I token di input non memorizzati nella cache sono diminuiti del 54,9%
e l’output dello strumento di ricerca del 97,1%. È una singola ripetizione, non
una garanzia generale. Le posizioni coincidevano in due attività; nell’attività
più ampia, la risposta Codex del ramo `rg` ha saltato una posizione presente
nell’output grezzo dello strumento.

Consultare [metodologia e limiti](benchmark/README.md),
[riepilogo dei risultati](benchmark/RESULTS-2026-09-22.md),
[dati delle singole esecuzioni](benchmark/results/2026-09-22.json) e
[log di esecuzione grezzi](benchmark/logs/2026-09-22/README.md).

## Limiti e compatibilità

Il punteggio è `sigmoid(logit(Yes) - logit(No))`. È un valore di rilevanza,
non una probabilità calibrata né una garanzia di correttezza. Ambiguità,
negazioni, lingua, righe lunghe e contenuto avversario possono cambiare i
risultati. Non vi sono garanzie di accuratezza, velocità o latenza; non usare
l’output come unica base per decisioni di sicurezza, legali, mediche o di
sicurezza informatica.

`jgrep` elabora in modo incrementale e mantiene l’ordine di input. Supporta
UTF-8, terminatori LF/CRLF e percorsi Unicode. Le righe oltre il limite
semantico di 4.096 token non vengono troncate in silenzio. La ricerca ricorsiva
non segue i symlink di directory e salta i binari rilevati con una diagnostica;
un file non testuale nominato esplicitamente è un errore. I codici di uscita
sono `0` se trova una selezione, `1` se non la trova e `2` in caso di
errore. Le pipe di output interrotte sono silenziose quando la piattaforma lo
consente.

## Sviluppo e licenza

Il controllo locale comune è:

```sh
cargo xtask ci
```

Il codice sorgente è **GPL-3.0-or-later**. Consultare [LICENSE](LICENSE),
[NOTICE](NOTICE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
[SECURITY.md](SECURITY.md) e [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
Il modello Qwen predefinito è un artefatto Apache-2.0 scaricato separatamente,
non codice sorgente del progetto sotto GPL.

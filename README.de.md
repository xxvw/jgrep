# jgrep

`jgrep` ist ein lokal ausgeführtes, semantisches Befehlszeilenwerkzeug im Stil
von `grep`. Es gibt die Eingabezeilen aus, deren Bedeutung zu einem natürlich-
sprachlichen Kontext passt, und funktioniert mit Dateien und Pipelines.
Python, Ollama und ein Hintergrunddienst sind nicht erforderlich.

Der Ansatz ist von Jevs „System-One“-Idee inspiriert, bei der eine binäre
Relevanzentscheidung statt Text erzeugt wird. Das Projekt ist jedoch weder mit
Jev, TypeSafe, Qwen, Hugging Face oder llama.cpp verbunden noch von ihnen
befürwortet oder vertrieben. Die maßgebliche Dokumentation ist die englische
[README.md](README.md).

**Sprachen:** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## Installation

Installieren Sie ohne das Repository zu klonen: Kopieren Sie den vollständigen
Block für Ihre Plattform als einen einzelnen Befehl in die Shell. Er lädt das
auf `v0.1.1` festgelegte Installationspaket, prüft dessen SHA-256 vor dem
Entpacken und startet einen Wrapper mit deutscher Startmeldung. Der Wrapper
delegiert an den gemeinsamen geprüften Kerninstaller, der das passende native
Archiv für macOS (Apple Silicon und Intel), Windows x64 oder Linux x64 (glibc
2.35 oder neuer) auswählt und erneut prüft.

Das Release `v0.1.1` ist älter als die Umbenennung des Repositorys. Deshalb
beginnen seine unveränderlichen Archiv- und Installerverzeichnisnamen weiterhin
mit `localjev-grep`; Repository-URLs, aktueller Quellcode und Befehl heißen `jgrep`.

### macOS und Linux

Diesen einzelnen zusammengesetzten Befehl vollständig in Bash oder zsh einfügen:

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
  bash "$workdir/localjev-grep-installers-${version}/installers/de/install.sh" \
    --version "$version"
)
```

### Windows PowerShell

Diesen einzelnen PowerShell-Block vollständig einfügen:

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
    & (Join-Path $workdir "localjev-grep-installers-$version\installers\de\install.ps1") -Version $version
  } finally {
    Remove-Item -LiteralPath $workdir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
```

Diese Befehle verwenden weder `git clone`, `curl | sh` noch
`Invoke-Expression`. Unter macOS/Linux ist `$HOME/.local/bin` das
Standardziel (oder `$XDG_BIN_HOME`, wenn gesetzt); bei Bedarf diesen Ordner zu
`PATH` hinzufügen. In PowerShell `-AddToPath` an den abschließenden
Installer-Aufruf anhängen, um den Ordner zum Benutzer-`PATH` hinzuzufügen.
Optionen, Offline-Installationen und Zielverzeichnisse beschreibt die
[Installationsanleitung](docs/installation-and-agents.md). Für die Einrichtung
eines Programmieragenten siehe die
[jgrep-Agent-Plugin-Anleitung](plugins/jgrep-agent/README.de.md).

### Optionale Kompilierung aus dem Quellcode

Für Entwicklung sowie auf nicht unterstützten Architekturen oder älterem Linux
kann das Projekt aus dem Quellcode gebaut werden; dafür werden die in
`rust-toolchain.toml` festgelegte Rust-Toolchain, CMake und ein C++-Compiler
für das eingebettete llama.cpp benötigt:

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

Auf Apple Silicon kann `--device auto` Metal verwenden. Die Ausführung auf der
CPU ist auf allen unterstützten Plattformen möglich; die Release-Artefakte
verwenden keine entwicklungsrechnerspezifischen CPU-Instruktionen.

## Erstes Modell und Beispiele

Die semantische Suche nutzt die offizielle Qwen2.5-0.5B-Instruct-GGUF-Variante
Q8_0 (etwa 676 MB). Sie wird beim ersten Bedarf geladen; der Cache lässt sich
vorab füllen mit:

```sh
jgrep --download-model
```

Das Artefakt ist auf Revision und SHA-256 festgelegt, wird vor der atomaren
Ablage im anwendungseigenen Benutzer-Cache geprüft und danach wiederverwendet.
Modell und Cache sind nicht Teil von Git oder Quellarchiven.

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

Ohne Dateiangabe oder mit `-` liest `jgrep` von stdin. Mehrere `-e`-Kontexte
werden ODER-verknüpft. Die Originalzeilen gehen an stdout; Diagnosemeldungen
und Download-Fortschritt gehen an stderr.

## Offline und Optionen

Eine gecachte Standarddatei oder ein lokales GGUF kann ohne Netzverbindung
verwendet werden:

```sh
jgrep --offline 'network connection failure' app.log
jgrep --model /path/to/model.gguf --offline 'network connection failure' app.log
```

`--offline` fordert nie das Netz an und scheitert ohne gültiges lokales Modell.
Es kann nicht mit `--download-model` kombiniert werden. Mit Kontext lädt
`--download-model` erst den Cache und sucht anschließend; ohne Kontext lädt es
nur herunter.

Standardmäßig beurteilt das lokale Modell jede Zeile anhand der Token `Yes`
und `No`. `-E` verwendet die `regex`-Syntax von Rust, `-F` sucht feste
Zeichenketten; beide Modi laden kein Modell herunter und laden es nicht. `-E`
und `-F` schließen sich aus, `-i` gilt nur für diese lexikalischen Modi.
GNU-BRE, PCRE und Rückreferenzen werden nicht vollständig unterstützt.

`--ai` ist für Programmieragenten gedacht: Es gibt nur kompakte Positionen
im Format `pfad:zeile` aus, ohne passenden Quelltext, ANSI-Farbe, Score oder
Kontext. Standardmäßig begrenzt es den gesamten Aufruf auf 50 Positionen;
mit `--ai-max-results <n>` lässt sich dieses Limit anpassen. Anschließend
kann der Agent nur die benötigten engen Bereiche abrufen.

| Option | Zweck |
| --- | --- |
| `-e <Kontext>` | Kontext hinzufügen; jede passende Bedingung genügt. |
| `-n`, `-H`, `-h` | Zeilennummern, Dateinamen zeigen oder unterdrücken. |
| `-c`, `-l`, `-L`, `-q`, `-m` | Zählen, Dateinamen, stille Suche, Höchstzahl je Eingabe. |
| `-v`, `-r`, `-A/-B/-C` | Auswahl umkehren, rekursiv suchen, Kontextzeilen ausgeben. |
| `--include`, `--exclude`, `--color`, `--line-buffered` | Pfadfilter, Farbe und zeilenweises Flushen. |
| `--threshold`, `--score` | Semantischer Schwellenwert (standardmäßig `0.5`) und Score-Ausgabe. |
| `--ai`, `--ai-max-results <n>` | Kompakte Agent-Ausgabe und Positionslimit. |
| `--model`, `--download-model`, `--offline`, `--device auto\|cpu` | Modell- und Geräteverwaltung. |

`--` trennt einen Kontext oder Pfad, der mit `-` beginnt. Semantik-spezifische
Optionen wie `--threshold` und `--score` werden in den lexikalischen Modi
abgewiesen. `-m 0`, leere Eingaben, Hilfe und lexikalische Suchen initialisieren
kein Modell.

## Codex-Token-Benchmark

In einem kontrollierten Benchmark mit drei Aufgaben zur Suche nach
Codepositionen verbrauchte `jgrep --ai -F` **16,4 % weniger Codex-Gesamttoken**
als `rg -F`. Nicht zwischengespeicherte Eingabetoken sanken um 54,9 %, die
Ausgabe des Suchwerkzeugs um 97,1 %. Dies ist ein einzelner Durchlauf und keine
allgemeine Garantie. Bei zwei Aufgaben stimmten die Positionen überein; bei der
breiten Aufgabe übersprang die Codex-Antwort im `rg`-Arm eine Position, die in
der unveränderten Werkzeugausgabe vorhanden war.

Siehe [Methodik und Einschränkungen](benchmark/README.md),
[Ergebnisübersicht](benchmark/RESULTS-2026-09-22.md),
[Daten der einzelnen Läufe](benchmark/results/2026-09-22.json) und
[unveränderte Ausführungsprotokolle](benchmark/logs/2026-09-22/README.md).

## Grenzen und Kompatibilität

Der Score lautet `sigmoid(logit(Yes) - logit(No))`. Er ist ein Relevanzwert,
keine kalibrierte Wahrscheinlichkeit und keine Richtigkeitsgarantie.
Mehrdeutigkeit, Verneinungen, Sprache, lange Zeilen und adversarialer Inhalt
können Ergebnisse verändern. Es gibt keine Zusage zu Genauigkeit, Durchsatz
oder Latenz. Das Werkzeug darf nicht als alleinige Grundlage für sicherheits-,
rechts-, medizin- oder sicherheitskritische Entscheidungen dienen.

`jgrep` verarbeitet Daten inkrementell und in Eingabereihenfolge, unterstützt
UTF-8, LF/CRLF und Unicode-Pfade. Zeilen oberhalb des semantischen
4.096-Token-Limits werden nicht still gekürzt. Die rekursive Suche folgt keinen
Verzeichnis-Symlinks und überspringt erkannte Binärdateien mit Diagnose; eine
explizit angegebene Nicht-Textdatei ist ein Fehler. Die Rückgabecodes sind `0`
bei Auswahl, `1` ohne Auswahl und `2` bei Fehler. Defekte Ausgabe-Pipelines
werden nach Möglichkeit still behandelt.

## Entwicklung und Lizenz

Die gemeinsame lokale Prüfung lautet:

```sh
cargo xtask ci
```

Der Quellcode steht unter **GPL-3.0-or-later**. Siehe [LICENSE](LICENSE),
[NOTICE](NOTICE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
[SECURITY.md](SECURITY.md) und [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
Das Standard-Qwen-Modell ist ein separat heruntergeladenes Apache-2.0-Artefakt,
nicht GPL-lizenzierter Projektquellcode.

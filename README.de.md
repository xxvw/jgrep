# localjev-grep

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

Native Archive für macOS (Apple Silicon und Intel), Windows x64 sowie Linux
x64 (glibc 2.35 oder neuer) gibt es unter
[GitHub Releases](https://github.com/xxvw/localjev-grep/releases). Nach dem
Entpacken `jgrep` beziehungsweise `jgrep.exe` in den `PATH` aufnehmen.

Für eine lokale Kompilierung werden die in `rust-toolchain.toml` festgelegte
Rust-Toolchain, CMake und ein C++-Compiler für das eingebettete llama.cpp
benötigt:

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

| Option | Zweck |
| --- | --- |
| `-e <Kontext>` | Kontext hinzufügen; jede passende Bedingung genügt. |
| `-n`, `-H`, `-h` | Zeilennummern, Dateinamen zeigen oder unterdrücken. |
| `-c`, `-l`, `-L`, `-q`, `-m` | Zählen, Dateinamen, stille Suche, Höchstzahl je Eingabe. |
| `-v`, `-r`, `-A/-B/-C` | Auswahl umkehren, rekursiv suchen, Kontextzeilen ausgeben. |
| `--include`, `--exclude`, `--color`, `--line-buffered` | Pfadfilter, Farbe und zeilenweises Flushen. |
| `--threshold`, `--score` | Semantischer Schwellenwert (standardmäßig `0.5`) und Score-Ausgabe. |
| `--model`, `--download-model`, `--offline`, `--device auto\|cpu` | Modell- und Geräteverwaltung. |

`--` trennt einen Kontext oder Pfad, der mit `-` beginnt. Semantik-spezifische
Optionen wie `--threshold` und `--score` werden in den lexikalischen Modi
abgewiesen. `-m 0`, leere Eingaben, Hilfe und lexikalische Suchen initialisieren
kein Modell.

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

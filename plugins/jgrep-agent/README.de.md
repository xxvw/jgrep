# jgrep-agent-Plugin für Codex

jgrep-agent weist Coding-Agenten an, Code zuerst mit der kompakten Ausgabe von
jgrep zu finden. Dadurch werden nur relevante Quellbereiche gelesen und
Tool-Ausgaben bleiben klein.

## Installation

Fügen Sie in Codex den lokalen Marketplace dieses Repositorys hinzu und
installieren Sie das Plugin:

~~~sh
codex plugin marketplace add xxvw/jgrep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@jgrep
~~~

Starten Sie nach der Installation eine neue Codex-Sitzung. Installieren Sie
anschließend die ausführbare Datei jgrep gemäß dem
[Haupt-README](https://github.com/xxvw/jgrep/blob/main/README.md) oder der
[Installations- und Agentenanleitung](https://github.com/xxvw/jgrep/blob/main/docs/installation-and-agents.md).

## Arbeitsablauf für Agenten

Verwenden Sie --ai als erste Suche. Jeder Treffer ist ein kompakter Quellort
ohne Quelltext:

~~~sh
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/
jgrep --ai --ai-max-results 25 -r -E 'TODO|FIXME' src/
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'wo Authentifizierungsfehler behandelt werden' src/
~~~

Teilen Sie jeden Datensatz am abschließenden :LINE, also am letzten
Doppelpunkt vor einer Dezimalzahl. Teilen Sie nicht am ersten Doppelpunkt: Ein
Windows-Pfad kann beispielsweise mit C:\ beginnen. -:LINE bezeichnet
Standardeingabe aus einer reproduzierbaren Pipeline.

Nutzen Sie -F für bekannte Symbole oder festen Text und -E für reguläre
Muster. Beide sind präzise lexikalische Suchen und laden kein Modell. Für
Verhalten oder Konzepte verwenden Sie die semantische --ai-Suche.

Standardmäßig gibt es höchstens 50 Quellorte. Meldet jgrep auf stderr, dass
dieses Limit erreicht wurde, ist das Ergebnis unvollständig. Grenzen Sie dann
Abfrage oder Verzeichnis ein, bevor Sie --ai-max-results erhöhen. Lesen Sie
nach der Suche nur die genannten Zeilenbereiche.

Falls jgrep nicht installiert ist oder die kompakte Suche die Anfrage nicht
ausdrücken kann, verwenden Sie rg als Rückfall:

~~~sh
rg -n -F 'validate_session' src/
~~~

# Plug-in Codex `jgrep-agent`

Ce plug-in donne à Codex des consignes pour employer `jgrep` comme première
recherche d’emplacements dans le code. Il privilégie la sortie compacte afin
de ne lire que les plages source nécessaires. Installez aussi l’exécutable
`jgrep` : le plug-in ne l’inclut pas.

## Installation

Depuis un terminal où le CLI Codex est disponible, ajoutez le marketplace du
projet puis installez le plug-in :

```sh
codex plugin marketplace add xxvw/localjev-grep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@localjev-grep
```

Démarrez ensuite une nouvelle session Codex pour charger le plug-in. Installez
`jgrep` en suivant le [README principal](https://github.com/xxvw/localjev-grep/blob/main/README.md) et le
[guide d’installation et d’intégration des agents](https://github.com/xxvw/localjev-grep/blob/main/docs/installation-and-agents.md).

## Recherche compacte pour les agents

Commencez par `--ai`. Il imprime seulement des emplacements
`chemin:ligne`, sans texte source, et limite par défaut la sortie à 50
résultats :

```sh
# Recherche sémantique d’un comportement
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'où les échecs d’authentification sont traités' src/

# Identifiant ou texte connu : correspondance littérale sans modèle
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/

# Motif connu : expression régulière Rust sans modèle
jgrep --ai --ai-max-results 25 -r -E 'validate_(session|token)' src/
```

Analysez toujours le suffixe final correspondant à `:[0-9]+$` dans un résultat. Un
chemin Windows peut déjà contenir un deux-points, par exemple
`C:\work\src\auth.rs:57`. Lisez seulement les petites plages citées avant
d’élargir la recherche.

Si stderr indique que la limite a été atteinte, le résultat est incomplet :
restreignez la requête ou le répertoire avant d’augmenter
`--ai-max-results`. Utilisez `rg` si `jgrep` n’est pas installé ou si la
recherche compacte ne peut pas exprimer la tâche.

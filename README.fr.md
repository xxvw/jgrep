# localjev-grep

`jgrep` est une commande locale de type `grep` pour la recherche sémantique.
Elle affiche les lignes dont le sens correspond à une requête en langage
naturel et conserve le flux de travail habituel avec fichiers et tubes.
Python, Ollama et un service permanent ne sont pas nécessaires.

Le projet s’inspire de l’idée de Jev de renvoyer une décision binaire de
pertinence plutôt que de générer du texte. Il n’est ni affilié à, ni approuvé
par, ni une distribution de Jev, TypeSafe, Qwen, Hugging Face ou llama.cpp.
La documentation de référence est le [README.md](README.md) anglais.

**Langues :** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## Installation

Les archives natives pour macOS (Apple Silicon et Intel), Windows x64 et Linux
x64 (glibc 2.35 ou ultérieure) sont publiées dans
[GitHub Releases](https://github.com/xxvw/localjev-grep/releases). Après
extraction, ajoutez `jgrep` ou `jgrep.exe` au `PATH`.

Pour compiler les sources, il faut la chaîne Rust fixée dans
`rust-toolchain.toml`, CMake et un compilateur C++ pour llama.cpp embarqué :

```sh
git clone https://github.com/xxvw/localjev-grep.git
cd localjev-grep
cargo build --release
./target/release/jgrep --help
```

Windows PowerShell :

```powershell
git clone https://github.com/xxvw/localjev-grep.git
Set-Location localjev-grep
cargo build --release
.\target\release\jgrep.exe --help
```

Sur Apple Silicon, `--device auto` peut utiliser Metal. L’exécution CPU est
disponible sur toutes les plates-formes prises en charge et les archives de
publication évitent les instructions CPU propres à la machine de compilation.

## Modèle initial et exemples

La recherche sémantique utilise le GGUF Q8_0 officiel de Qwen2.5-0.5B-Instruct
(environ 676 Mo). Il est récupéré au premier besoin ; pour préparer le cache à
l’avance :

```sh
jgrep --download-model
```

La révision et la somme SHA-256 sont fixées dans le code. Le fichier est
vérifié, placé de façon atomique dans un cache applicatif propre à l’utilisateur,
puis réutilisé. Le modèle et le cache ne font pas partie de Git ni des archives
sources.

Bash / zsh :

```sh
jgrep 'network connection failure' app.log
cat app.log | jgrep 'network connection failure'
jgrep -n -r --include '*.log' 'authentication failed' logs/
jgrep -E -i 'error|warning' app.log
jgrep -F 'connection refused' app.log
```

Windows PowerShell :

```powershell
.\jgrep.exe 'network connection failure' .\app.log
Get-Content .\app.log | .\jgrep.exe 'network connection failure'
.\jgrep.exe -n -r --include '*.log' 'authentication failed' .\logs
.\jgrep.exe -E -i 'error|warning' .\app.log
```

Sans fichier, ou avec `-`, `jgrep` lit stdin. Plusieurs `-e` sont reliés
par un OU. Les lignes d’origine sélectionnées vont sur stdout ; diagnostics et
progression du téléchargement vont sur stderr.

## Hors ligne et options

Un modèle standard déjà en cache, ou un GGUF local explicite, peut être utilisé
sans demande réseau :

```sh
jgrep --offline 'network connection failure' app.log
jgrep --model /path/to/model.gguf --offline 'network connection failure' app.log
```

`--offline` n’utilise jamais le réseau et échoue sans modèle local valide. Il
est incompatible avec `--download-model`. Avec un contexte,
`--download-model` prépare le cache puis recherche ; sans contexte, il
télécharge seulement le modèle.

Par défaut, le modèle local juge chaque ligne à partir des jetons `Yes` et
`No`. `-E` utilise les expressions régulières Rust et `-F` une chaîne
littérale ; ces modes lexicaux ne téléchargent ni ne chargent le modèle.
`-E` et `-F` sont incompatibles, et `-i` est réservé aux modes lexicaux.
GNU BRE, PCRE et les références arrière ne sont pas entièrement compatibles.

`--ai` est conçu pour les agents de programmation : il ne renvoie que des
emplacements compacts `chemin:ligne`, sans texte source correspondant,
couleur ANSI, score ni contexte. Il limite par défaut l’invocation entière à
50 emplacements ; utilisez `--ai-max-results <n>` pour modifier cette limite.
L’agent peut ensuite demander uniquement les plages étroites dont il a besoin.

| Option | Rôle |
| --- | --- |
| `-e <contexte>` | Ajoute un contexte ; n’importe quelle correspondance suffit. |
| `-n`, `-H`, `-h` | Numéros de ligne, afficher ou masquer les noms de fichiers. |
| `-c`, `-l`, `-L`, `-q`, `-m` | Compte, noms, sortie silencieuse et limite par entrée. |
| `-v`, `-r`, `-A/-B/-C` | Inversion, parcours récursif et lignes de contexte. |
| `--include`, `--exclude`, `--color`, `--line-buffered` | Filtres de chemin, couleur et flush par ligne. |
| `--threshold`, `--score` | Seuil sémantique (par défaut `0.5`) et score affiché. |
| `--ai`, `--ai-max-results <n>` | Sortie compacte pour agent et limite d’emplacements. |
| `--model`, `--download-model`, `--offline`, `--device auto\|cpu` | Modèle, téléchargement, réseau et appareil. |

Utilisez `--` avant un contexte ou un chemin qui commence par `-`. Les
options propres à la sémantique, dont `--threshold` et `--score`, sont
refusées dans les modes lexicaux. `-m 0`, l’entrée vide, l’aide et les
recherches lexicales n’initialisent pas le modèle.

## Limites et compatibilité

Le score est `sigmoid(logit(Yes) - logit(No))`. C’est une valeur de
pertinence, pas une probabilité calibrée ni une garantie de correction.
L’ambiguïté, les négations, la langue, les longues lignes et un contenu
adversarial peuvent modifier les résultats. Aucune promesse de précision, de
débit ou de latence n’est fournie ; ne l’utilisez pas comme seule base d’une
décision de sûreté, juridique, médicale ou de sécurité informatique.

`jgrep` traite les données progressivement et préserve leur ordre. Il gère
UTF-8, les fins de ligne LF/CRLF et les chemins Unicode. Les lignes au-delà de
la limite sémantique de 4 096 jetons ne sont pas tronquées silencieusement. La
recherche récursive ne suit pas les liens symboliques de répertoires et ignore
les binaires détectés avec un diagnostic ; un fichier non textuel explicitement
nommé est une erreur. Les codes de sortie sont `0` en cas de sélection, `1`
sans sélection et `2` en cas d’erreur. Les tubes de sortie rompus sont traités
silencieusement lorsque la plate-forme le permet.

## Développement et licence

La vérification locale commune est :

```sh
cargo xtask ci
```

Le code source est sous **GPL-3.0-or-later**. Consultez [LICENSE](LICENSE),
[NOTICE](NOTICE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
[SECURITY.md](SECURITY.md) et [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
Le modèle Qwen par défaut est un artefact Apache-2.0 téléchargé séparément, et
non du code source du projet sous GPL.

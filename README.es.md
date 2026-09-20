# localjev-grep

`jgrep` es un comando local de estilo `grep` para búsqueda semántica. Muestra
las líneas cuyo significado coincide con una consulta en lenguaje natural y
conserva el flujo habitual con archivos y tuberías. No requiere Python, Ollama
ni un servidor persistente.

Está inspirado en el enfoque de Jev de devolver una decisión binaria de
relevancia en vez de generar texto. Este proyecto no está afiliado a Jev,
TypeSafe, Qwen, Hugging Face ni llama.cpp, no cuenta con su aprobación y no es
una distribución de ellos. La referencia canónica es el [README.md](README.md)
en inglés.

**Idiomas:** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## Instalación

Los archivos nativos para macOS (Apple Silicon e Intel), Windows x64 y Linux
x64 (glibc 2.35 o posterior) se publican en
[GitHub Releases](https://github.com/xxvw/localjev-grep/releases). Descomprima
el archivo y añada `jgrep` o `jgrep.exe` al `PATH`.

Para compilar desde el código fuente se necesitan la herramienta Rust fijada en
`rust-toolchain.toml`, CMake y un compilador de C++ para llama.cpp incorporado:

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

En Apple Silicon, `--device auto` puede usar Metal. La CPU funciona en todas
las plataformas admitidas y los binarios de distribución no incluyen
optimizaciones de CPU específicas del equipo de compilación.

## Modelo inicial y ejemplos

La búsqueda semántica usa el GGUF Q8_0 oficial de Qwen2.5-0.5B-Instruct
(aprox. 676 MB). Se obtiene al primer uso semántico; para preparar la caché
antes, ejecute:

```sh
jgrep --download-model
```

La revisión y el SHA-256 del artefacto están fijados. El archivo se verifica,
se coloca de forma atómica en la caché de la aplicación del usuario y se
reutiliza. El modelo y la caché no se incluyen en Git ni en los archivos fuente.

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

Sin archivos, o usando `-`, `jgrep` lee stdin. Varios `-e` se combinan con
OR. Las líneas originales seleccionadas van a stdout; los diagnósticos y el
progreso de descarga van a stderr.

## Uso sin conexión y opciones

Se puede usar el modelo predeterminado almacenado en caché, o un GGUF local,
sin solicitar la red:

```sh
jgrep --offline 'network connection failure' app.log
jgrep --model /path/to/model.gguf --offline 'network connection failure' app.log
```

`--offline` nunca usa la red y falla si no existe un modelo local válido. No
se puede combinar con `--download-model`. Con un contexto,
`--download-model` calienta la caché y después busca; sin contexto solo
descarga el modelo.

El modo predeterminado evalúa cada línea localmente con los tokens `Yes` y
`No`. `-E` usa expresiones regulares Rust y `-F` una subcadena literal;
estos modos léxicos no descargan ni cargan el modelo. `-E` y `-F` son
incompatibles y `-i` solo sirve en ellos. No se promete compatibilidad total
con GNU BRE, PCRE ni retroreferencias.

`--ai` está pensado para agentes de programación: devuelve solo ubicaciones
compactas `ruta:línea`, sin texto fuente coincidente, color ANSI, puntuación
ni contexto. De forma predeterminada limita toda la invocación a 50
ubicaciones; use `--ai-max-results <n>` para cambiar ese límite. Después, el
agente puede obtener únicamente los rangos estrechos que necesite.

| Opción | Uso |
| --- | --- |
| `-e <contexto>` | Añade un contexto; basta cualquier coincidencia. |
| `-n`, `-H`, `-h` | Números de línea y mostrar u ocultar nombres de archivo. |
| `-c`, `-l`, `-L`, `-q`, `-m` | Recuento, nombres, salida silenciosa y límite por entrada. |
| `-v`, `-r`, `-A/-B/-C` | Inversión, búsqueda recursiva y líneas de contexto. |
| `--include`, `--exclude`, `--color`, `--line-buffered` | Filtros de ruta, color y vaciado por línea. |
| `--threshold`, `--score` | Umbral semántico (predeterminado `0.5`) y valor mostrado. |
| `--ai`, `--ai-max-results <n>` | Salida compacta para agentes y límite de ubicaciones. |
| `--model`, `--download-model`, `--offline`, `--device auto\|cpu` | Modelo, descarga, red y dispositivo. |

Use `--` antes de un contexto o una ruta que comienza con `-`. Las opciones
exclusivas del modo semántico, como `--threshold` y `--score`, se rechazan
en modos léxicos. `-m 0`, la entrada vacía, la ayuda y las búsquedas léxicas
no inicializan el modelo.

## Límites y compatibilidad

La puntuación es `sigmoid(logit(Yes) - logit(No))`. Es una señal de
relevancia, no una probabilidad calibrada ni una garantía de corrección. La
ambigüedad, las negaciones, el idioma, las líneas largas y contenido adversario
pueden alterar los resultados. No se hacen promesas de precisión, rendimiento
ni latencia; no use el resultado como única base de decisiones de seguridad,
legales, médicas o de ciberseguridad.

`jgrep` procesa de forma incremental y mantiene el orden de entrada. Admite
UTF-8, finales LF/CRLF y rutas Unicode. Las líneas que superan el límite
semántico de 4.096 tokens no se truncan silenciosamente. La búsqueda recursiva
no sigue enlaces simbólicos de directorios y omite binarios detectados con un
diagnóstico; un archivo no textual indicado explícitamente es un error. Los
códigos de salida son `0` si hay selección, `1` si no la hay y `2` ante
error. Las tuberías de salida rotas se manejan silenciosamente cuando es posible.

## Desarrollo y licencia

La comprobación local común es:

```sh
cargo xtask ci
```

El código fuente se publica bajo **GPL-3.0-or-later**. Consulte
[LICENSE](LICENSE), [NOTICE](NOTICE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
[SECURITY.md](SECURITY.md) y [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
El modelo Qwen predeterminado es un artefacto Apache-2.0 descargado por
separado, no código fuente del proyecto bajo GPL.

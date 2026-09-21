# localjev-grep

`jgrep` — локальная семантическая команда в стиле `grep`. Она выводит строки,
смысл которых соответствует запросу на естественном языке, и сохраняет
привычную работу с файлами и конвейерами. Python, Ollama и фоновая служба не
нужны.

Подход вдохновлён идеей Jev возвращать бинарное решение о релевантности, а не
генерировать текст. Проект не аффилирован с Jev, TypeSafe, Qwen, Hugging Face
или llama.cpp, не одобрен ими и не является их дистрибутивом. Полное описание
поддерживается в английском [README.md](README.md).

**Языки:** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## Установка

Установите программу без клонирования репозитория: вставьте целиком блок для
вашей платформы как одну команду. Он скачивает пакет установки, закреплённый
на `v0.1.1`, проверяет его SHA-256 до распаковки и запускает обёртку с русским
стартовым сообщением. Обёртка передаёт выполнение общему проверенному основному
установщику, который выбирает и повторно проверяет подходящий нативный архив
для macOS (Apple Silicon и Intel), Windows x64 или Linux x64 (glibc 2.35 или
новее).

### macOS и Linux

Вставьте эту единую составную команду целиком в Bash или zsh:

```sh
(
  set -e
  version=v0.1.1
  archive="localjev-grep-installers-${version}.tar.gz"
  workdir="$(mktemp -d)"
  trap 'rm -rf "$workdir"' EXIT
  base="https://github.com/xxvw/localjev-grep/releases/download/${version}"
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
  bash "$workdir/localjev-grep-installers-${version}/installers/ru/install.sh" \
    --version "$version"
)
```

### Windows PowerShell

Вставьте этот единый блок PowerShell целиком:

```powershell
& {
  $ErrorActionPreference = 'Stop'
  $version = 'v0.1.1'
  $archive = "localjev-grep-installers-$version.zip"
  $workdir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
  New-Item -ItemType Directory -Path $workdir | Out-Null
  try {
    $base = "https://github.com/xxvw/localjev-grep/releases/download/$version"
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile (Join-Path $workdir $archive)
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile (Join-Path $workdir "$archive.sha256")
    $manifest = (Get-Content -LiteralPath (Join-Path $workdir "$archive.sha256") -Raw).Trim()
    $manifestPattern = '^[A-Fa-f0-9]{64}  ' + [regex]::Escape($archive) + '$'
    if ($manifest -notmatch $manifestPattern) { throw 'installer bundle checksum manifest is invalid' }
    $expected = $manifest.Substring(0, 64).ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath (Join-Path $workdir $archive) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw 'installer bundle checksum mismatch' }
    Expand-Archive -LiteralPath (Join-Path $workdir $archive) -DestinationPath $workdir -Force
    & (Join-Path $workdir "localjev-grep-installers-$version\installers\ru\install.ps1") -Version $version
  } finally {
    Remove-Item -LiteralPath $workdir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
```

Эти команды не используют `git clone`, `curl | sh` или
`Invoke-Expression`. В macOS/Linux путь по умолчанию — `$HOME/.local/bin`
(либо `$XDG_BIN_HOME`, если он задан); при необходимости добавьте его в
`PATH`. В PowerShell добавьте `-AddToPath` к заключительному вызову установщика,
чтобы включить этот каталог в пользовательский `PATH`. Параметры, автономную
установку и каталоги назначения описаны в
[руководстве по установке](docs/installation-and-agents.md). Для настройки
агента программирования смотрите
[руководство по плагину jgrep для агента](plugins/jgrep-agent/README.ru.md).

### Необязательная сборка из исходного кода

Для разработки, неподдерживаемой архитектуры или более старого Linux можно
собрать программу из исходного кода; нужны зафиксированная в
`rust-toolchain.toml` версия Rust, CMake и C++-компилятор для встроенной
зависимости llama.cpp:

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

На Apple Silicon `--device auto` может использовать Metal. Выполнение на CPU
доступно на всех поддерживаемых платформах; релизные сборки не используют
специфичные для компьютера разработчика инструкции CPU.

## Модель и примеры

Семантический режим использует официальный Qwen2.5-0.5B-Instruct GGUF Q8_0
(около 676 МБ). Он загружается при первом семантическом поиске. Чтобы заранее
подготовить кэш, выполните:

```sh
jgrep --download-model
```

Версия модели и SHA-256 закреплены в исходном коде. Файл проверяется и
атомарно помещается в пользовательский кэш приложения, а затем используется
повторно. Модель и кэш не включаются в Git или исходные архивы.

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

Если файлы не указаны или указан `-`, ввод читается из stdin. Несколько
`-e` объединяются по ИЛИ. Оригинальные выбранные строки печатаются в stdout,
а диагностика и ход загрузки — в stderr.

## Офлайн-режим и параметры

Кэшированную стандартную модель или указанный локальный GGUF можно запускать
без сети:

```sh
jgrep --offline 'network connection failure' app.log
jgrep --model /path/to/model.gguf --offline 'network connection failure' app.log
```

`--offline` не обращается к сети и завершится с ошибкой, если нет корректной
локальной модели. Он несовместим с `--download-model`. С контекстом
`--download-model` сначала заполняет кэш, затем ищет; без контекста команда
только скачивает модель.

По умолчанию локальная модель оценивает каждую строку по токенам `Yes` и
`No`. `-E` включает синтаксис регулярных выражений Rust, `-F` — поиск
фиксированной строки; оба лексических режима не скачивают и не загружают
модель. `-E` и `-F` нельзя сочетать, `-i` доступен только в этих режимах.
GNU BRE, PCRE и обратные ссылки не поддерживаются полностью.

| Параметр | Назначение |
| --- | --- |
| `-e <контекст>` | Добавить контекст; достаточно любого совпадения. |
| `-n`, `-H`, `-h` | Номера строк, показ или скрытие имён файлов. |
| `-c`, `-l`, `-L`, `-q`, `-m` | Счёт, имена файлов, тихий выход, лимит строк на ввод. |
| `-v`, `-r`, `-A/-B/-C` | Инверсия, рекурсивный поиск, строки контекста. |
| `--include`, `--exclude`, `--color`, `--line-buffered` | Отбор путей, цвет и построчная очистка буфера. |
| `--threshold`, `--score` | Семантический порог (по умолчанию `0.5`) и показ оценки. |
| `--ai`, `--ai-max-results <ЧИСЛО>` | Компактный вывод расположений для кодирующих агентов и его лимит. |
| `--model`, `--download-model`, `--offline`, `--device auto\|cpu` | Модель, загрузка, сеть и устройство вывода. |

Используйте `--` перед контекстом или путём, который начинается с `-`.
Семантические параметры, например `--threshold` и `--score`, в лексических
режимах отклоняются. `-m 0`, пустой ввод, справка и лексический поиск не
инициализируют модель.

`--ai` выводит только компактные расположения `путь:строка`: без исходного
текста совпадений, ANSI-цвета, оценки или строк контекста. По умолчанию за
один запуск выводится не более 50 расположений; измените предел через
`--ai-max-results`. Затем кодирующий агент может запросить лишь нужные узкие
диапазоны строк, сокращая расход токенов при вызовах инструментов.

## Ограничения и совместимость

Оценка вычисляется как `sigmoid(logit(Yes) - logit(No))`. Это показатель
релевантности, а не калиброванная вероятность и не гарантия правильности.
Неоднозначность, отрицания, язык, длинные строки и враждебное содержимое
файлов могут менять результат. Нет обещаний точности, производительности или
задержки; не используйте результат как единственное основание для решений,
связанных с безопасностью, правом, медициной или информационной безопасностью.

`jgrep` читает потоково и выводит строки в порядке ввода, поддерживает UTF-8,
LF/CRLF и пути Unicode. Строки выше семантического лимита в 4 096 токенов не
обрезаются молча. Рекурсивный поиск не следует по символьным ссылкам на
каталоги и пропускает обнаруженные двоичные файлы с диагностикой; явно
указанный нетекстовый файл — ошибка. Коды выхода: `0` — найдено,
`1` — не найдено, `2` — ошибка. Разрыв выходного конвейера по возможности
обрабатывается без лишнего сообщения.

## Разработка и лицензия

Общая локальная проверка:

```sh
cargo xtask ci
```

Исходный код распространяется по **GPL-3.0-or-later**. См. [LICENSE](LICENSE),
[NOTICE](NOTICE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md),
[SECURITY.md](SECURITY.md) и [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
Стандартная модель Qwen — отдельно скачиваемый материал Apache-2.0, а не
исходный код проекта под GPL.

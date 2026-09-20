# localjev-grep

`jgrep`은 자연어 문맥과 의미가 맞는 입력 줄을 찾는 로컬 grep 스타일 CLI입니다. 기본 의미 검색에서는 로컬 Qwen 모델이 각 줄의 관련성을 Yes/No로 판정하고, 선택된 원본 줄을 출력합니다. `-E` 정규식과 `-F` 고정 문자열의 일반 검색도 제공합니다.

> **v0.1.0:** macOS(Apple Silicon/Intel), Windows x64, Linux x64용 버전별 네이티브 아카이브는 [GitHub Releases](https://github.com/xxvw/localjev-grep/releases)에서 받을 수 있습니다. 정확도·처리량·지연 시간에 관한 벤치마크 보장은 제공하지 않습니다.

`jgrep`은 Jev의 예/아니오 판정 상호작용 방식을 참고했을 뿐입니다. 이 프로젝트는 Jev, TypeSafe, Qwen, Hugging Face, llama.cpp와 제휴하거나 이들의 승인을 받은 적이 없으며, 이들 어느 하나의 배포판도 아닙니다. 자세한 기능 명세는 영어 정본 [README.md](README.md)를 따릅니다.

**언어:** [English](README.md) · [日本語](README.ja.md) · [简体中文](README.zh-CN.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Deutsch](README.de.md) · [Русский](README.ru.md) · [Français](README.fr.md) · [Português (Brasil)](README.pt-BR.md) · [Italiano](README.it.md) · [العربية](README.ar.md) · [हिन्दी](README.hi.md)

## 설치

지원 아카이브는 macOS Apple Silicon/Intel, Windows x64, Linux x64(glibc 2.35 이상)용입니다. 운영체제에 맞는 압축 파일은 [GitHub Releases](https://github.com/xxvw/localjev-grep/releases)에서 다운로드합니다.

소스에서 빌드하려면 다음을 실행합니다.

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

소스 빌드에는 `rust-toolchain.toml`에 고정된 Rust 도구 모음, CMake, 그리고 내장 llama.cpp를 빌드할 C++ 컴파일러가 필요합니다. Apple Silicon에서는 `auto` 장치 설정이 Metal을 사용할 수 있고, 모든 지원 플랫폼에서 CPU 실행을 사용할 수 있습니다.

## 사용법

```text
jgrep [OPTIONS] <context> [FILE ...]
```

파일을 생략하거나 파일 인수로 `-`를 주면 표준 입력을 읽습니다. 위치 문맥 대신 `-e`를 쓸 수 있으며, 여러 `-e` 문맥은 OR 조건으로 결합됩니다.

### Bash

```sh
# 기본 모델을 미리 내려받기(문맥 없이 실행하면 다운로드만 수행)
jgrep --download-model

# 기본 의미 검색
jgrep 'database login was rejected' service.log

# 스트리밍 파이프와 재귀 검색
journalctl -f | jgrep --line-buffered 'connection was reset'
jgrep -n -r --include '*.log' 'authentication failed' logs/

# 네트워크를 사용하지 않는 의미 검색
jgrep --offline 'request timed out' service.log

# 모델을 초기화하지 않는 일반 검색
jgrep -E -i 'error|warning' service.log
jgrep -F 'connection refused' service.log
```

### Windows PowerShell

압축을 푼 디렉터리에 `jgrep.exe`가 있다고 가정합니다.

```powershell
.\jgrep.exe --download-model
Get-Content .\service.log | .\jgrep.exe --offline 'connection was reset'
.\jgrep.exe -n -r --include '*.log' 'authentication failed' .\logs
.\jgrep.exe -F 'connection refused' .\service.log
```

`--download-model`에 문맥을 함께 주면 검색 전에 캐시를 준비합니다. 문맥 없이 쓰면 다운로드 후 종료합니다. `--offline`은 네트워크를 절대 사용하지 않으며, 유효한 로컬 모델이 없으면 실패합니다. 두 옵션은 함께 쓸 수 없습니다. `--model /path/to/model.gguf`로 기존 GGUF 파일을 지정할 수 있습니다.

## AI 에이전트용 압축 출력

`--ai`는 코딩 에이전트의 도구 호출을 위해 일치 위치만 `path:line` 형식으로 출력합니다.
일치한 원문 줄, ANSI 색상, 점수, 주변 문맥은 출력하지 않습니다. 기본값은 한 번의 호출
전체에서 최대 50개 위치이며, `--ai-max-results <NUM>`로 이 상한을 바꿀 수 있습니다.
에이전트는 위치를 받은 뒤 별도 읽기 작업으로 필요한 좁은 줄 범위만 가져올 수 있습니다.

## 주요 옵션

| 옵션 | 설명 |
| --- | --- |
| `-e <context>` | 문맥을 추가합니다. 하나라도 일치하면 줄을 선택합니다. |
| `-E`, `-F` | Rust 정규식 또는 고정 문자열 검색을 선택합니다. 둘을 함께 쓸 수 없습니다. |
| `-i`, `-v` | 대소문자 무시(`-E`/`-F`에서만) 또는 최종 선택 반전입니다. |
| `-n`, `-H`, `-h`, `-c`, `-l`, `-L`, `-q`, `-m <NUM>` | 줄 번호, 파일 이름, 개수, 파일 이름만 출력, 조기 종료를 제어합니다. `-m 0`은 모델 작업을 하지 않습니다. |
| `-r`, `--include <GLOB>`, `--exclude <GLOB>` | 결정적인 경로 순서로 재귀 검색하고 대상 경로를 제한합니다. |
| `-A`, `-B`, `-C` | 뒤쪽, 앞쪽 또는 주변 문맥 줄을 함께 출력합니다. |
| `--color <auto\|always\|never>`, `--line-buffered` | ANSI 색상과 스트리밍 파이프의 줄 단위 flush를 제어합니다. |
| `--ai`, `--ai-max-results <NUM>` | 에이전트용 `path:line` 위치 출력과 호출 전체의 최대 결과 수(기본 50)를 제어합니다. |
| `--threshold <0..1>`, `--score` | 의미 검색의 관련성 기준값(기본값 `0.5`)과 선택된 줄의 점수를 제어합니다. |
| `--model <PATH>`, `--download-model`, `--offline`, `--device <auto\|cpu>` | 로컬 모델 파일, 캐시 준비, 네트워크 차단, 추론 장치를 제어합니다. |

의미 전용 옵션은 일반 검색 모드에서 오류가 됩니다. `-`로 시작하는 문맥이나 경로 앞에는 `--`를 둡니다.

## 모델, 개인 정보, 한계

의미 검색은 공식 **Qwen2.5-0.5B-Instruct GGUF Q8_0** 아티팩트(약 676 MB)를 사용합니다. 처음 필요한 의미 검색 또는 `--download-model` 실행 시 모델을 받으며, 고정된 리비전과 SHA-256을 확인한 뒤 사용자 캐시에 원자적으로 저장하고 재사용합니다. `--model`로 지정한 파일은 다운로드하거나 덮어쓰지 않습니다.

선택한 파일의 텍스트는 선택적 모델 다운로드 이후에도 로컬에서 처리됩니다. `--score` 값은 Yes와 No 토큰 로짓의 차이에서 계산한 비보정 관련성 점수이며, 확률이나 정확성 보장이 아닙니다. 모호한 표현, 부정, 여러 언어, 긴 줄, 입력 속의 적대적 내용은 결과에 영향을 줄 수 있습니다. 의미 프롬프트는 4,096 토큰으로 제한되며, 들어가지 않는 줄은 조용히 자르지 않고 오류로 처리합니다.

의미 검색은 일반 문자열 비교보다 모델 다운로드와 추론 시간이 더 들 수 있습니다. 버전이 지정된 소규모 로컬 측정 결과는 [eval/RESULTS-v0.1.0.md](eval/RESULTS-v0.1.0.md)에 공개되어 있지만, 일반적인 정확도·처리량·지연 시간 보장은 제공하지 않습니다. 안전·법률·의료·보안처럼 중요한 판단의 유일한 근거로 사용하면 안 됩니다.

## 입출력과 호환성

`jgrep`은 입력을 점진적으로 읽고 입력 순서대로 선택된 줄을 출력합니다. UTF-8, LF/CRLF 줄 끝, 유니코드 경로를 지원합니다. 재귀 검색은 디렉터리 심볼릭 링크를 따르지 않으며, 그 과정에서 발견한 이진 파일은 진단과 함께 건너뜁니다. 명시적으로 지정한 비텍스트 파일은 오류입니다.

`grep`처럼 일치가 있으면 종료 코드 `0`, 없으면 `1`, 오류면 `2`를 반환합니다. GNU 기본 정규식, PCRE, 역참조, 셸 글롭 인수 확장 및 모든 GNU grep 플래그와의 호환성을 제공하지는 않습니다.

## 라이선스

프로젝트 소스 코드는 **GPL-3.0-or-later**로 배포됩니다. 기본 Qwen 모델은 별도로 내려받는 Apache-2.0 아티팩트이며 프로젝트 소스 코드와 같은 라이선스가 아닙니다. 재배포 전 [LICENSE](LICENSE), [NOTICE](NOTICE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)를 확인하세요.

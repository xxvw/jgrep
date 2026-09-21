# Codex용 jgrep-agent

`jgrep-agent`는 Codex가 코드 위치를 찾을 때 먼저 `jgrep --ai`를 사용하도록 하는
로컬 플러그인입니다. 이 모드는 후보 위치만 반환하므로 검색 도구 출력이 작아집니다.

## 설치

먼저 에이전트 프로세스의 `PATH`에서 `jgrep`를 실행할 수 있도록 설치합니다. 지원
플랫폼, 한 줄 설치, 수동 설치와 오프라인 설치는 프로젝트의
[README](https://github.com/xxvw/localjev-grep/blob/main/README.md) 및
[설치와 에이전트 통합 안내](https://github.com/xxvw/localjev-grep/blob/main/docs/installation-and-agents.md)를 참고하세요.

그런 다음 Codex에 마켓플레이스와 플러그인을 추가하고 새 Codex 세션을 시작합니다.

```sh
codex plugin marketplace add xxvw/localjev-grep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@localjev-grep
```

플러그인을 실행하는 프로세스의 `PATH`에도 `jgrep` 설치 디렉터리가 포함되어야 합니다.

## 사용법

개념이나 동작을 찾을 때는 의미 검색을 사용합니다.

```sh
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  '인증 실패를 처리하는 위치' src/
```

이미 알고 있는 식별자나 고정 문자열에는 모델을 사용하지 않는 `-F`를 씁니다. 정규식이
필요하면 `-E`를 씁니다.

```sh
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/
jgrep --ai --ai-max-results 25 -r -E 'ERROR|WARN' src/
```

출력은 소스 본문이 없는 `path:line` 레코드입니다. Windows 드라이브 문자에도 콜론이
있으므로 **마지막 `:LINE`**(10진수 줄 번호) 접미사를 파싱해야 합니다. 결과 한도에
도달했다는 알림이 나오면 결과가 완전하지 않습니다. 먼저 경로나 조건을 좁히고, 필요한
경우에만 한도를 늘리세요. 그 후 반환된 위치 주변의 좁은 범위만 읽습니다.

`jgrep`가 설치되지 않았거나 이 검색 방식으로 표현할 수 없으면 `rg`를 대신 사용합니다.

```sh
rg -n -F 'validate_session' src/
```

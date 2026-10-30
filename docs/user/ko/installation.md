[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · **한국어** · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# 설치

GitPoulpi에는 실행 파일이 세 개 있습니다:

| 실행 파일 | 설명 |
|---|---|
| `gitpoulpi-tui` | 터미널 UI. |
| `gitpoulpi` | 명령줄 인터페이스(스크립트용 `--json`). 하위 명령 없이 실행하면 UI를 엽니다. |
| `gitpoulpi-mcp` | AI 에이전트용 MCP 서버. [MCP와 에이전트](mcp-and-agents.md)를 참고하세요. |

아카이브에는 이들 옆에 `gitpoulpi-launch`도 들어 있습니다. 바로 가기가 이미 열려 있는 GitPoulpi 창을 보여 주려고 실행하는 아주 작은 프로그램입니다(`gitpoulpi launch`). Windows에서는 콘솔 창이 없어서 클릭해도 창이 깜박이지 않습니다. 나머지와 함께 설치, 업데이트, 제거됩니다.

## 필요한 것

- `PATH`에 있는 **시스템 `git`**. GitPoulpi는 대부분의 작업에 libgit2를, 나머지에는
  사용 중인 `git`을 사용합니다.
  버전 **2.34 이상**이 필요합니다. macOS에서는 Apple Command Line Tools의 `git`도, Homebrew의 `git`도 사용할 수 있습니다.
- 마우스를 지원하는 **터미널**. kitty, Ghostty, WezTerm, iTerm2, sixel을 쓰는 Windows Terminal과 foot은 commit
  그래프를 그림으로 그립니다. 그 밖의 터미널(Windows 콘솔, tmux 및 대부분의
  다른 터미널)은 문자 그래프를 표시합니다. [터미널 고르기](terminals.md)를
  참고하세요.
- 도구 모음와 사이드바의 아이콘을 위한 **Nerd Font** 또는 이모지가 있는 글꼴.
- 선택: Git LFS 저장소를 위한 **`git-lfs`** 3.0 이상. 없으면 LFS 파일이 작은 포인터
  파일로 표시되고 GitPoulpi가 그 이유를 알려 줍니다. 설치 방법은
  [Git LFS](lfs.md)를 참고하세요.
- 선택: 호스팅 토큰과 AI API 키를 저장하기 위한 **credential helper**(Git
  Credential Manager, libsecret, `pass`, Keychain).
  [호스팅과 이슈 트래커](hosting-integration.md)를 참고하세요.

## 릴리스 아카이브에서

릴리스 아카이브는 `gitpoulpi-<version>-<target>.zip`(Windows) 또는
`.tar.gz`(Linux, macOS)이며 옆에 `.sha256` 파일이 있습니다. 아카이브를
`.sha256` 파일과 대조해 확인하고, 압축을 풀고, 실행 파일 세 개를 `PATH`에 있는
폴더에 넣으세요. 아카이브 안의 `INSTALL.txt`에도 같은 단계가 있습니다.

## 설치 프로그램으로

릴리스마다 OS별 설치 프로그램도 제공합니다. 둘 다 같은 선택(바로 가기, PATH,
`gp` 같은 명령 별칭)을 설치된 `gitpoulpi` 자체를 호출해 처리합니다. 설정 > 일반 >
런처의 동작과 같은 코드이므로, 설치 프로그램이 해 둔 일과 설정에 표시되는 내용은
언제나 일치합니다.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

현재 사용자용으로 관리자 권한 요청 없이 `%LOCALAPPDATA%\Programs\GitPoulpi`에
설치합니다(첫 대화상자에서 모든 사용자용 설치와 설치 폴더를 고를 수 있습니다).
체크박스: 시작 메뉴 바로 가기, 바탕 화면 바로 가기, PATH에 추가. 한 페이지에서
**명령 별칭**을 묻습니다. 기본값은 `gp`이고 원하는 이름을 쓸 수 있으며, 비워 두면
별칭을 만들지 않습니다. PowerShell은 이미 `gp`를 `Get-ItemProperty`에 쓰므로
PowerShell에서는 다른 이름을 고르거나 `gitpoulpi`를 쓰세요.

설치 프로그램은 실행 파일 세 개와 `.gitpoulpi-dist` 표시 파일을 복사하므로
[지금 업데이트](#제자리-업데이트)가 zip과 똑같이 동작합니다. 사용자별 설치는 쓰기가
가능해 스스로 업데이트할 수 있지만, Program Files의 모든 사용자용 설치는 교체하려면
권한 상승이 필요하므로 새 설치 프로그램으로 다시 설치하세요. 제거는 Windows 설정 >
앱에서 합니다. `gitpoulpi uninstall`(별칭, PATH 항목, 바로 가기)을 실행한 뒤 업데이트
프로그램이 남긴 `.old` 파일을 포함해 파일을 삭제합니다. 설정과 저장소는 그대로 남습니다.

### Linux와 macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

OS와 CPU에 맞는 아카이브를 고르고 `.sha256`을 확인한 뒤 `~/.local/opt/gitpoulpi`에
설치하고, `gitpoulpi`와 `gitpoulpi-mcp`를 `~/.local/bin`에 링크합니다. root 권한이
필요 없으며 홈 폴더 밖은 건드리지 않습니다. `--yes`가 없으면 별칭(기본값 `gp`),
PATH, 바로 가기에 대해 묻습니다.

| 플래그 | 효과 |
|---|---|
| `--prefix DIR` | 프로그램 파일을 `DIR`(절대 경로)에 설치합니다. |
| `--version X.Y.Z` | 최신 릴리스 대신 이 릴리스를 설치합니다. |
| `--archive FILE` | 로컬 아카이브를 사용합니다. 옆에 `FILE.sha256`이 있어야 합니다(`--no-verify`로 건너뜀). 네트워크를 쓰지 않습니다. |
| `--alias NAME` / `--no-alias` | `NAME`으로 GitPoulpi를 시작하게 하거나(`gitpoulpi alias create`) 별칭을 만들지 않습니다. |
| `--no-path` | 새 셸의 PATH에 GitPoulpi를 추가하지 않습니다(그렇지 않으면 `gitpoulpi path add`). |
| `--desktop` | 응용 프로그램 메뉴와 바탕 화면 바로 가기도 만듭니다. |
| `--yes` | 묻지 않습니다. 플래그와 기본값이 적용됩니다(별칭 `gp`, PATH 예, 바로 가기 아니요). |
| `--uninstall` | `gitpoulpi uninstall`을 실행한 뒤 링크와 폴더를 제거합니다. |

최신 릴리스를 조회하려면 저장소가 공개여야 합니다. 비공개 저장소라면 브라우저에서
아카이브를 내려받아 `--archive`를 쓰세요.
## 소스에서

최신 안정 Rust 툴체인이 필요합니다(<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

실행 파일은 `target/release/`에 있습니다. `release` 프로필은 빠르게 빌드됩니다.
가장 작고 빠른 실행 파일이 필요하면 `dist` 프로필을 사용하세요. 빌드에 더 오래
걸립니다:

```
cargo build --profile dist
```

저장소 루트의 `build-opti` 스크립트는 세 실행 파일에 대해 이 빌드를 실행하고 크기를
출력합니다:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

사용하는 언어와 기능만 빌드하려면 [사용자 지정 빌드](custom-builds.md)를
참고하세요.

## 릴리스 아카이브 직접 빌드하기

```
cargo run -p xtask -- dist
```

`dist` 프로필을 빌드하고 `target/dist/gitpoulpi-<version>-<target>.zip`(Windows)
또는 `.tar.gz`(다른 시스템)를 만듭니다. 아카이브에는 실행 파일 세 개,
`INSTALL.txt`, `README.md`, `LICENSE`, `CHANGELOG.md`가 들어 있습니다. 옆에
`.sha256` 파일도 만들어집니다.

## 업데이트

제목 표시줄의 앱 이름과 저장소 경로 사이에 실행 중인 버전(`v1.0.0`)이 표시됩니다. 이를 클릭하거나
명령 팔레트(`Ctrl+P`)에서 **도움말: 릴리스 노트**를 실행하면 변경 내역이 실행 중인
버전 위치로 스크롤된 릴리스 노트 탭이 열립니다.

GitPoulpi는 하루에 한 번 백그라운드에서 GitHub에 새 릴리스가 있는지 묻습니다
(`updates.mode`, [설정 > 동기화](settings.md#동기화)). 새 릴리스가 있으면 칩이 강조색으로
`v0.2.0 사용 가능`이라고 표시되고, 클릭하면 그 릴리스의 노트가 맨 위에 있는 릴리스 노트
탭이 열립니다. 탭에는 버튼 세 개가 있으며 각각 키와 팔레트 명령이 있습니다:

- **다운로드**(`d`, *업데이트: 다운로드*)는 브라우저에서 릴리스 페이지를 엽니다.
- **이 버전 건너뛰기**(`s`, *업데이트: 이 버전 건너뛰기*)는 더 새로운 릴리스가 나올
  때까지 알림을 숨깁니다.
- **지금 확인**(`r`, *업데이트: 지금 업데이트 확인*)은 즉시 다시 확인합니다.

`updates.mode = "off"`이면 요청을 전혀 보내지 않습니다. 네트워크에 연결되어 있지 않거나
저장소가 비공개이고 github.com 토큰이 저장되어 있지 않으면 확인은 조용히 실패합니다
(애플리케이션 로그에 한 줄, 인터페이스에는 아무것도 표시되지 않음).
`GITPOULPI_UPDATE_API`는 확인 요청을 다른 API 루트로 보냅니다(테스트에서 사용).

### 제자리 업데이트

릴리스 아카이브로 설치한 빌드는 스스로를 교체할 수 있습니다. 아카이브에는 실행 파일 옆에
작은 `.gitpoulpi-dist` 파일이 들어 있으며, 실행 중인 빌드는 그 파일이 실행 파일 옆에
있고 자신의 플랫폼을 가리킬 때만 "업데이트 가능"입니다. `cargo`로 만든 빌드, 다른 곳에
복사한 사용자 지정 빌드, 그 파일을 빼 버린 패키지는 스스로 업데이트하지 않습니다. 이런
빌드의 릴리스 노트 탭은 그 사실을 알려 주고 **다운로드**만 제공합니다.

업데이트 가능한 빌드에는 탭에 다음이 더해집니다:

- **지금 업데이트**(`u`, *업데이트: 지금 업데이트*)는 백그라운드에서 아카이브와 그
  `.sha256`을 내려받고(상태 표시줄 표시기의 작업으로 퍼센트가 표시되며 다른 작업처럼
  취소할 수 있음), 해시를 확인하고, 아카이브를 풀어 각 새 파일을 이전 파일 옆에
  `<name>.new`로 둡니다. 실행 중인 것은 건드리지 않습니다. `auto` 모드에서는 확인에서
  새 릴리스가 발견되는 즉시 같은 일이 저절로 일어납니다.
- **재시작하여 업데이트**(`R`, *업데이트: 재시작하여 업데이트*. 다운로드가 준비되면 칩에도
  같은 문구가 표시됨)는 언제나 사용자가 직접 누르며 자동으로 실행되지 않습니다. 세션의
  독립 데몬을 중지하고(내장 데몬은 창과 함께 종료됨) 파일을 교체한 뒤 같은 인수와 세션으로
  새 버전을 시작합니다.
- **이전 버전으로 되돌리기**(`v`, *업데이트: 이전 버전으로 되돌리기*)는 이전 파일로 되돌리고
  재시작합니다. 이전 버전의 파일은 새 버전이 1분 동안 실행될 때까지 `<name>.old`로
  남았다가 삭제되므로, 이 버튼은 그 첫 실행 동안만 있습니다. git 실행 취소 기록에는
  포함되지 않습니다. 저장소가 아니라 GitPoulpi 자체의 파일을 바꾸기 때문입니다.

파일을 교체하는 방식: Linux와 macOS에서는 각 새 파일을 이전 파일 위로 이름 변경합니다
(원자적이며, 실행 중인 프로그램은 종료할 때까지 이전 사본을 유지함). Windows에서는 실행 중인
`.exe`를 덮어쓸 수 없으므로 이전 파일을 `<name>.old`로 이름 바꾸고 새 파일을 옮겨 넣으며,
아직 실행 중인 `.old`는 이후 시작 때 삭제됩니다. 교체는 전부 아니면 전무입니다. 다른
프로그램이 실행 파일 중 하나를 붙잡고 있으면(예: `gitpoulpi-mcp`를 실행 중인 MCP 클라이언트)
GitPoulpi는 그 파일 이름을 알려 주고, 아무것도 바꾸지 않으며, 다시 시도할 수 있게 다운로드를
준비된 채로 둡니다.

다음은 각각 메시지와 함께 거부됩니다: `.sha256` 파일이 없는 릴리스, 일치하지 않는 해시, 이
플랫폼용이 아닌 이름의 아카이브 파일(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), 표시 파일이
다른 버전이나 플랫폼을 가리키는 아카이브, 그리고 다음 호스트 밖의 모든 다운로드 링크:
GitHub의 `github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com`, `github-releases.githubusercontent.com`(첫 요청과 모든
리디렉션 단계이며, 다른 곳으로 리디렉션되면 다운로드가 중단됨). 비공개 저장소의 아카이브는
브라우저 세션이 필요하므로, 저장소가 공개될 때까지 그곳에서는 다운로드가 실패합니다.
## 잘 동작하는지 확인하기

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version`은 버전과 빌드에 사용된 commit을 출력합니다. 그런 다음
[시작하기](getting-started.md)를 따르세요.

## 제거

`gitpoulpi uninstall`(두 설치 프로그램이 실행하는 것)은 GitPoulpi가 만든 명령 별칭, PATH 항목, 바로 가기를 제거한 뒤 릴리스 설치라면 프로그램 파일도 제거합니다. `.gitpoulpi-dist` 표시가 없는 빌드(cargo 빌드)에서는 거부하며 `--keep-files`는 프로그램 파일을 남겨 둡니다. MCP 에이전트는 사용할 수 없습니다. 설치 프로그램 없이 설치했다면 실행 파일 세 개를 삭제하세요. GitPoulpi는 다음 파일도 보관합니다:

| 항목 | 위치 |
|---|---|
| 설정(`config.toml`)과 세션 | OS의 구성 폴더. `GITPOULPI_CONFIG_DIR`로 바꿀 수 있습니다. |
| 저장소별 상태 | 기본적으로 저장소의 `.git` 폴더 안 `gitpoulpi/`(`session.per_repo_state_dir`로 작업 트리의 `.gitpoulpi/`나 사용자 데이터 폴더로 옮길 수 있음). |
| 캐시(아바타) | OS의 캐시 폴더. |
| 토큰과 API 키 | credential helper. **설정 > SSH 및 자격 증명 > 모든 사용자 이름과 비밀번호 삭제**로 제거합니다. |

정확한 경로는 [설정 참조](config-reference.md)를 참고하세요.

[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · [Français](../fr/open-in.md) · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · **한국어** · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# 열기 위치

제목 표시줄에서 저장소 경로 옆에 있는 **열기 위치 ▾** 버튼은 활성 탭의 폴더를 다른 곳에서 엽니다. 터미널, 새 멀티플렉서 탭이나 창, 편집기, 파일 관리자 등입니다. 목록에 무엇을 둘지는 사용자가 정합니다. 고정된 것은 없고, 모든 항목은 레이블과 명령으로 이루어집니다.

## 사용 방법

- **열기 위치 ▾ 클릭**: 목록이 펼쳐집니다. 항목을 클릭하면 실행됩니다. 기본 항목은 `•`로 표시되며, 마지막 행인 **설정...**은 [설정](settings.md#열기-위치)의 열기 위치 페이지를 엽니다.

  ![열기 위치 목록: 터미널, 편집기 두 개, Git 도구, 파일 관리자가 있고 기본 항목에는 점이 표시되며 맨 끝에 설정이 있음](../images/open-in-menu.png)

- **키보드**: `Alt+O`가 같은 목록을 엽니다(`Up`/`Down`, `Enter`, `Esc`, 항목이 일곱 개를 넘으면 입력해서 검색). `Ctrl+Alt+O`는 기본 항목을 바로 실행합니다. 팔레트(`Ctrl+P`): `탭: 열기 위치...`와 `탭: 기본 위치에서 열기`. 제목 표시줄에 버튼을 놓을 공간이 없어도 팔레트 명령은 동작합니다. 두 키 모두 다시 할당할 수 있습니다. [단축키](keybindings.md)를 참고하세요.
- **저장소 탭을 마우스 오른쪽 버튼으로 클릭**: 탭 메뉴에 같은 항목이 있는 **열기 위치** 하위 메뉴가 있습니다.
- 목록은 이번 세션에서 마지막으로 사용한 항목에서 시작합니다.
- 상태 표시줄에는 곧바로 "...에서 여는 중..."이 표시됩니다. 프로그램이 약 1.5초 안에 실패하면(예: 서버가 없는 tmux, 잘못된 옵션) 오류 출력의 첫 줄이 상태 표시줄에 나타납니다. 설치되지 않은 프로그램은 즉시 이름이 표시됩니다.

직접 만든 목록을 저장하기 전까지는 감지된 환경에서 기본 항목이 정해집니다. herdr 안에서는 "Herdr: new tab", Omarchy에서는 kitty의 새 창(kitty가 그 터미널입니다), 그 밖에는 파일 관리자입니다.

이 버튼은 GitPoulpi를 시작한 폴더를 포함해 모든 저장소 탭에 표시되며 `ui.tab_dir_follow = "auto"` 모드에서도 표시됩니다. `auto` 모드에서는 새 탭을 열 때 기본 항목도 저절로 실행됩니다(시작할 때 복원되는 탭은 제외). [활성 탭의 폴더 따라가기](following-the-tab-folder.md)를 참고하세요.

## 항목의 모양

`config.toml`에서는 항목마다 `[[ui.open_in]]` 테이블이 하나씩 있습니다.

```toml
[ui]
open_in_default = "Herdr: new tab"   # a label; empty means the first entry

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

직접 쓸 일은 드뭅니다. 설정 페이지에서 편집하기 때문입니다. 모든 키는 [설정 참조](config-reference.md)에 있습니다.

### 명령과 변수

명령은 먼저 인수로 분할되고(공백, `'...'`, `"..."`로 단어를 묶음), 그다음 각 인수 안의 변수가 치환됩니다. 따라서 이름에 공백이 있는 폴더도 하나의 인수로 유지됩니다. 명령이 직접 셸을 시작하지 않는 한 **셸은 관여하지 않습니다**.

| 변수 | 치환되는 값 |
|---|---|
| `${cwd}` | 탭의 폴더. |
| `${repo}` | 저장소 루트(워크트리 탭에서는 `${cwd}`와 다름). |
| `${name}` | 폴더의 기본 이름. |
| `${branch}` | 체크아웃된 브랜치. detached 상태이면 비어 있음. |
| `${web_url}` | origin 리모트가 GitHub 또는 GitLab일 때 그 웹 페이지. 그렇지 않으면 비어 있음. |
| `${env:NAME}` | 환경 변수 `NAME`. 설정되지 않았으면 비어 있음. |

예전 `{repo}`도 계속 동작하며 `${cwd}`를 의미합니다. 그 밖의 `${...}`는 변수 이름을 알려 주는 오류입니다.

프로그램은 탭의 폴더를 작업 디렉터리로 하여 시작되며, 셸 스니펫에 유용한 환경 변수 세 개를 받습니다.

| 변수 | 값 |
|---|---|
| `GITPOULPI_CWD` | 탭의 폴더. |
| `GITPOULPI_REPO` | 저장소 루트. |
| `GITPOULPI_BRANCH` | 브랜치. 없으면 비어 있음. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

Windows에서는 프로그램을 `PATH`와 `PATHEXT`로 찾으므로 `code`와 `cursor`(`.cmd` 파일)도 동작하며 콘솔 창이 깜박이지 않습니다.

## 프리셋

설정 > 열기 위치 > **프리셋 추가**에는 사용 중인 운영 체제용으로 미리 만들어 둔 항목만 카테고리별로 나열됩니다(Windows 프리셋은 Linux에 표시되지 않는 식입니다). 행은 카테고리로 시작합니다: `탐색기: ...`, `멀티플렉서: ...`, `터미널: ...`, `편집기: ...`, `Git 도구: ...`, `AI 에이전트: ...`, `웹: ...`. 맨 위의 **감지됨**은 계산된 기본 목록의 항목입니다. 각 카테고리 안에서는 바로 사용할 수 있는 프리셋(그 멀티플렉서나 터미널 안에서 실행 중이고 프로그램이 `PATH`에 있는 경우)이 먼저, 그다음 설치된 것, 그다음 나머지 순입니다. 콤보박스에 입력해서 검색할 수 있습니다. 예를 들어 `lazygit`이나 `Git 도구:`를 입력합니다. **현재 탭에서 테스트**로 항목을 쓰기 전에 미리 시험해 볼 수 있습니다.

![열기 위치 페이지의 '+ 추가' 목록: 감지된 항목이 먼저, 이어서 탐색기, 멀티플렉서 등의 카테고리가 나오며 각 프리셋 앞에 카테고리가 붙음](../images/settings-open-in-presets.png)

"창에서 도구 실행" 프리셋(`lazygit in Herdr tab`, `yazi in tmux window`, `Claude Code in Herdr split` 등)은 호스트(herdr, tmux, zellij, WezTerm, kitty, `xdg-terminal-exec`, Windows Terminal)와 도구가 모두 설치되어 있을 때만 제공됩니다. `$EDITOR in ...`은 `EDITOR` 변수에 지정된 편집기(herdr에서는 `vi`로 대체)를 herdr나 tmux에서 실행합니다.

Herdr의 도구 프리셋은 탭이나 분할을 만든 다음 그 셸에 명령을 입력하므로, 도구가 해당 폴더에서 시작되고 도구가 종료된 뒤에도 창이 열려 있습니다. `AI 에이전트` 프리셋은 `herdr agent start`를 사용하므로 herdr가 에이전트의 상태를 추적합니다.

| 카테고리 | 프리셋 (`V` = 실행해 보았거나 설치된 도움말과 대조해 확인함) |
|---|---|
| 탐색기 | Windows `explorer.exe` (V), macOS Finder (V), Linux `xdg-open` (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM; 창/탭/터미널에서 yazi, superfile, nnn, lf, ranger |
| 멀티플렉서 | herdr 탭, 워크스페이스, 오른쪽/아래 분할 (V); tmux 창, 오른쪽/아래 분할, 분리된 세션 (V); zellij 창, 탭, 플로팅; WezTerm 탭, 분할, 새 창; kitty 탭, 분할 창, OS 창 (V: 도움말의 옵션 기준, `allow_remote_control` 필요); screen; cmux; psmux |
| 터미널 | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp; Windows: Windows Terminal (창, 탭, 분할, 그리고 PowerShell 7, Windows PowerShell, 명령 프롬프트, WSL 각각의 탭), PowerShell 7 / Windows PowerShell / 명령 프롬프트 / WSL 창, Git Bash |
| 편집기 | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows); Neovim, Vim, Helix, 창/탭/터미널에서 `$EDITOR` |
| Git 도구 | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop; 창/탭/터미널에서 lazygit, tig, gitui (herdr, tmux, kitty, `xdg-terminal-exec` 용은 V) |
| AI 에이전트 | herdr 탭/분할 (V, Claude 기준), tmux, zellij, WezTerm, kitty, 새 터미널에서 Claude Code, Codex, Gemini CLI, opencode |
| 웹 | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

Windows의 WSL: `wsl.exe --cd ${cwd}`는 Windows 경로를 그대로 받으며, Windows Terminal의 `wsl.exe` 탭은 해당 폴더에서 시작됩니다. WSL은 이 폴더를 `/mnt/` 아래에 매핑합니다.

탐색기, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual Studio, Zed(Zed는 Unix 전용)는 존재하는 환경에서만 나타납니다. tmux, zellij, cmux, screen, kitty 원격 제어, `sh`로 구동되는 herdr 프리셋은 Unix 전용입니다. herdr(일반 프리셋), WezTerm, Alacritty, VS Code, Cursor, JetBrains IDE는 어디서나 제공됩니다.

### 변수 `${web_url}`

저장소의 `origin` 리모트가 GitHub 또는 GitLab 프로젝트(github.com, gitlab.com, 또는 `[[integrations]]`에 항목이 있는 자체 관리 호스트)일 때 그 웹 페이지입니다. 그렇지 않으면 비어 있고, 이 변수를 쓰는 항목은 빈 주소를 여는 대신 메시지와 함께 중단됩니다. 선택 사항입니다. `Repository page on the web` 외에는 이 변수가 필요한 프리셋이 없습니다.

## 설정 페이지

설정 > **열기 위치**에서 목록을 편집합니다. 항목은 왼쪽의 [목록](settings.md#목록)이며(기본 항목에는 `★`가 붙음), 선택한 항목의 필드는 오른쪽에 있습니다. `+ 추가`, `복제`, `제거`, `↑` / `↓`, 행 드래그로 목록을 관리합니다.

| 행 | 기능 |
|---|---|
| **기본 항목** | **기본 위치에서 열기**가 실행하는 항목. 열기 위치 목록에서 `•`로 표시됩니다. |
| **레이블** | 목록에 표시되는 이름. 고유해야 하며 비워 둘 수 없습니다. |
| **명령** | 명령과, 삽입할 변수 칩(`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`). |
| **현재 탭에서 테스트** | 항목을 지금 활성 탭의 폴더에서 실행합니다. |
| **+ 추가 ▾** | 미리 만들어 둔 항목을 추가합니다(감지된 항목이 먼저 나옵니다). **사용자 지정**은 빈 항목을 추가하고 그 명령으로 이동합니다. |
| **감지된 기본값 복원** | 목록을 이 컴퓨터에서 감지된 항목으로 바꾸고 기본값을 지웁니다. |

편집 내용은 만드는 즉시 저장됩니다. 첫 편집에서 `[[ui.open_in]]` 배열 전체를 `config.toml`에 기록합니다. 그 전까지 목록은 계산된 것이며([예전 키](#예전-키) 참고) 파일에는 `open_in` 키가 없습니다. 설정 검색에서도 이 페이지를 찾을 수 있습니다.

## 바이너리 파일: 열기

바이너리 파일(PDF, Word 문서, 스프레드시트, 압축 파일 등)은 Diff 보기나 파일 보기에서 보여 줄 텍스트가 없습니다. 운영 체제가 해당 형식의 앱을 알고 있으면 보기에 평소 안내 옆에 **열기** 버튼이 나타납니다. 목록에는 시스템이 그 형식을 여는 앱이 점으로 표시되어 맨 위에 오고, 이어서 시스템이 그 형식에 등록한 다른 모든 앱(시스템 자체의 *연결 프로그램* 목록)이 나옵니다. Linux에서는 파일 이름의 MIME 형식을 기준으로 기본 앱은 `mimeapps.list`, 나머지는 `.desktop` 파일에서 가져옵니다. Windows에서는 레지스트리(사용자의 선택과 확장자에 등록된 프로그램), macOS에서는 기본 앱만 사용합니다.

`o`는 기본 앱을 실행하고 `Shift+O`는 목록을 엽니다. 둘 다 명령 팔레트(*파일: 기본 앱으로 열기*, *파일: 연결 프로그램...*)에 있고, **연결 프로그램...**은 Status 창의 파일과 commit의 파일 목록에서 오른쪽 클릭 메뉴에도 있습니다. 검색은 확장자마다 한 번씩 백그라운드에서 실행되며, 그동안 보기에는 "앱 찾는 중..."이 표시됩니다. 시스템이 앱을 모르는 형식과 일반 텍스트는 기존 안내를 그대로 유지하고 버튼이 없습니다.

![파일 보기의 PDF와 Open in 목록: 점으로 표시된 기본 앱이 맨 위에, 그 아래에 시스템이 해당 형식에 등록한 다른 앱](../images/open-with.png)

작업 트리의 파일은 그대로 앱에 전달됩니다. commit(또는 인덱스)의 버전은 **읽기 전용 임시 사본**으로 기록되고 상태 표시줄에 안내가 표시되므로 앱이 저장소를 바꿀 수 없습니다. 둘 다 저장소를 바꾸지 않습니다. 저널 항목이 없고 실행 취소에도 영향이 없습니다.

메뉴의 목록은 메뉴 옆에 **연결 프로그램...** 행과 같은 높이로, 하위 메뉴처럼 열립니다(오른쪽에 공간이 없으면 왼쪽). 마우스로도 키보드로도 같습니다. 파일 메뉴에서 연 이미지 편집기 목록(**편집 프로그램**)도 마찬가지입니다.

### Git LFS 파일

Git LFS 객체 카드([Git LFS](lfs.md) 참고)는 텍스트 아래에 같은 두 동작을 둡니다. **기본 프로그램으로 열기**와 **열기**이며, 키(`o`, `Shift+O`)와 팔레트 행도 같습니다. 이 동작은 포인터가 아니라 실제 파일을 엽니다. 작업 트리의 파일은 그대로, commit이나 인덱스의 버전은 로컬 LFS 저장소에 있는 객체의 읽기 전용 임시 사본을 엽니다. 아직 다운로드되지 않은 객체는 카드에 버튼이 없고, 상태 표시줄이 내용이 아직 다운로드되지 않았다고 알립니다. 포인터만 들어 있는 작업 트리 파일에도 같은 거부가 적용됩니다. 카드든 파일 메뉴든 **기본 프로그램으로 열기**, **열기**, **편집 프로그램**은 포인터 텍스트를 여는 대신 내용이 다운로드되지 않았다고 알립니다(LFS 도구 모음의 pull이나 체크아웃 행으로 내려받은 뒤 다시 시도하세요). 열기는 읽기 전용이라 저널 항목이 없고 실행 취소할 것도 없습니다.

### Windows의 브라우저

Chrome과 Edge는 `--single-argument %1`을 등록하므로 스위치 뒤의 모든 것을 따옴표까지 포함해 하나의 주소로 읽습니다. GitPoulpi는 이런 프로그램에 경로를 Windows처럼 그대로, 따옴표 없이 전달하므로 경로에 공백이 있는 파일도 `http://"c/...`로 실패하지 않고 열립니다. 다른 모든 프로그램은 경로를 하나의 일반 인자로 받습니다.

## 예전 키

예전 키 `ui.terminal_launch`와 `ui.terminal_launch_command`는 `ui.open_in`이 없을 때만 읽습니다. 그 경우 계산되는 목록은 다음과 같습니다.

1. 예전 터미널 선택. `auto`가 아닌 경우입니다(`custom`은 `{repo}`를 `${cwd}`로 바꾼 "Terminal (custom)"이 되고, 프리셋 id는 그 프리셋이 됩니다). `auto` 값은 첫 항목인 "Default terminal"이 됩니다.
2. OS의 파일 탐색기,
3. 환경에서 감지된 멀티플렉서와 터미널,
4. `PATH`에서 찾은 편집기.

설정에서 목록을 저장하면 `ui.open_in`이 우선하고 예전 키는 무시됩니다. 팔레트 명령 **탭: 터미널에서 열기**와 **탭: 파일 관리자에서 열기**는 이 목록을 사용하지 않습니다.

## 문제 해결

- *"program X was not found"*: 그 프로그램이 GitPoulpi가 보는 `PATH`에 없습니다. 해당 프로그램이 있는 셸에서 GitPoulpi를 시작하거나 전체 경로를 쓰세요.
- *tmux/zellij/herdr 항목이 즉시 실패함*: 이들은 실행 중인 서버와 통신하므로 GitPoulpi가 그 안에서 실행될 때만 동작합니다.
- *kitty 탭 항목이 실패함*: `kitty.conf`에 `allow_remote_control`이 필요합니다.
- *Windows 탐색기*: `explorer.exe`는 성공해도 종료 코드 1로 끝납니다. GitPoulpi는 이를 성공으로 처리합니다.

[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · **한국어** · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# 터미널 고르기

GitPoulpi는 마우스를 지원하는 어떤 터미널에서든 실행되지만, 그림을 그리고, 트루컬러를
표시하고, 키보드 입력을 빠짐없이 보내 주는 터미널에서 가장 보기 좋고 쓰기 편합니다.
이 페이지는 OS별로 각 터미널이 무엇을 제공하는지, 어떤 설정을 바꿔야 하는지 설명합니다.

**읽는 방법.** 표는 **2026년 11월**의 시험 결과입니다. 모든 터미널에 원시 이스케이프
시퀀스를 보냈고, 실행할 수 있는 곳에서는 GitPoulpi 자체도 Linux, Windows 11, macOS 15에서
실행했습니다. 버전은 [아래](#시험한-버전)에 있습니다. 각 칸은 다음 중 하나입니다.

- **동작**
- **부분 동작** (무엇이 빠졌는지 함께 적음)
- **동작 안 함** (이유: *터미널*, *OS 계층*, 또는 *GitPoulpi*)
- **해당 없음** (그 OS에 그 터미널이 없음)
- **측정 안 함** (이유와 함께)

멀티플렉서(tmux, zellij, screen, herdr, psmux, 터미널 창)는 자체 페이지가 있습니다.
[멀티플렉서](multiplexers.md)를 참고하세요.

## 추천 터미널

| OS | 추천 순서 | 이유 |
|---|---|---|
| **Windows** | **Windows Terminal**, 그다음 **WezTerm**과 **Rio** | Windows Terminal은 그래프와 아바타를 sixel 그림으로 올바른 위치에 그리며 런처의 기본값입니다. WezTerm과 Rio는 이를 iTerm2 그림으로 그리며, 2026년 11월의 ConPTY 수정 이후 올바른 위치에 놓입니다([Windows](#windows) 참고). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty와 Ghostty는 그림, 포인터 모양, 완전한 키보드 입력을 제공합니다. foot은 sixel 그림을 그리며 키보드 입력이 완전하고 포인터 모양도 있습니다. |
| **macOS** | **iTerm2**, 그다음 **kitty**와 **Ghostty** | iTerm2 3.7.7은 kitty 프로토콜로 그림을 그립니다. Terminal.app도 동작하지만 그림도 트루컬러도 없습니다. kitty와 Ghostty는 Linux와 같게 동작합니다(macOS에서는 측정 안 함). |

어떤 OS에서든 **screen**과 **zellij**는 터미널의 그림 프로토콜을 GitPoulpi에게서 가리므로
문자로 그립니다. tmux는 sixel을 통과시킵니다. [멀티플렉서](multiplexers.md)를 참고하세요.

## GitPoulpi가 사용하는 것

- **그림**(그래프 열, 작성자 아바타, 이미지 diff와 파일 보기): 유니코드 자리 표시자를
  쓰는 **kitty 그래픽 프로토콜**, **iTerm2 인라인 이미지**(`OSC 1337`), 또는 **sixel**.
  감지는 기본적으로 거부하는 방식이며 `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION`,
  멀티플렉서 변수를 보고 판단합니다. 환경이 어떤 터미널인지 알려 주지 않으면
  (`-ti vt340`으로 시작한 xterm, 이미지를 켠 VS Code 터미널, `TERM_PROGRAM`이 전달되지
  않는 SSH 너머의 모든 터미널) 터미널 자신의 응답이 결정합니다. GitPoulpi는 시작할 때
  한 번 묻고(아래 "시작 시 질의" 참고) 터미널의 XTVERSION 이름(kitty, Ghostty,
  WezTerm, iTerm2, mintty, foot) 또는 장치 속성의 sixel을 사용합니다. 멀티플렉서 안에서는
  응답을 무시합니다. tmux는 뒤에 무엇이 있든 sixel을 지원한다고 답하기 때문입니다.
- **어디서 어떤 프로토콜인가.** kitty, Ghostty, iTerm2 3.7.7 이상: kitty 프로토콜.
  WezTerm과 Rio: iTerm2 프로토콜이며 PNG의 투명도를 섞습니다. mintty와 3.7.7보다 오래된
  iTerm2: 불투명한 배경이 있는 iTerm2 프로토콜. Windows Terminal, foot, Tabby, Konsole,
  mlterm: **sixel**. 그 밖의 모든 곳: 문자이며 아바타는 이니셜 배지가 됩니다.
  `ui.image_protocol`로 하나를 강제할 수 있습니다.
- **iTerm2 3.7.7.** 모든 `OSC 1337` 인라인 이미지는 "Remember my choice"를 선택할 때까지
  창을 막는 모달 대화상자 "Allow Terminal-Initiated Display?"를 엽니다. 같은 그림을 kitty
  프로토콜(유니코드 자리 표시자 포함)로 보내면 질문 없이 그려지므로 GitPoulpi는 거기서
  kitty 프로토콜을 씁니다. `ui.image_protocol = "iterm2"`로 인라인 이미지를 쓸 수도 있습니다.
- **Sixel**은 **그래프 열**(레인, 노드, 그리고 `ui.avatars = "graph"`이면 commit 원 안의
  아바타)과 그 밖의 **아바타**(Log, commit 상세, 파일 기록, Blame, 이미지 보기)에 그려집니다.
  sixel에는 알파도 ID도 크기 조절도 없으므로 그림은 정확히 `열 수 x 칸 너비` 곱하기
  `행 수 x 칸 높이` 픽셀이며, 무언가 있는 곳에만 배경이 유지되는 빈 칸 위에 그립니다.
  레인, 스크롤 위치, 확대 비율이 바뀔 때만 다시 인코딩합니다. sixel에는 삭제 기능이 없으므로
  GitPoulpi는 그 칸 위에 텍스트를 덮어써서 지웁니다. sixel에는 칸의 실제 픽셀 크기가
  필요합니다. Windows Terminal, Tabby, mintty는 시작할 때와 확대 후에 `CSI 16 t`에 한 번
  답합니다(응답이 없어도 막히지 않습니다). 다른 곳에서는 터미널의 창 크기나
  `GITPOULPI_TERMINAL_CELL_PX=10x20`에서 얻습니다. 말없이 sixel을 그리는 터미널에는
  `ui.image_protocol = "sixel"`이 필요합니다. 1.22 이전의 Windows Terminal은 sixel을 그리지
  못하므로 거기서는 `ui.image_protocol = "none"`으로 설정하세요.
- **트루컬러**: 테마는 RGB입니다. `ui.color_depth = "auto"`는 `COLORTERM=truecolor` 없이
  `*-256color`인 `TERM`(그리고 트루컬러가 없는 Terminal.app)에서는 256색으로, Linux
  콘솔에서는 16색으로 낮춥니다.
- **마우스**: 버튼, 드래그, 이동을 포함한 SGR 마우스 보고(호버 피드백, 끌어서 놓기,
  오른쪽 클릭, 더블 클릭)입니다.
- **시작 시 질의.** 질문 네 개를 한 번의 쓰기로 보냅니다. kitty 키보드 플래그(`CSI ? u`),
  배경색(`OSC 11`, `ui.theme = system`에서만 사용), 터미널 이름(XTVERSION, `CSI > 0 q`),
  장치 속성(`CSI c`, 마지막: 모든 터미널이 답하므로 읽기를 마무리합니다). 읽기는 100 ms
  제한의 `poll` 루프 하나이며 키보드 리더가 시작되기 전에 끝납니다. 아무것도 답하지 않는
  터미널은 100 ms를 한 번 소비하고 나머지는 환경이 알려 주는 대로 둡니다. Unix 전용입니다.
  Windows에서는 콘솔이 키를 직접 보고하고 환경이 결정합니다. 응답은 활동 로그의 환경
  보고서(`start-up query:` 줄)에 있습니다.
- **키보드**: **kitty 키보드 프로토콜**(disambiguate 플래그만)을 시작할 때 요청합니다.
  이로써 `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`, `Ctrl+Shift+Z`가 전달됩니다. 없으면 기존
  키가 남습니다(`Ctrl+Y`가 다시 실행하고, 일부 단축키는 `Alt` 변형으로 대체됩니다).
  [단축키](keybindings.md#터미널이-보낼-수-없는-키와-kitty-키보드-프로토콜)를 참고하세요.
- **클립보드**: 모든 복사 동작은 하나의 정책을 따릅니다. 로컬에서는 GitPoulpi가 실행되는
  머신의 **시스템 클립보드**에 넣고, 터미널이 `OSC 52`를 지원하는 것으로 알려진 경우
  (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio, Tabby, iTerm2)에는
  터미널로도 `OSC 52`를 보냅니다. 시스템 클립보드가 실패하면 `OSC 52`가 대체 수단입니다.
  **SSH를 통하거나 디스플레이가 없으면**(Linux 콘솔, 컨테이너) `OSC 52`만 보내며, tmux
  안에서는 그대로의 형태와 통과용으로 감싼 형태를 모두 보냅니다. 상태 표시줄은 시스템
  클립보드가 텍스트를 받았을 때만 "복사했습니다"라고 말합니다. 터미널로 보낸 것은 터미널이
  따랐는지 확인할 방법이 없으므로 "터미널 클립보드로 보냈습니다 (OSC 52, 확인되지 않음)"라고
  말합니다. 활동 로그의 환경 보고서가 `clipboard:` 줄에 경로를 알려 줍니다.
- **마우스 포인터 모양**(구분선 위의 크기 조절 포인터, 클릭할 수 있는 항목 위의 손, 텍스트
  필드 위의 I-beam): `OSC 22`이며, 구현한 터미널에서는 덤으로 제공됩니다. GitPoulpi는 항상
  보냅니다(Linux 콘솔과 이를 삼키거나 그대로 출력하는 screen, zellij, herdr 뒤는 제외하며,
  tmux 뒤에서는 통과용으로 감쌉니다). `TERM_PROGRAM=mintty`에서는 mintty 자체의 커서
  이름(`hand`, `ibeam`, `sizewe`, ...)을 보내며, mintty는 이것만 읽습니다. **Rio는 어떤
  OS에서도 `OSC 22`를 구현하지 않습니다**. 소스가 시퀀스를 해석한 뒤 버리므로 어느 쪽의
  설정으로도 바뀌지 않습니다.
- **텍스트 캐럿**: `ui.text_cursor = "terminal"`(기본값)이면 터미널 자체의 커서를 포커스된
  텍스트 필드의 캐럿 위치에 깜박이는 막대(`DECSCUSR`)로 놓고 다른 곳에서는 숨기며, 종료하면
  사용자의 커서 모양이 돌아옵니다. `drawn`은 대신 GitPoulpi 자체의 캐럿을 그립니다.
- **포커스 이벤트**를 켭니다(창이 포커스를 잃으면 툴팁이 닫힙니다). **Bracketed paste**는
  시작할 때 요청하고 모든 종료 경로에서 해제하므로(Unix) 여러 줄 붙여넣기가 하나의 붙여넣기로
  도착합니다. 줄바꿈이 필드를 제출하는 일도, 글자가 단축키를 실행하는 일도 없습니다.
  Windows 콘솔에는 붙여넣기 괄호가 없으므로 GitPoulpi가 대신 하는 일은
  [Windows](#windows)를 참고하세요. GitPoulpi는 OSC 8 하이퍼링크와 undercurl을 요청하지
  않습니다. 그림 경로에서만 각 프레임을 동기화된 갱신(모드 2026, 모르는 터미널은 무시함)으로
  감싸므로 팝업 위에서 그림이 깜박이지 않습니다.
- **이모지와 기호**: 터미널과 글꼴이 변형 선택자에 대해 서로 다르게 동작하므로
  GitPoulpi는 기본 이모지 표현을 가진 글리프나 너비가 1인 BMP 도형만 씁니다.

## 기능 표

행은 터미널, 열은 OS입니다. 칸의 "GitPoulpi"는 GitPoulpi 자체를 거기서 실행했다는 뜻이고,
"probe"는 원시 이스케이프 시퀀스만 보냈다는 뜻입니다. **해당 없음** 칸: kitty에는 네이티브
Windows 빌드가 없고(릴리스는 Linux와 macOS), Ghostty에는 Windows 빌드가 없으며(libghostty
라이브러리만 크로스 플랫폼), foot은 Wayland 터미널이고(Linux 전용), xterm은 X11 클라이언트이며
(Linux, 그리고 XQuartz를 통한 macOS), iTerm2와 Terminal.app은 macOS 전용이고, Windows Terminal,
conhost, mintty는 Windows 전용입니다(mintty는 Cygwin과 MSYS2에서도 쓰입니다).

**macOS에서 측정 안 함: kitty, Ghostty, WezTerm, Rio, Warp.** GPU(Metal 또는 OpenGL 3.3)가
필요합니다. 같은 프로그램이 하는 일의 기준은 이들의 Linux 결과입니다. macOS 칸에 의존하기
전에 GPU가 있는 Mac에서 다시 실행하세요.

### 그림: GitPoulpi가 고르는 프로토콜과 표시 여부

| 터미널 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **동작**: kitty 프로토콜 (GitPoulpi) | 해당 없음 | 측정 안 함 (GPU 필요) |
| **Ghostty** | **동작**: kitty 프로토콜 (GitPoulpi) | 해당 없음 | 측정 안 함 (GPU 필요) |
| **WezTerm** | **동작**: 투명도가 있는 iTerm2 프로토콜 (GitPoulpi). 자리 표시자가 찌그러져 그려지므로 kitty 프로토콜은 쓰지 않음 | **동작**: 투명도가 있는 iTerm2 프로토콜, 그래프와 아바타가 제 행에 놓임 (공식 20240203과 업스트림 PR #8092 빌드). 공식 20240203은 GitPoulpi가 그림 둘레에 커서를 표시하기 전에는 그림을 엉뚱한 행에 놓았음, "ConPTY의 한계" 참고. | 측정 안 함 (GPU 필요) |
| **foot** | **동작**: 투명도가 있는 sixel (GitPoulpi) | 해당 없음 | 해당 없음 |
| **Rio** | **동작**: 투명도가 있는 iTerm2 프로토콜 (GitPoulpi) | **동작**: 투명도가 있는 iTerm2 프로토콜, 시험한 모든 글꼴 크기에서 칸에 정확히 맞음 (0.5.28, GitPoulpi). sixel과 kitty는 도착하지 않으므로(ConPTY가 버림) 고를 것이 없음 | 측정 안 함 (GPU 필요) |
| **Alacritty** | 텍스트: **동작** (터미널에 이미지 지원 없음) | 텍스트: **부분 동작** (그림 없음, 이미지 diff는 블록) | 텍스트: **동작** |
| **Tabby** | **동작**: 투명도가 있는 sixel (probe, Linux의 Tabby에서 GitPoulpi는 실행 안 함) | **동작**: 위치가 맞는 sixel (GitPoulpi) | **동작**: 투명도가 유지되는 sixel (GitPoulpi) |
| **Warp** | 측정 안 함 (XWayland가 필요하고 합성 키 입력을 잘못 해독해 아무것도 입력할 수 없었음) | **동작 안 함** (터미널: sixel, iTerm2, kitty 이미지 없음). GitPoulpi는 측정 안 함 (Warp의 입력 상자에서 시작할 수 없었음) | 측정 안 함 (GPU 필요) |
| **xterm** | probe: `-ti vt340`이면 sixel **동작**. GitPoulpi는 장치 속성에서 이를 고름(시작 시 질의는 시험 이후에 추가됨). 또는 `ui.image_protocol = "sixel"` 설정 | 해당 없음 | 텍스트: **동작**. sixel이 그려지며(probe) `ui.image_protocol = "sixel"`이 필요 |
| **iTerm2** | 해당 없음 | 해당 없음 | **동작**: 3.7.7 이상에서는 kitty 프로토콜, 대화상자 없음 (GitPoulpi). 이전 버전은 불투명한 배경의 인라인 이미지. sixel은 그려지지만 설정되지 않은 픽셀을 불투명하게 칠하므로 `ui.image_protocol = "sixel"`은 단색 사각형이 됨 |
| **Terminal.app** | 해당 없음 | 해당 없음 | 텍스트: **동작** (그림 없음) |
| **Windows Terminal** | 해당 없음 | **동작**: sixel (GitPoulpi, 1.25) | 해당 없음 |
| **conhost** | 해당 없음 | 텍스트: **부분 동작** (문자 그래프, 반블록 이미지 diff) | 해당 없음 |
| **mintty** | 해당 없음 | **동작 안 함** (ConPTY 뒤의 터미널: 커서를 표시하고 놓은 일반 `OSC 1337` 그림조차 전혀 그려지지 않음. 그래프 열은 비어 있고 텍스트와 팔레트는 정상) | 해당 없음 |

### 마우스 포인터 모양 (OSC 22)

| 터미널 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **동작** | 해당 없음 | 측정 안 함 (GPU 필요) |
| **Ghostty** | **동작** | 해당 없음 | 측정 안 함 (GPU 필요) |
| **WezTerm** | **동작 안 함** (터미널: 시험한 빌드에 구현되지 않음) | 공식: **동작 안 함** (터미널). 포인터 모양을 추가하는 업스트림 WezTerm PR #8092 빌드: **동작** (손, I-beam, 크기 조절 화살표) | 측정 안 함 (GPU 필요) |
| **foot** | **동작** | 해당 없음 | 해당 없음 |
| **Rio** | **동작 안 함** (터미널: 해석한 뒤 버림) | **동작 안 함** (터미널, 같은 이유. ConPTY 때문이 아님) | **동작 안 함** (터미널, 같은 소스. Linux와 Windows에서만 실행) |
| **Alacritty** | **동작 안 함** (터미널) | **동작 안 함** (터미널) | **동작 안 함** (터미널) |
| **Tabby** | **동작 안 함** (터미널: xterm.js) | **동작 안 함** (터미널) | **동작 안 함** (터미널) |
| **Warp** | 측정 안 함 | **동작 안 함** (터미널) | 측정 안 함 |
| **xterm** | **동작 안 함** (터미널: CSS 이름을 모름) | 해당 없음 | 측정 안 함 (포인터가 창 밖에 있었음) |
| **iTerm2** | 해당 없음 | 해당 없음 | **동작 안 함** (터미널) |
| **Terminal.app** | 해당 없음 | 해당 없음 | **동작 안 함** (터미널) |
| **Windows Terminal** | 해당 없음 | **동작 안 함** (터미널) | 해당 없음 |
| **conhost** | 해당 없음 | **동작 안 함** (터미널) | 해당 없음 |
| **mintty** | 해당 없음 | mintty 자체의 이름으로 **동작**, GitPoulpi가 그것을 보냄 | 해당 없음 |

칸이 "동작 안 함"이어도 마우스 아래의 구분선이나 버튼은 여전히 밝아집니다.

### 키보드 프로토콜

GitPoulpi는 Linux와 macOS에서 kitty 키보드 프로토콜을 요청합니다. Windows에서는 요청하지
않습니다. 콘솔이 수정자를 직접 보고하며, 아래 조합은 터미널이 자기 몫으로 가져가는 것에
의해 제한됩니다.

| 터미널 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **동작** | 해당 없음 | 측정 안 함 (GPU 필요) |
| **Ghostty** | **동작** | 해당 없음 | 측정 안 함 (GPU 필요) |
| **WezTerm** | **부분 동작**: `wezterm.lua`에서 `enable_kitty_keyboard = true`로 하기 전에는 응답 없음 (기본값 `false`) | **부분 동작**: WezTerm이 `Ctrl+Shift+N`을 가져감(자체 창을 엶). `Ctrl+,`는 전달되지 않음 (공식 빌드). PR #8092 빌드는 두 조합 모두 가져가지 않지만 `Ctrl+,`로 설정이 열리지는 않음 (ConPTY를 통한 kitty 키보드 응답 없음) | 측정 안 함 (GPU 필요) |
| **foot** | **동작** | 해당 없음 | 해당 없음 |
| **Rio** | **동작** (기본값으로 켜져 있음) | **부분 동작**: Rio가 `Ctrl+Shift+N`을 가져감(자체 창을 엶) | 측정 안 함 (GPU 필요) |
| **Alacritty** | **동작** (0.13 이상) | **부분 동작**: `Ctrl+Shift+N`이 `Ctrl+N`으로 도착하고 `Ctrl+,`는 전달되지 않음 | **동작** |
| **Tabby** | **동작 안 함** (터미널: 응답 없음. `Ctrl+Shift+N`과 `Ctrl+,`를 자기 몫으로 가져감) | **부분 동작**: Tabby가 `Ctrl+Shift+N`과 `Ctrl+,`를 가져감 | **동작 안 함** (터미널: 응답 없음). 일부 조합이 도착하지 않음 |
| **Warp** | 측정 안 함 | 측정 안 함 | 측정 안 함 (GPU 필요) |
| **xterm** | **동작 안 함** (터미널) | 해당 없음 | **동작 안 함** (터미널) |
| **iTerm2** | 해당 없음 | 해당 없음 | **동작** (3.7.7. 안 되면 *Profiles > Keys*를 확인) |
| **Terminal.app** | 해당 없음 | 해당 없음 | **동작 안 함** (터미널: 응답 없음, 기존 키) |
| **Windows Terminal** | 해당 없음 | **부분 동작**: WT가 `Ctrl+Shift+N`(새 창)과 `Ctrl+,`(자체 설정)를 가져감 | 해당 없음 |
| **conhost** | 해당 없음 | 측정 안 함 | 해당 없음 |
| **mintty** | 해당 없음 | **부분 동작**: 어느 조합도 눈에 보이는 효과가 없었음 | 해당 없음 |

조합이 도착하지 않으면 `Alt` 변형을 쓰거나
([단축키](keybindings.md#터미널이-보낼-수-없는-키와-kitty-키보드-프로토콜)),
*설정 > 키보드 및 마우스*에서 그 조합을 다시 지정하세요.

### 클립보드 (복사 동작)

복사는 로컬 디스플레이가 있으면 항상 시스템 클립보드를 쓰므로, 이 칸들은 터미널의
`OSC 52` 경로를 설명합니다. 이는 SSH, 컨테이너, 대체 수단일 때 중요합니다. 시스템
클립보드를 쓴 GitPoulpi의 복사 시험은 모두 동작했습니다.

| 터미널 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **동작** | 해당 없음 | 측정 안 함 (GPU 필요) |
| **Ghostty** | **동작** (tmux, zellij, herdr 뒤에서 확인). 단독: 측정 안 함 (클립보드 쓰기에는 키보드 포커스가 필요한데 시험에서 주지 않았음) | 해당 없음 | 측정 안 함 (GPU 필요) |
| **WezTerm** | **동작** | **동작** | 측정 안 함 (GPU 필요) |
| **foot** | 측정 안 함 (쓰기에는 키보드 포커스가 필요한데 시험에서 주지 않았음) | 해당 없음 | 해당 없음 |
| **Rio** | 측정 안 함 (같은 이유) | **동작** | 측정 안 함 (GPU 필요) |
| **Alacritty** | 측정 안 함 (같은 이유) | **동작 안 함** (기본 설정에서 클립보드가 바뀌지 않음. 조사하지 않음) | **동작** |
| **Tabby** | **동작** | **동작** | **동작** |
| **Warp** | 측정 안 함 | **동작 안 함** (클립보드가 바뀌지 않음) | 측정 안 함 (GPU 필요) |
| **xterm** | 측정 안 함 | 해당 없음 | **동작 안 함** (클립보드가 바뀌지 않음) |
| **iTerm2** | 해당 없음 | 해당 없음 | *Settings > General > Selection > Applications in terminal may access clipboard*를 켜기 전에는 **동작 안 함** |
| **Terminal.app** | 해당 없음 | 해당 없음 | **동작 안 함** (터미널) |
| **Windows Terminal** | 해당 없음 | **동작** (포커스가 있는 동안) | 해당 없음 |
| **conhost** | 해당 없음 | 측정 안 함 (probe를 실행하지 않음) | 해당 없음 |
| **mintty** | 해당 없음 | **동작 안 함** (기본 설정에서 클립보드가 바뀌지 않음) | 해당 없음 |

### 붙여넣기

Linux와 macOS에서 GitPoulpi는 bracketed paste를 요청하므로 여러 줄 붙여넣기는 하나의
붙여넣기입니다. 터미널별로 실행하지는 않았습니다(터미널별로는 **측정 안 함**). Windows
콘솔에는 붙여넣기 괄호가 없으므로, 먼저 묻지 않는 터미널은 텍스트를 키 입력으로 전달하며
모든 줄바꿈이 `Enter`가 됩니다. 그래서 GitPoulpi는 30 ms 안에 이어지는 키 입력(안쪽
`Enter`, 또는 여덟 글자 이상)을 하나의 붙여넣기로 취급합니다. 키를 누르고 있는 것이나
손으로 친 "abc, `Enter`"는 붙여넣기가 아닙니다.

| 터미널 | Windows (새 브랜치 프롬프트에 두 줄 붙여넣기) |
|---|---|
| **Windows Terminal** | **동작**: WT가 먼저 "여러 줄을 붙여넣겠습니까?"를 묻고(자체 경고), 브랜치는 만들어지지 않음 |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **동작** (브랜치가 만들어지지 않음) |
| **Warp** | 측정 안 함 (GitPoulpi를 그 안에서 시작할 수 없었음) |
| **conhost** | 측정 안 함 (시험이 보낸 `Ctrl+V`와 `Shift+Insert`가 기존 콘솔에 붙여넣어지지 않았음) |

### 이모지와 아이콘 너비

| 터미널 | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **동작** (넓은 칸이 정렬됨. kitty는 톱니바퀴 U+2699를 비워 그리며, 알려진 글꼴 문제) | WezTerm, Rio, Tabby, Windows Terminal: **동작**. Alacritty: **부분 동작** (이모지가 단색 윤곽선) | Alacritty, Tabby: **동작**. 그 밖: 측정 안 함 (GPU 필요) |
| **Tabby** | **동작** | **동작** | **동작** |
| **Warp** | 측정 안 함 | **동작** | 측정 안 함 |
| **xterm** | **부분 동작** (네모: 글꼴) | 해당 없음 | **부분 동작** (네모: 글꼴) |
| **iTerm2**, **Terminal.app** | 해당 없음 | 해당 없음 | **동작** |
| **conhost** | 해당 없음 | **부분 동작**: 컬러 이모지가 그려지고 넓은 칸이 정렬됨 | 해당 없음 |
| **mintty** | 해당 없음 | **동작** | 해당 없음 |

### 시험한 버전

kitty 0.49.2(macOS 0.50.0은 시작되지 않음), Ghostty 1.4.0, WezTerm은 Linux에서 20260716,
Windows에서 20240203(마지막 태그 릴리스), foot 1.28.0, Rio 0.5.28(Linux, Windows)과
0.5.29(macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411(Linux)과
378(XQuartz), iTerm2 3.7.7, Terminal.app 455.1(macOS 15.8), Windows Terminal 1.25.2733,
mintty 3.8.3. 터미널은 빠르게 바뀝니다. 이보다 새로운 버전은 다를 수 있습니다.

## 터미널별 안내

### Windows

- **Windows Terminal.** 런처의 기본값이며 Windows에서 그림 지원이 가장 좋습니다. sixel 그래프와
  아바타, OSC 52, 좋은 이모지, 칸 크기 질의에도 답합니다. *Settings > Profiles > Defaults >
  Appearance > Font face*에서 Nerd Font를 고르세요. 마우스가 캡처된 동안 텍스트를 선택하려면
  `Shift`를 누르고 있습니다. `Ctrl+Shift+N`과 `Ctrl+,`를 가져가므로 `Alt` 변형을 쓰거나
  다시 지정하세요. 포인터 모양은 바뀌지 않습니다.
- **WezTerm.** `wezterm.lua`에서 `config.enable_kitty_keyboard = true`(Linux와 macOS 빌드)와
  이모지 대체 글꼴이 있는 글꼴을 설정하세요. 예:
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  그림은 iTerm2 프로토콜을 쓰며 설정할 것은 없습니다. 마지막 태그 릴리스
  20240203-110809(2024년 2월)는 그림을 올바른 행에 놓습니다(2026년 11월에 현재
  GitPoulpi로 다시 확인. 아래 ConPTY 수정 전에는 그렇지 않았음). 업스트림 WezTerm PR #8092가
  포인터 모양을 추가합니다.
- **Rio** (MSI, UAC 필요). 그림(그래프, 아바타, 이미지 diff)은 투명도가 있는 iTerm2
  프로토콜을 쓰며 정확히 제 칸에 놓입니다. 0.5.28에서 기본 글꼴과 18, 22.5 px(칸 8x16, 11x21,
  13x26 픽셀, 항상 정수 픽셀)로 측정했으며, 20x10칸과 4x27칸 그림 모두 칸에 정확히 맞았습니다.
  설정할 것은 없습니다. 의사 콘솔이 칸 크기 질의를 삼키므로 GitPoulpi는 Rio의 칸이 너비의
  두 배 높이라고 가정합니다(세 크기 모두 그렇습니다). 비율이 다른 글꼴이면 둥근 commit
  표시를 얻도록 `GITPOULPI_TERMINAL_CELL_PX=WxH`(예: `10x24`)를 설정하세요. Rio는 어떤
  OS에서도 포인터 모양이 없습니다.
- **Tabby.** sixel 그래프와 아바타, 복사와 붙여넣기가 동작합니다. Tabby는 `Ctrl+Shift+N`과
  `Ctrl+,`를 자기 몫으로 가져갑니다.
- **Alacritty.** OpenGL 3.3이 필요합니다. 문자 그래프. 기본 설정에서는 `OSC 52`를 통한 복사가
  켜져 있지 않았습니다.
- **mintty** (Git Bash, MSYS2, Cygwin). 포인터 모양이 동작합니다(mintty 자체의 이름).
  그림은 표시되지 않으며(그래프 열은 비어 있음. mintty 3.8.3은 일반 probe로 놓은 경우에도
  ConPTY 뒤에서 `OSC 1337` 그림을 그리지 않음) 거기서는 칸 크기 질의에도 답하지 않습니다.
  `ui.graph_renderer = "text"`로 설정하세요.
- **Warp.** GitPoulpi로는 측정 안 함: Warp의 입력 상자에서 시작할 수 없었습니다. 이 터미널에는
  sixel, iTerm2, kitty 이미지가 없습니다.
- **conhost, cmd, PowerShell 창.** 문자 그래프, 반블록 이미지 diff. 기본적으로 이런 창에서
  시작한 GitPoulpi는 선택한 터미널에서 다시 열립니다. "다시 시작 규칙"을 참고하세요.

**ConPTY의 한계.** Windows는 모든 터미널의 프로그램을 의사 콘솔(ConPTY) 뒤에서 실행합니다.
최신 ConPTY를 함께 제공하는 Windows Terminal과 WezTerm 빌드는 그림 시퀀스를 통과시키지만,
Windows 안의 것(Rio와 mintty가 사용)과 WezTerm 20240203에 포함된 것은 그렇지 않습니다.
잃는 것: DCS(sixel)와 APC(kitty 그래픽) 시퀀스가 버려지므로 iTerm2 그림(`OSC 1337`)만
도착하며, 그래서 Rio와 WezTerm은 이를 씁니다. `CSI 16 t`(칸 크기)는 Windows Terminal과
Tabby에서만 응답합니다. 수정된 것: `OSC 1337`은 ConPTY를 즉시 통과해 ConPTY가 마지막으로
그린 커서 위치에 놓이는데, ConPTY는 커서가 보이는 동안에만 커서를 그립니다. GitPoulpi는
커서를 숨겨 두었으므로 그림이 오래된 행에 놓였습니다(푸터 위에 그래프, 스크롤 뒤 찢어진
화면). 이제 Windows에서는 쓰는 그림마다 같은 쓰기 안에서 둘레에 커서를 표시하고 다시
숨기며, Rio와 WezTerm 20240203은 모든 그림을 제 칸에 놓습니다(`pty_conpty_picture_anchor`가
ConPTY 뒤의 커서 이동을 다시 읽음). Windows 밖이나 sixel에서는 달라지는 것이 없습니다.

**Windows 붙여넣기.** 위의 "붙여넣기" 표를 참고하세요. GitPoulpi는 연속된 키 입력을 하나의
붙여넣기로 묶으므로 시험한 터미널에서는 여러 줄 붙여넣기가 프롬프트를 제출하지 않습니다.

**다시 시작 규칙.** exe를 더블 클릭하면 자체의 기존 콘솔 창(conhost)에서 시작합니다.
`ui.launch_terminal`이 다른 터미널(기본값: Windows Terminal)을 가리키면 GitPoulpi는 거기서
다시 열리고 첫 창은 닫힙니다(`ui.relaunch_in_terminal`). 이는 기존 콘솔 창에 **혼자 있을
때만** 합니다. exe를 직접 실행하는 터미널(`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty
`-e`, Tabby `run`)은 원하던 곳이므로 다시 시작하지 않으며, 셸에서 실행하면 혼자가 아니므로
역시 아무 일도 일어나지 않습니다. `ui.relaunch_in_terminal = false`로 끌 수 있습니다.

### Linux

- **kitty.** 설정 없이 모두 동작합니다. `Shift+클릭`은 kitty 자체의 것이므로 선택을 확장하려면
  `Alt+클릭`을 쓰세요([마우스](mouse-and-gestures.md)).
- **Ghostty.** 설정 없이 모두 동작합니다. 아이콘이 구멍으로 보이면 `font-family`에 이모지나
  Nerd Font를 추가하세요(키를 반복하면 대체 글꼴이 추가됩니다).
- **WezTerm.** iTerm2 프로토콜로 그림을 그리며 투명도가 섞입니다. 조합을 모두 받으려면
  `enable_kitty_keyboard = true`로 설정하세요. 포인터 모양은 없습니다.
- **foot.** sixel 그래프와 아바타, 포인터 모양, 완전한 키보드, 빠름. `foot.ini`에
  `font=JetBrainsMono Nerd Font:size=11`을 설정하세요. `OSC 52`를 통한 복사는
  `security.osc52`가 제어합니다.
- **Rio.** 투명도가 있는 iTerm2 프로토콜로 그림을 그리며 키보드가 완전합니다. 포인터 모양은
  없습니다(Rio는 `OSC 22`를 버립니다).
- **Alacritty.** 문자 그래프, 0.13부터 kitty 키보드. 이미지 지원 없음.
- **Tabby.** sixel 그림. kitty 키보드 없음, 포인터 모양 없음(Electron, xterm.js).
  `Ctrl+Shift+N`과 `Ctrl+,`를 가져갑니다.
- **xterm.** sixel을 쓰려면 `xterm -u8 -ti vt340`으로 시작하세요(박스 그리기에 UTF-8이
  필요합니다). 이모지 대신 네모가 나오는 것은 글꼴 때문입니다.
- **Warp.** 측정 안 함: XWayland가 필요하고 시험에서 합성 키 입력을 잘못 해독했습니다.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** 2026년 11월 시험에 포함되지 않았습니다. 문자와
  기존 키로 동작하므로 `Alt` 변형과 다시 실행용 `Ctrl+Y`를 쓰세요. VTE는 `OSC 52`를 무시하므로
  SSH를 통한 복사는 클립보드에 닿지 않습니다. 입력기(IBus, Fcitx)가 `Ctrl+Space` 스테이징
  전환을 가져갈 수 있으니 *설정 > 키보드 및 마우스*에서 다시 지정하세요. Konsole은 자체
  문서를 근거로 sixel을 인식합니다.
- **Wayland, Hyprland, Omarchy.** 포인터 힌트(`OSC 22`)는 kitty, Ghostty, foot에서 동작합니다.
  Omarchy 런처 항목은 `xdg-terminal-exec`를 거칩니다.

### macOS

- **iTerm2.** 3.7.7 이상은 질문 없이 kitty 프로토콜로 그림을 그립니다. 도움이 되는 설정:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    `OSC 52`를 통한 복사에 필요합니다(없으면 iTerm2가 "the terminal attempted to access
    the clipboard but it was denied"를 표시합니다).
  - *Profiles > Keys > Left Option key: Esc+*: `Alt` 조합에 필요합니다.
  - `ui.image_protocol = "iterm2"`로 설정하거나(또는 3.7.7보다 오래된 버전을 쓰면)
    "Allow Terminal-Initiated Display?" 대화상자에서 "Remember my choice"를 선택하세요.
    그러면 그림에 불투명한 배경이 있습니다(배경이 테마 색과 다르면 보이는 사각형.
    `ui.graph_renderer = "text"`로 없앨 수 있음).
  - kitty 키보드 프로토콜은 기본으로 켜져 있습니다(3.6.9 이상).
- **Terminal.app.** 256색뿐입니다. GitPoulpi가 `ui.color_depth = 256`을 스스로 고르며 테마가
  더 밋밋해 보입니다. 그림 없음, kitty 키보드 없음, `OSC 52`를 통한 클립보드 없음. `Alt`
  조합에는 *Use Option as Meta key*를 켜세요.
- **Alacritty, Tabby.** Alacritty: 문자 그래프, kitty 키보드, `OSC 52` 복사. `Alt` 조합에는
  `option_as_alt = "Both"`를 설정하세요. Tabby: sixel 그래프와 아바타.
- **Option을 Meta로.** 터미널이 Option을 Meta로 바꾸지 않으면 Option은 문자를 합성합니다
  (Option+B는 적분 기호를 입력). iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. GitPoulpi의 `Alt` 조합에는 이것이 필요하며 어디서도
  기본으로 켜져 있지 않습니다.
- **kitty, Ghostty, WezTerm, Rio, Warp**: macOS에서는 측정 안 함(GPU 필요). Linux와 같게
  동작할 것으로 예상합니다.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`. sixel에는 `ui.image_protocol = "sixel"`이
  필요합니다.

### VS Code와 Warp

VS Code 터미널(`TERM_PROGRAM=vscode`)은 시험에 포함되지 않았습니다.
`terminal.integrated.enableImages`가 켜져 있으면 장치 속성으로 인식되고, 아니면 GitPoulpi는
문자로 그립니다. `ui.image_protocol = "sixel"`로 그림을 강제할 수 있습니다. Warp: 위의 OS별
안내를 참고하세요.

## 멀티플렉서

tmux, zellij, screen, herdr, psmux, 그리고 터미널 자체의 창은 GitPoulpi와 터미널 사이에
있으며 통과하는 것을 바꿉니다. 요약: tmux는 sixel을 통과시키고 `allow-passthrough on`이면
포인터 모양과 kitty 그림도 통과시킵니다. zellij와 screen은 그림도 포인터 모양도 통과시키지
않습니다. herdr는 kitty 그림을 통과시킵니다. 모든 세부 사항과 추가할 설정은
[멀티플렉서](multiplexers.md)에 있습니다.

## 무엇이 어떻게 대체되는가

| 없는 것 | 대신 얻는 것 |
|---|---|
| 그림 프로토콜 | 문자 그래프. `ui.avatar_fallback`이 무엇이든 아바타는 색이 있는 이니셜 배지(한 글자)가 됩니다. 텍스트 칸에 욱여넣은 identicon은 알아볼 수 없기 때문입니다. 이미지 diff와 파일 보기는 색이 있는 반블록이나 정보 카드를 씁니다. |
| 트루컬러 | 색이 256색 또는 16색 팔레트로 매핑됩니다(`ui.color_depth`). |
| kitty 키보드 프로토콜 | 기존 키. 일부 조합이 다른 키로 도착합니다. 설정이 경고하며, 다시 지정하거나 `Alt` 변형을 쓰세요. |
| OSC 52 | 로컬에서는 달라지는 것이 없습니다(시스템 클립보드를 씀). SSH를 통하거나 디스플레이가 없으면 복사 동작이 클립보드에 아무것도 넣지 못합니다. 마우스로 선택하고(`Shift`를 누른 채) 터미널 자체의 복사를 쓰세요. |
| OSC 22 | 포인터가 바뀌지 않습니다. 마우스 아래의 구분선이나 버튼은 여전히 밝아집니다. |
| 이모지 글꼴 | 아이콘 대신 네모나 구멍. 그래프와 텍스트는 여전히 동작합니다. Nerd Font나 이모지 글꼴을 설치해 대체 글꼴로 나열하세요. |

## 알아 둘 설정

| 설정 | 용도 |
|---|---|
| `ui.image_protocol` | `auto`(기본값), `kitty`, `iterm2`, `sixel`, `none`. 그림이 있어야 할 곳에 깨진 것이 보이면 `none`으로 설정하세요. 다시 시작. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text`는 항상 올바릅니다(ConPTY의 한계가 적용되는 곳에서 쓰세요). 다시 시작. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | 터미널이나 멀티플렉서가 오작동하면 `auto` 또는 `off`. 다시 시작. |
| `ui.text_cursor` | `terminal`(기본값) 또는 `drawn`. |
| `ui.mouse_enabled` | 끄면 마우스를 터미널에 맡깁니다. 다시 시작. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | OS 바로 가기가 여는 터미널과, 더블 클릭 시 GitPoulpi를 거기서 다시 열지 여부([설정](settings.md)). |

[멀티플렉서](multiplexers.md), [그림](image-view.md), [단축키](keybindings.md),
[문제 해결](troubleshooting.md)도 참고하세요.

## 출처

문서는 2026년 10월에 확인했고 동작은 2026년 11월에 시험했습니다. 별도 표시가 없으면 1차 출처입니다.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html), kitty 키보드는 [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol)에서 (2차 출처)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (2차 출처), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (`OSC 22` 행 없음), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps), OSC 52는 [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp)에서 (2차 출처)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (트루컬러 없음): [Apple discussion](https://discussions.apple.com/thread/255315820) (2차 출처)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES), zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- 비교 사이트 (2차 출처, 교차 확인용으로만 사용): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

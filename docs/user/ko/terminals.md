[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · **한국어** · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# 터미널 고르기

GitPoulpi는 마우스를 지원하는 어떤 터미널에서든 실행되지만, 그림을 그리고, 트루컬러를
표시하고, 키보드 입력을 빠짐없이 보내 주는 터미널에서 가장 보기 좋고 쓰기 편합니다.
이 페이지는 OS별로 터미널의 순위를 매기고, 각 터미널이 무엇을 제공하는지, 어떤 설정을
바꿔야 하는지 설명합니다. 사실 관계는 2026년 10월에 각 프로젝트의 문서, 변경 내역,
이슈 트래커로 확인했습니다([출처](#출처) 참고). 여기 있는 내용을 모든 터미널에서
시험한 것은 아닙니다. **(확인되지 않음)** 표시가 있는 항목은 1차 출처로 확인하지
못했습니다.

## 추천 터미널

| OS | 추천 순서 | 이유 |
|---|---|---|
| **Windows** | **WezTerm**, 그다음 **Windows Terminal** | WezTerm은 그래프와 아바타를 투명도와 함께 그림으로 그립니다. Windows Terminal은 OS 통합이 가장 좋으며(런처의 기본값) 그래프와 아바타를 sixel 그림으로 그립니다(투명도 없음). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, 그다음 **foot** | kitty와 Ghostty는 그림과 완전한 키보드 입력을 제공합니다. WezTerm도 그림을 제공합니다. foot은 그래프와 아바타를 sixel 그림으로 그리며 빠르고 키보드 입력이 완전합니다. Omarchy에서 그림을 쓰려면 Ghostty나 kitty를 고르세요. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, 그다음 **iTerm2** | 이유는 같습니다. Terminal.app도 동작하지만 그림도 트루컬러도 없습니다. |

어떤 OS에서든 **tmux, zellij, screen**은 터미널의 그림 프로토콜을 GitPoulpi에게서
가리므로 문자로 그립니다([멀티플렉서](#멀티플렉서) 참고).

## GitPoulpi가 사용하는 것

- **그림**(그래프 열, 작성자 아바타, 이미지 diff와 파일 보기): 유니코드 자리 표시자를
  쓰는 **kitty 그래픽 프로토콜**, **iTerm2 인라인 이미지**(`OSC 1337`), **sixel** 중
  하나이며, 이 순서로 우선합니다. 감지는 기본적으로 거부하는 방식이며 `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION`, 멀티플렉서 변수를 보고 판단하며, 키보드는 읽지
  않습니다(따라서 터미널에 장치 속성도 묻지 않습니다). kitty와 Ghostty는 kitty
  프로토콜을 씁니다. WezTerm은 (의도적으로) iTerm2 프로토콜을 쓰며 PNG의 투명도를
  섞어 줍니다. iTerm2, Rio, mintty는 불투명한 배경과 함께 iTerm2 프로토콜을 씁니다.
  Windows Terminal, foot, Konsole, mlterm은 **sixel**을 씁니다. 어느 것도 해당하지
  않으면 아바타는 이니셜 배지입니다.
- **Sixel**은 **그래프 열**(레인, 노드, 그리고 `ui.avatars = "graph"`이면 commit 원 안의 아바타)에 쓰이고, 그 밖에는 **아바타**(Log, commit 상세, 파일 기록, Blame, 이미지 보기)에 쓰입니다. sixel에는 알파도 ID도 크기 조절도 없으므로 그림은 정확히 `열 수 x 칸 너비` 곱하기 `행 수 x 칸 높이` 픽셀이며, 무언가 있는 곳만 배경이 유지되는 빈 칸 위에 그립니다. 인코딩된 그림은 캐시에 보관되고, 레인, 스크롤 위치, 확대 비율이 바뀔 때만 다시 인코딩되며, 아래 칸이 다시 그려졌을 때만 다시 보냅니다. sixel에는 삭제 기능이 없으므로 GitPoulpi는 그 칸 위에 텍스트를 덮어써서 지웁니다. sixel에는 칸의 실제 픽셀 크기가 필요합니다. Windows Terminal에는 시작할 때와 확대 후에 한 번 물어봅니다(`CSI 16 t`, 약 10 ms 안에 응답하며 응답이 없어도 막히지 않습니다). 다른 곳에서는 창 크기나 `GITPOULPI_TERMINAL_CELL_PX=10x20`에서 얻습니다. sixel을 그리지만 인식되지 않는 터미널(`terminal.integrated.enableImages`를 켠 VS Code 터미널, `-ti vt340`으로 시작한 xterm)에는 `ui.image_protocol = "sixel"`이 필요합니다. 1.22 이전의 Windows Terminal은 sixel을 그리지 못하므로 거기서는 `ui.image_protocol = "none"`으로 설정하세요(그래프는 문자로 남습니다).
- **트루컬러**: 테마는 RGB입니다. `ui.color_depth = "auto"`는 `COLORTERM=truecolor`
  없이 `*-256color`인 `TERM`(그리고 Terminal.app)에서는 256색으로, Linux 콘솔에서는
  16색으로 낮춥니다. kitty, WezTerm, Windows Terminal은 트루컬러로 간주합니다.
- **마우스**: 버튼, 드래그, 이동을 포함한 SGR 마우스 보고(호버 피드백, 끌어서 놓기,
  오른쪽 클릭, 더블 클릭)입니다.
- **키보드**: **kitty 키보드 프로토콜**(disambiguate 플래그만)을 시작할 때 100 ms
  제한 시간으로 요청합니다. 이로써 `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`,
  `Ctrl+Shift+Z`가 전달됩니다. 없으면 기존 키가 남습니다(`Ctrl+Y`가 다시 실행하고,
  일부 단축키는 `Alt` 변형으로 대체됩니다).
  [단축키](keybindings.md#터미널이-보낼-수-없는-키와-kitty-키보드-프로토콜)를
  참고하세요.
- **클립보드**: 복사 동작은 `OSC 52`를 씁니다.
- **마우스 포인터 모양**(구분선 위의 크기 조절 포인터, 클릭할 수 있는 항목 위의 손):
  `OSC 22`이며, 구현된 곳에서만 덤으로 제공됩니다.
- **포커스 이벤트**를 켭니다(창이 포커스를 잃으면 툴팁이 닫힙니다). GitPoulpi는
  bracketed paste, OSC 8 하이퍼링크, undercurl을 요청하지 않습니다. 그림 경로에서만
  각 프레임을 동기화된 갱신(모드 2026, 모르는 터미널은 무시함)으로 감싸므로 팝업 위에서
  그림이 깜박이지 않습니다.
- **이모지와 기호**: 터미널과 글꼴이 변형 선택자에 대해 서로 다르게 동작하므로
  GitPoulpi는 기본 이모지 표현을 가진 글리프나 너비가 1인 BMP 도형만 씁니다.

## 기능 표

"그림"은 그 터미널에서 GitPoulpi가 오늘 실제로 그리는 것(**kitty** 또는 **iTerm2**
프로토콜, 또는 **텍스트**)이며, 터미널이 할 수 있는 것이 아닙니다.

| 터미널 | OS | 그림 (그래프, 아바타, 이미지 diff) | 트루컬러 | kitty 키보드 (전체 단축키) | OSC 52 복사 | OSC 22 포인터 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | 예 | 예 | 예 | 예 |
| **Ghostty** (1.3) | Linux, macOS (Windows 빌드 없음) | kitty | 예 | 예 | 예 | 예, 1.0부터 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, 투명도 포함 | 예 | `enable_kitty_keyboard = true`일 때만 (기본값 `false`) | 예 (쓰기 전용) | 아니요 (pull request가 아직 열려 있음) |
| **foot** (1.28) | Linux (Wayland) | 그래프와 아바타 모두 sixel (kitty 그래픽 없음) | 예 | 예, 1.10.3부터 | 예 (`security.osc52`) | 예, 1.12부터 |
| **Alacritty** (0.17) | 전체 | 텍스트 (upstream에 이미지 지원 없음) | 예 | 예, 0.13부터 | 예 (쓰기. 붙여넣기는 기본적으로 꺼짐) | 아니요 (구현되지 않음) |
| **Konsole** | Linux | 그래프와 아바타 모두 sixel | 예 | 2026년에 추가 중 (merge request !1188), 릴리스 (확인되지 않음) | 쓰기 전용, 2024년 중반에 병합됨 | (확인되지 않음) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | 텍스트 | 예 | 아니요 (릴리스된 VTE에 없음) | 아니요 (VTE가 무시함) | 아니요 |
| **iTerm2** (3.7) | macOS | iTerm2, 불투명한 배경 | 예 | 예, 3.6.9부터 (Profiles > Keys 확인) | 기본적으로 꺼짐: "Applications in terminal may access clipboard"를 켜세요 | (확인되지 않음) |
| **Terminal.app** | macOS | 텍스트 | 아니요 (256색) | 아니요 | (확인되지 않음) | (확인되지 않음) |
| **Windows Terminal** (1.25) | Windows | 그래프와 아바타 모두 sixel | 예 | 예, 1.25부터 안정판 (2026년 10월). 1.25 Preview는 2026년 3월부터 | 예 (쓰기. 포커스가 있을 때만) | 아니요 (구현되지 않음) |
| **Rio** | Windows, Linux, macOS | iTerm2, 불투명한 배경 (kitty 그래픽도 지원함) | 예 | 예 (기본값으로 켜짐) | 예 | 아니요 (구현되지 않음) |
| **Warp** | macOS, Linux, Windows | 텍스트 (감지되지 않음) | 예 | 예 | 예 (해당 시퀀스를 따름) | (확인되지 않음) |
| **VS Code 터미널** (xterm.js) | 전체 | 텍스트 (sixel과 iTerm2 이미지가 `terminal.integrated.enableImages` 뒤에 있으며, `ui.image_protocol = "sixel"`이면 sixel을 씀) | 예 | (확인되지 않음) | (확인되지 않음) | 아니요 |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, 불투명한 배경 | 예 | (확인되지 않음, 문서화되지 않음) | (확인되지 않음, 문서화되지 않음) | 예 |
| **Windows 콘솔** (conhost, cmd, PowerShell 창) | Windows | 텍스트 | 최근 Windows 10/11만 | 필요 없음: 한정 키를 스스로 보고함 | 아니요 | 아니요 |
| **ConEmu** | Windows | 텍스트 | (확인되지 않음) | (확인되지 않음) | (확인되지 않음) | (확인되지 않음) |
| **tmux, zellij, screen** | 전체 | 텍스트, 항상 | 설정 필요 | 설정 필요 | 설정 필요 | 아니요 |

OSC 22의 "아니요 (구현되지 않음)" 칸은 시험이 아니라 프로젝트의 변경 내역과 문서에
언급이 없다는 점과 사용자 보고에서 나온 것이므로 "문서화되지 않음"으로 받아들이세요.
이모지와 변형 선택자의 너비는 표로 만들지 않았습니다. 터미널과 글꼴마다 다르며, 그래서
GitPoulpi가 VS16 글리프를 피합니다([GitPoulpi가 사용하는 것](#gitpoulpi가-사용하는-것)
참고).

## 터미널별 안내

### Windows

- **WezTerm.** Windows에서 그림 지원이 가장 좋습니다. `ui.image_protocol`은 `auto`로
  두세요. `wezterm.lua`에서 전체 단축키를 위해 `config.enable_kitty_keyboard = true`를
  설정하고, 이모지 대체 글꼴이 있는 글꼴을 지정하세요. 예:
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  WezTerm의 마지막 태그된 안정판은 20240203-110809-5046fc22(2024년 2월)이고, 더 새로운
  수정은 nightly로 배포됩니다. WezTerm에는 아직 OSC 22 포인터 모양이 없습니다.
- **Windows Terminal.** 트루컬러, 마우스, OSC 52, 좋은 이모지 지원. GitPoulpi는 그래프와 아바타를
  sixel 그림으로 그립니다(Windows Terminal 1.22 이상). Nerd Font는 *설정 > 프로필 > 기본값 > 모양 > 글꼴*에서 고르세요.
  마우스가 캡처된 상태에서 텍스트를 선택하려면 `Shift`를 누르고 있으세요. 포인터
  모양은 바뀌지 않습니다. 1.25보다 오래된 빌드(kitty 키보드 프로토콜 없음)에서는 `Alt`
  변형을 사용하세요.
- **Rio**는 불투명한 그림 배경으로 동작합니다. 시험이 덜 되었습니다.
- **mintty**(Git Bash, MSYS2, Cygwin)는 불투명한 배경과 함께 iTerm2 프로토콜로 그림을
  그립니다. 시험이 덜 되었습니다.
- **Windows 콘솔, ConEmu, cmd**: 피하세요. 그림이 제대로 그려지지 않거나 아예 그려지지
  않습니다. exe를 더블 클릭하면 선택한 터미널에서 GitPoulpi가 다시 열립니다
  (*설정 > 일반 > 런처*).

### Linux

- **kitty.** 설정 없이 모든 것이 동작합니다. `Shift+클릭`은 kitty 자체의 동작이므로
  선택 범위를 넓히려면 `Alt+클릭`을 쓰세요([마우스](mouse-and-gestures.md)).
- **Ghostty.** 설정 없이 모든 것이 동작합니다. 아이콘이 구멍으로 보이면 `font-family`에
  이모지 또는 Nerd Font를 추가하세요(키를 반복하면 대체 글꼴이 추가됩니다).
- **WezTerm.** iTerm2 프로토콜로 그림을 그리고 투명도를 섞습니다. `enable_kitty_keyboard`
  주의 사항은 Windows와 같습니다.
- **foot.** sixel 그래프와 아바타, 완전한 키보드, 빠름. `foot.ini`에 `font=JetBrainsMono Nerd Font:size=11`을
  지정하세요.
- **Alacritty.** 문자 그래프, 0.13부터 kitty 키보드. Alacritty에는 이미지 지원이
  없습니다.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** 문자와 기존 키로 동작합니다. `Alt` 변형과
  다시 실행용 `Ctrl+Y`를 쓰세요(최근 Konsole은 kitty 키보드 프로토콜이 추가될 수
  있습니다). VTE에서는 복사 동작(`OSC 52`)이 클립보드에 아무것도 넣지 않습니다. 입력기
  (IBus, Fcitx)가 `Ctrl+Space` 스테이징 전환을 가로챌 수 있습니다 -
  **(확인되지 않음)**. *설정 > 키보드 및 마우스*에서 다시 지정하세요.
- **Wayland, Hyprland, Omarchy.** 마우스 포인터 힌트(`OSC 22`)는 kitty, Ghostty, foot에서
  동작합니다. Omarchy 런처 항목은 `xdg-terminal-exec`를 거칩니다.

### macOS

- **kitty, Ghostty, WezTerm**: Linux와 같습니다.
- **iTerm2.** 불투명한 배경으로 그림을 그립니다(배경이 테마와 다르면 사각형이 보입니다.
  `ui.graph_renderer = "text"`로 없앨 수 있습니다). 복사를 위해 *Settings > General >
  Selection > Applications in terminal may access clipboard*를 켜고, `Alt` 단축키를 위해
  *Profiles > Keys > Left Option key: Esc+*를 설정하세요.
- **Terminal.app.** 256색만 지원합니다. GitPoulpi가 `ui.color_depth = 256`을 스스로
  고르며 테마가 더 밋밋해 보입니다. 그림이 없습니다. `Alt` 단축키를 위해 *Use Option as
  Meta key*를 체크하세요.

### VS Code와 Warp

VS Code 터미널(`TERM_PROGRAM=vscode`)과 Warp는 그림 터미널로 인식되지 않으므로 GitPoulpi가
문자로 그립니다. 둘 다 쓸 만합니다. VS Code에서는 `terminal.integrated.enableImages`를
켜고 `ui.image_protocol = "sixel"`로 설정하면 아바타 그림을 얻습니다. Warp는 kitty 키보드 프로토콜 지원을 문서화하고
있으며, VS Code 터미널의 지원은 **(확인되지 않음)**입니다.

## 멀티플렉서

GitPoulpi는 tmux, zellij, screen, herdr(`TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` 또는
`tmux*`, `HERDR_*`)를 감지하고 **문자**로 그립니다. 멀티플렉서는 그림 이스케이프가 아니라
셀을 전달하기 때문입니다. 다른 기능을 쓰려면 `~/.tmux.conf`에 다음을 추가하세요.

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

그림을 전달하는 멀티플렉서에서 그림을 강제로 쓰려면 `ui.image_protocol`을 `kitty` 또는
`iterm2`로 설정하세요. kitty, Ghostty, WezTerm
안에서 `kitty_graphics = true`로 실행한 herdr는 자동으로 감지됩니다.
[문제 해결](troubleshooting.md#그래프가-그림이-아니라-문자로-그려짐-herdr-tmux-기타-터미널)을
참고하세요. `GITPOULPI_TERMINAL_IMAGE=off`는 모든 그림을 끕니다.

tmux는 kitty 키보드 프로토콜 자체를 전달하지 않습니다. `extended-keys`는 한정 키를
`CSI u` 또는 `xterm` 시퀀스로 보내게 합니다. zellij 0.41 이상은 변경 내역에 따르면
kitty 키보드 프로토콜을 지원하고 OSC 52를 전달합니다.

## 무엇이 어떻게 낮아지는가

| 없는 것 | 대신 얻는 것 |
|---|---|
| 그림 프로토콜 | 문자 그래프. 아바타는 `ui.avatar_fallback`과 무관하게 색이 있는 이니셜 배지(한 글자)가 됩니다. 텍스트 칸에 욱여넣은 identicon은 알아볼 수 없기 때문입니다. 이미지 diff와 파일 보기는 색이 있는 반블록 또는 정보 카드를 씁니다. |
| 트루컬러 | 색이 256색 또는 16색 팔레트로 매핑됩니다(`ui.color_depth`). |
| kitty 키보드 프로토콜 | 기존 키. 일부 단축키가 다른 키로 도착합니다. 설정이 경고하며, 다시 지정하거나 `Alt` 변형을 쓰세요. |
| OSC 52 | 복사 동작이 클립보드에 아무것도 넣지 않습니다. 마우스로 선택하고(`Shift`를 누른 채) 터미널 자체의 복사를 쓰세요. |
| OSC 22 | 포인터가 바뀌지 않습니다. 마우스 아래의 구분선이나 버튼은 여전히 밝아집니다. |
| 이모지 글꼴 | 아이콘 대신 상자나 구멍이 보입니다. 그래프와 텍스트는 그대로 동작합니다. Nerd Font나 이모지 글꼴을 설치하고 대체 글꼴로 지정하세요. |

## 알아 둘 설정

| 설정 | 용도 |
|---|---|
| `ui.image_protocol` | `auto`(기본값), `kitty`, `iterm2`, `sixel`, `none`. 그림이 있어야 할 자리에 깨진 문자가 보이면 `none`으로 두세요. 다시 시작해야 합니다. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text`는 항상 올바릅니다. 다시 시작해야 합니다. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | 터미널이나 멀티플렉서가 오동작하면 `auto` 또는 `off`. 다시 시작해야 합니다. |
| `ui.mouse_enabled` | 끄면 마우스를 터미널에 맡깁니다. 다시 시작해야 합니다. |
| `ui.launch_terminal` | OS 바로 가기가 여는 터미널([설정](settings.md)). |

[이미지](image-view.md), [단축키](keybindings.md), [문제 해결](troubleshooting.md)도
참고하세요.

## 출처

2026년 10월에 확인했습니다. 별도 표시가 없으면 1차 출처입니다.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html), kitty 키보드는 [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol)에서 (2차 출처)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (2차 출처), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps), OSC 52는 [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp)에서 (2차 출처)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (트루컬러 없음): [Apple discussion](https://discussions.apple.com/thread/255315820) (2차 출처)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES), zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- 비교 사이트 (2차 출처, 교차 확인용으로만 사용): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

아직 확인되지 않은 것: Konsole과 mlterm의 sixel(프로젝트 자체의 기능 목록에서 가져왔으며 릴리스로 다시 확인하지 않았고, 어느 터미널의 실제 창에서도 시험하지 않았습니다). Konsole, iTerm2, Terminal.app, Warp의 OSC 22. Terminal.app, VS Code,
mintty의 OSC 52. VS Code, mintty, ConEmu의 kitty 키보드 프로토콜. ConEmu의 모든 칸.
터미널별 이모지 너비. Konsole이 kitty 키보드 프로토콜을 어느 릴리스에 싣는지는 확인하지
못했습니다.

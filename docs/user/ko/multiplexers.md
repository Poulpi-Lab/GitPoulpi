[English](../multiplexers.md) · **한국어**

# 멀티플렉서와 터미널 창

멀티플렉서는 GitPoulpi와 터미널 사이에 있습니다. 칸을 스스로 다시 그리고 자신이 이해하는
이스케이프 시퀀스만 전달합니다. 그래서 터미널 단독으로 동작하는 것([터미널 고르기](terminals.md))이
tmux, zellij, screen, herdr 안에서는 동작하지 않을 수 있습니다. 이 페이지는 무엇이 통과하는지와
추가할 설정을 설명합니다. 결과는 터미널 표와 같은 2026년 11월 시험에서 나왔습니다: tmux 3.8
(MSYS2: 3.7c), zellij 0.45.1과 0.46.0, GNU screen 5.0(macOS는 4.00.03을 제공), herdr 0.9.5,
psmux 3.4.0.

GitPoulpi는 `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` 또는 `tmux*`, `HERDR_*`에서 멀티플렉서를
감지합니다. 그 안에서는 아래 표가 GitPoulpi가 동작하는 경로를 감지한다고 한 곳을 제외하고
그림이 기본적으로 문자가 됩니다. `GITPOULPI_TERMINAL_IMAGE=off`는 모든 그림을 끄고,
`ui.image_protocol`은 프로토콜을 강제합니다.

## 한눈에 보기

| | 그림 | 포인터 모양 (OSC 22) | 클립보드 (OSC 52) | 키 |
|---|---|---|---|---|
| **tmux** | 뒤의 터미널이 지원하면 sixel (감지됨). `allow-passthrough on`과 `ui.image_protocol = "kitty"`이면 kitty 그림 | `allow-passthrough on`이면 kitty와 foot에서 가능. Ghostty는 불가 | `set-clipboard on`이면 가능 | kitty 키보드 없음. `Ctrl+Enter`와 `Ctrl+,`는 도착하지 않음 |
| **zellij** | 문자 | 아니오 | 예 | kitty 키보드 동작 |
| **GNU screen** | 문자 | 아니오 | 아니오 | 기존 키 |
| **herdr** | kitty 그림 (`kitty_graphics = true`이면 감지됨) | 아니오 | 예 | 측정 안 함 |
| **psmux** (Windows) | 문자 (설계상) | 아니오 (Windows Terminal에 없음) | 시스템 클립보드 동작 | 측정 안 함 |
| **MSYS2 tmux** (Windows) | 문자 (설계상) | 아니오 | 시스템 클립보드 동작 | 측정 안 함. 화면이 깨짐 |
| **WezTerm 창**, **Windows Terminal 창**, **kitty와 Ghostty 분할**, **iTerm2 분할 창** | 터미널 자체와 같음 | 터미널과 같음 | 터미널과 같음 | 터미널과 같음 |
| **iTerm2 `tmux -CC`** | 문자 | 아니오 | 아니오 (iTerm2가 기본으로 거부) | 측정 안 함 |

## tmux

**그림.**

- **Sixel**은 tmux 자체가 그리며, 지원하는 클라이언트용으로 다시 인코딩합니다. GitPoulpi는
  시작할 때 tmux에 한 번 묻고(`tmux display-message -p '#{client_termfeatures}'`, 400 ms
  제한) 클라이언트가 sixel을 나열하면 sixel을 씁니다. foot, WezTerm, Rio, Tabby 뒤에서는
  그렇고, kitty와 Ghostty 뒤에서는 그렇지 않습니다(tmux가 "SIXEL IMAGE" 텍스트를 출력).
  sixel을 넣어 빌드한 tmux가 필요합니다(Arch와 Homebrew의 것은 그렇습니다. `tmux -V`와
  sixel 시험으로 확인하세요). 설정은 없습니다. 장치 속성 질의에 대한 tmux 자신의 응답은
  뒤에 무엇이 있든 sixel을 지원한다고 하므로, GitPoulpi는 대신 클라이언트의 기능을 묻습니다.
- **tmux 뒤의 iTerm2**는 sixel을 알리지만 설정되지 않은 픽셀을 불투명하게 칠하므로, 대부분
  투명한 그래프가 단색 사각형이 됩니다. GitPoulpi는 iTerm2를 알아보고(`LC_TERMINAL`,
  `ITERM_SESSION_ID`, 클라이언트 종류) 문자로 그립니다.
- **kitty 그림**은 tmux의 DCS 통과를 거칩니다(kitty, Ghostty. 원시 probe에서는 iTerm2와
  Rio에서도 확인). `set -g allow-passthrough on`(tmux 3.3a 이상) **그리고** GitPoulpi의
  `ui.image_protocol = "kitty"`가 필요하며, 그러면 GitPoulpi가 업로드와 삭제를 감쌉니다.
  tmux 옵션이 없으면 감싼 것이 버려져 그래프가 빈 채로 남습니다.
- iTerm2 인라인 이미지는 tmux를 통과하지만 tmux가 위치를 지정할 수 없으므로, GitPoulpi는
  거기서 이를 쓰지 않습니다.

**포인터 모양.** GitPoulpi는 `OSC 22`를 통과용으로 감쌉니다. `allow-passthrough on`이면
kitty와 foot에서 창에 닿습니다. Ghostty는 감싼 것을 표시하지 않습니다(화살표가 남음).
WezTerm, Rio, Alacritty, Tabby, iTerm2에는 표시할 `OSC 22`가 없습니다.

**클립보드.** `set -g set-clipboard on`. GitPoulpi는 통과용으로 감싼 형태도 보냅니다
(`allow-passthrough on`이면 동작). kitty, Ghostty, Alacritty, Tabby 뒤에서는 도착하며,
iTerm2는 클립보드 권한을 켜기 전에는 거부합니다.

**키.** tmux는 kitty 키보드 질의에 답하지 않으며, `extended-keys`로도 `Ctrl+Enter`나
`Ctrl+,`가 통과하지 않습니다. kitty 안의 tmux 3.8로 측정하면 `extended-keys`가 off, on,
always 어느 쪽이든 둘 다 그냥 `Enter`와 `,`로 도착합니다. `Ctrl+Shift+Z` 같은 글자 조합은
`extended-keys`가 on 또는 always **이고** `extended-keys-format csi-u`(tmux 3.5 이상)**이고**
애플리케이션이 확장 키를 요청할 때만 도착하는데, GitPoulpi는 요청하지 않으므로 이 설정은
여기서 아무것도 바꾸지 않습니다. tmux 안에서는 `Alt` 변형을 쓰세요
([단축키](keybindings.md#터미널이-보낼-수-없는-키와-kitty-키보드-프로토콜)).

**설정** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

**WSL 안의 tmux.** 측정 안 함. Linux 빌드이므로 Windows Terminal을 바깥 터미널로 한 위의
결과와 같을 것으로 예상합니다.

## zellij

그림: zellij는 kitty 프로토콜은 받지만 유니코드 자리 표시자는 받지 않고, sixel이나 iTerm2
이미지는 전달하지 않으므로 GitPoulpi는 문자로 그립니다(의도적인 거부). 포인터 모양: 삼켜지므로
GitPoulpi는 보내지 않습니다. 클립보드: `OSC 52`가 전달됩니다(kitty, Ghostty, Alacritty, Tabby
뒤에서 동작. iTerm2와 Terminal.app은 거부하거나 무시). 키: zellij는 kitty 키보드 질의에
답하므로 완전한 키보드가 동작합니다.

zellij 0.46은 새로 시작할 때 "Welcome to Zellij" 팝업을 엽니다. `Esc`를 누르세요. 종료한
세션은 이름을 유지하며, 같은 이름으로 다음에 시작하면 어떻게 할지 묻습니다. 설정은
필요 없습니다.

## GNU screen

아무것도 통과하지 않습니다. 그림과 `OSC 22`는 문자로 출력되거나 버려지고, `OSC 52`는
무시되며, 키보드는 기존 키입니다. GitPoulpi는 문자로 그립니다. screen 4.00.03(macOS가
제공하는 버전)은 인터페이스의 색과 넓은 글리프도 깨뜨립니다. screen 5는 Linux에서만
시험했습니다. 쓸 만한 세션이 필요하면 tmux나 zellij를 쓰세요.

## herdr

herdr는 **kitty 그림**을 다시 내보내는 터미널을 내장합니다. GitPoulpi는 herdr의
`config.toml`에 `kitty_graphics = true`(`[terminal]` 아래, 오래된 버전에서는
`[experimental]`)가 있고 herdr를 둘러싼 터미널이 이를 그릴 때(kitty, Ghostty, WezTerm)
이를 씁니다. 감지되며 설정은 없습니다. 설정이 다른 곳에 있으면(예: `HERDR_HOME` 아래.
GitPoulpi는 읽지 않음) `ui.image_protocol = "kitty"`로 설정하세요. SSH(`herdr --remote`)에서는
GitPoulpi가 로컬 클라이언트를 볼 수 없으므로 문자로 그립니다.
[문제 해결](troubleshooting.md#그래프가-그림이-아니라-문자로-그려짐-herdr-tmux-기타-터미널)을
참고하세요. herdr 0.9.5는 `OSC 22`를 전달하지 않으므로 GitPoulpi는 보내지 않습니다.
`OSC 52`는 kitty, Ghostty, Alacritty, Tabby 뒤에서 도착합니다. 원시 probe에서 herdr는 kitty,
Ghostty, iTerm2에서 kitty 그림을 그렸으며, 기본 설정에서도 그랬습니다. 키: 측정 안 함.

## psmux와 MSYS2 tmux (Windows)

- **psmux**(Rust, ConPTY, tmux 명령 언어. `winget install psmux`)는 `TMUX`를 설정하므로
  GitPoulpi는 설계상 거기서 문자로 그립니다. Windows Terminal 안에서는 정상적으로 실행되고
  복사는 시스템 클립보드로 동작하며, 포인터는 바뀌지 않습니다(Windows Terminal에는 `OSC 22`가
  없음). 다른 호스트 터미널은 측정하지 않았습니다.
- **MSYS2나 Cygwin의 tmux**도 `TMUX`를 설정하며(설계상 문자), 시험에서는 화면이 깨졌습니다.
  mintty와 Windows Terminal에서 오래된 프레임이 서로 겹쳐 그려졌습니다(`-u`가 있든 없든.
  없으면 박스 그리기가 `qqqq`로 출력됨). MSYS2 의사 터미널 안의 네이티브 Windows 프로그램은
  tmux가 올바르게 다시 그리지 못합니다. GitPoulpi에서 고칠 수 있는 것이 아니므로 psmux,
  Windows Terminal 창, WezTerm 창을 쓰세요.
- zellij와 screen에는 Windows 릴리스가 없습니다. herdr 0.9.5는 설치했지만 실행하지
  않았습니다.

## 터미널 창

터미널 자체에 속한 창은 GitPoulpi에게 멀티플렉서가 아닙니다.

- **WezTerm 창**(`wezterm cli split-pane`, `wezterm connect`, SSH 도메인): WezTerm이 모든 것을
  그리므로 그림이 동작하고(창 안의 그래프와 아바타) 포인터는 WezTerm 자체와 같게 동작합니다
  (업스트림 WezTerm PR #8092 빌드에서 모양이 동작하며, 창 구분선은 크기 조절 포인터를 줍니다).
  Linux와 macOS에서는 측정 안 함.
- **Windows Terminal 창**(`wt ; split-pane`): 창 안에서 sixel 그래프와 아바타가 동작하고, 복사도
  동작하며, 포인터 모양은 없습니다.
- **kitty, Ghostty, iTerm2 분할**: 터미널과 같은 네이티브입니다. kitty와 Ghostty는 macOS에서
  측정 안 함(GPU 필요).

## iTerm2 `tmux -CC` (제어 모드)

iTerm2는 tmux 창마다 네이티브 창을 열고 "[tmux bash]" 게이트웨이 창을 유지합니다. `TMUX`가
설정되므로 GitPoulpi는 네이티브 창에서 문자로 그립니다. 이미지는 제어 연결을 통해 iTerm2에
도달하지만("Allow Terminal-Initiated Display?" 대화상자가 `OSC 1337` 시험에서 나타났음),
GitPoulpi는 거기서 이를 쓰지 않습니다. 강제한 `ui.image_protocol = "kitty"`의 kitty 페이로드는
게이트웨이 창에 출력되므로 `auto`로 두세요. `OSC 22`와 `OSC 52`는 도착하지 않습니다.

## 알아 둘 설정

| 설정 | 용도 |
|---|---|
| `ui.image_protocol` | `kitty`는 그림을 전달하는 멀티플렉서(`allow-passthrough on`인 tmux, GitPoulpi가 읽지 않는 설정을 쓰는 herdr)를 통해 kitty 그림을 강제합니다. `none`은 문자. |
| `ui.graph_renderer` | `text`는 항상 올바릅니다. |
| `ui.kitty_keyboard` | 멀티플렉서가 키보드 질의에 오작동하면 `off`. |

멀티플렉서 뒤의 터미널은 [터미널 고르기](terminals.md)를, 그래프가 문자로 그려질 때는
[문제 해결](troubleshooting.md)을 참고하세요.

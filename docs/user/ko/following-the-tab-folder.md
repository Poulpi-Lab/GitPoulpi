[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · [Français](../fr/following-the-tab-folder.md) · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · **한국어** · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# 활성 탭의 폴더 따라가기

GitPoulpi는 여러 저장소를 탭으로 열어 둘 수 있습니다. GitPoulpi를 시작한 셸은 원래 폴더에
그대로 있습니다: 프로그램은 부모 셸의 폴더를 바꿀 수 없기 때문입니다. GitPoulpi는 활성 탭의
폴더에서 셸을 쓰는 두 가지 방법을 제공합니다:

- **종료 후 cd**: GitPoulpi가 종료되고 셸이 활성 탭의 폴더로 이동합니다. 이를 위해
  `gitpoulpi shell-init`이 출력하는 작은 셸 함수가 필요합니다.
- **열기 위치**: 활성 탭의 폴더를 터미널, 멀티플렉서 창, 편집기, 파일 관리자에서
  엽니다. 셸 설정이 필요 없습니다. [열기 위치](open-in.md)를 참고하세요.

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Open in ▾ ── ⌕ 전체 검색 ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Quit & cd here           │
```

- **열기 위치**는 제목 표시줄의 저장소 경로 뒤에 있습니다. 창이 너무 좁으면
  숨겨집니다.
- **종료 후 cd**는 탭 표시줄의 오른쪽 끝에 있습니다. 다음 두 조건이 모두 참일 때만
  표시됩니다:
  - GitPoulpi를 (아래의) 셸 함수로 시작했습니다.
  - 활성 탭이 GitPoulpi를 시작한 폴더가 아닙니다.
- 명령은 항상 팔레트(`Ctrl+P`)에 있습니다: `탭: 종료 후 여기로 cd`,
  `탭: 열기 위치...`(`Alt+O`), `탭: 기본 위치에서 열기`(`Ctrl+Alt+O`).
  셸 함수가 없으면 `탭: 종료 후 여기로 cd`가 설정 방법을 알려 줍니다.

일반 종료(`q`)는 셸의 폴더를 절대 바꾸지 않습니다. `auto`(아래)를 고르지 않는 한
**종료 후 cd**만 바꿉니다.

## 셸 함수 설정하기

`gitpoulpi shell-init <shell>`은 `gp`라는 이름의 함수를 출력합니다. 다른 이름을
쓰려면 `--name`을 사용하세요. 이 함수는 `gitpoulpi --cwd-file <temp file> ...`을
실행합니다. **종료 후 cd**로 종료하면 GitPoulpi가 그 파일에 폴더를 쓰고, 함수가 그
폴더로 이동합니다. GitPoulpi는 아무것도 설치하거나 편집하지 않습니다: 셸의 시작
파일에 한 줄을 추가하면 됩니다.

| 셸 | 추가할 파일 | 줄 |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | `$PROFILE`(`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

그런 다음 `gitpoulpi` 대신 `gp`로 GitPoulpi를 시작하세요. 인수는 그대로 전달되며
(`gp -s work`), 종료 상태는 `gitpoulpi`의 종료 상태입니다.

**기존 `gp`는 경고 없이는 절대 대체되지 않습니다.** 셸에 이미 `gp`라는 이름의 명령, 별칭,
함수가 있으면 스크립트는 경고를 출력하고 아무것도 정의하지 않습니다. 다른 이름을
고르세요. 예를 들어 `gitpoulpi shell-init bash --name gpl`입니다. PowerShell에서는
이 함수가 내장 `gp` 별칭(`Get-ItemProperty`)과 `gitpoulpi`를 가리키는 별칭을
대체합니다. 그 밖의 `gp`는 경고와 함께 유지됩니다.

모든 `gitpoulpi` 명령이 `--cwd-file`을 받지만, 파일을 쓰는 것은 TUI뿐입니다.

## 설정: 활성 탭의 폴더 따라가기

설정 > 열기 위치 > **활성 탭의 폴더 따라가기**(`ui.tab_dir_follow`):

| 값 | 동작 |
|---|---|
| `buttons`(기본값) | 위에서 설명한 **종료 후 cd**와 **열기 위치**. |
| `auto` | **종료 후 cd** 버튼이 없습니다. **열기 위치**는 그대로 있습니다. 모든 종료가 셸을 활성 탭의 폴더로 이동시킵니다(셸 함수로 GitPoulpi를 시작한 경우). 새 탭을 열 때마다 기본 **열기 위치** 항목이 실행됩니다. |

`auto` 모드에서 기본 항목은 직접 여는 탭에서만 실행됩니다: 홈 탭, worktree, 클론,
초기화에서 연 탭입니다. 시작할 때 복원되는 탭, 첫 번째 탭, 이미 열려 있는 탭으로
전환할 때는 실행되지 않습니다.

이전 설정 `ui.terminal_launch`와 `ui.terminal_launch_command`는 **열기 위치**
목록으로 옮겨졌습니다. [열기 위치](open-in.md#예전-키)를 참고하세요.

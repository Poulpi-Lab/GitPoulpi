[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · **한국어** · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi 문어" width="128"></p>

**데스크톱 앱처럼 쓰는 터미널용 Git 클라이언트입니다.** GitPoulpi는 클릭하고
드래그하고 오른쪽 클릭할 수 있는 완전한 commit 그래프, 모든 기능에 닿는
키보드 중심의 명령 팔레트, 코드 호스팅(GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, 클라우드 및 자체 호스팅)과 이슈 트래커(Jira, Trello, Linear, YouTrack, Redmine, Markdown 노트 폴더) 연동, 그리고 같은 작업을 스크립트로 쓸 수 있는 CLI와 MCP
서버로 제공하므로 AI 에이전트가 여러분과 똑같이 GitPoulpi를 조작할 수 있습니다.
GitPoulpi는 GitKraken에서 영감을 받아 Rust로 처음부터 다시 작성되었습니다: 네이티브 바이너리, 디스크와 메모리에 가벼움, Electron 없음, 계정 불필요, 원격 측정 없음. 좋아하는 터미널([어느 것?](../../docs/user/ko/terminals.md))에서 실행됩니다. 우리 중에는 그 안에서 사는 사람도 있으니까요.

![왼쪽의 Refs 사이드바, 가운데의 branch, tag, stash가 있는 commit 그래프, 오른쪽의 작업 중인 변경 사항과 commit 폼을 보여 주는 GitPoulpi](../../docs/user/images/hero.png)

- **마우스와 키보드, 똑같이.** 클릭, 더블 클릭, 오른쪽 클릭을 하고 ref를
  commit 위로 드래그 앤 드롭할 수 있습니다. 또는 `Ctrl+P`를 눌러 원하는 것을
  입력하십시오. 모든 마우스 동작에는 키보드 짝이 있고, 모든 명령이 팔레트에
  있습니다.
- **진짜 commit 그래프.** branch, merge, tag, stash, 작업 사본을 스크롤되는 하나의
  Log에 표시하며, 지원하는 터미널에서는 픽셀로, 그 밖의 곳에서는 문자로 그립니다.
  다양한 어두운/밝은 색상 테마, 작성자 아바타, 검색을 제공합니다.
- **정밀한 스테이징.** diff에서 바로 파일, 폴더, hunk, 한 줄 단위로 스테이징,
  스테이징 해제, 버리기를 할 수 있습니다. 인라인 또는 나란히 보기, 미니맵을
  제공합니다.
- **안전한 히스토리 편집.** 아무 commit이나 수정하고(메시지, 작성자, 날짜),
  검토 화면이 있는 interactive rebase를 하고, 미리보기와 한 번의 실행 취소로
  여러 commit을 한꺼번에 다시 쓰고, 여러 commit을 cherry-pick하고,
  fast-forward하고, worktree와 patch를 만들 수 있습니다.
- **호스팅과 이슈 트래커 내장.** pull request와 merge request(GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket)를 둘러보고, 체크아웃하고, 만들 수 있으며 Gerrit change를 리뷰할 수 있습니다. 이슈, 작업 항목, 카드(Jira, Trello, Linear, YouTrack, Redmine, 로컬 Markdown 노트 포함)를 둘러보고 만들 수 있고, 칸반 보드로도 볼 수 있습니다. 계정에서 클론하고 SSH 키를 관리합니다.
- **에이전트를 위한 CLI와 MCP.** `gitpoulpi <command> --json`이 UI가 하는
  모든 것을 다루고, `gitpoulpi-mcp`가 이를 MCP 도구로 제공합니다. 저장소마다
  백그라운드 데몬 하나가 모든 프런트엔드를 실시간으로 동기화합니다.
- **선택 사항인 AI, 필수가 아님.** 기본적으로 꺼져 있고 계정이 필요 없습니다:
  자신의 API 키나 `claude -p` 같은 로컬 에이전트 CLI를 가져오십시오. commit
  메시지 생성, 변경 사항 설명, pull request 설명 작성, 작업 중인 변경 사항을
  여러 commit으로 나누기, 히스토리 다시 쓰기 규칙 초안 작성을 할 수 있습니다.
- **Git LFS, 서브모듈, worktree, patch, blame과 파일 히스토리, 이미지 diff,
  내장 편집기.**

## 스크린샷

| | |
|---|---|
| ![Material 레인 팔레트와 행 색조가 켜진 Log 그래프](../../docs/user/images/theme-material.png) **commit 그래프**, 여기서는 행 색조를 켠 Material 레인 팔레트입니다. | ![각 hunk에 Stage Hunk와 Discard Hunk 버튼이 있는 수정된 파일의 diff](../../docs/user/images/diff.png) **diff**, hunk별 또는 줄별로 스테이징과 버리기를 할 수 있습니다. |
| ![stash라는 단어로 필터링한 명령 팔레트](../../docs/user/images/palette.png) **명령 팔레트**(`Ctrl+P`)는 모든 명령에 닿습니다. | ![작업 중인 변경 사항에 대해 commit 네 개를 제안하고 검토를 기다리는 commit composer](../../docs/user/images/composer.png) **commit composer**는 commit하기 전에 AI가 제안한 commit을 검토합니다. |
| ![cherry-pick, revert, reset, rebase, branch, tag 동작이 있는 commit의 오른쪽 클릭 메뉴](../../docs/user/images/context-menu.png) 무엇이든 **오른쪽 클릭**하면 해당 동작이 나옵니다. | ![프로필, 동작별 지침, 활성화 스위치가 있는 설정의 AI 페이지](../../docs/user/images/settings-ai.png) **설정**, 검색과 선택 사항인 AI 페이지를 포함합니다. |

더 많은 그림과 설명은 [사용자 문서](../../docs/user/ko/README.md)에 있습니다.

## 설치

시스템 `git`이 필요합니다. `git-lfs`는 선택 사항입니다.

### Linux와 macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Windows 설치 프로그램

[최신 릴리스](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)에서
`gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`를 내려받아 실행하십시오.
관리자용으로는 MSI가 있습니다.

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

[Docker에서 실행하기](../user/ko/docker.md)를 참고하십시오.

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### 릴리스 아카이브

[최신 릴리스](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest)에서
내려받아 압축을 풀고, 안에 있는 `INSTALL.txt`를 따르십시오.

### 소스에서

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

자신만의 릴리스 아카이브, 더 작은 맞춤 빌드, OS별 참고 사항:
[docs/user/ko/installation.md](../user/ko/installation.md),
[docs/user/ko/custom-builds.md](../user/ko/custom-builds.md).

## 빠른 시작

```
cd path/to/your/repository
gitpoulpi
```

- `?`는 모든 키와 동작이 담긴 도움말을, `Ctrl+P`는 명령 팔레트를, `Ctrl+,`는
  설정을 열고, `q`는 종료합니다.
- commit을 클릭하면 변경 내용을 볼 수 있습니다. Log의 맨 위 행(작업 중인 변경
  사항)을 클릭하면 파일을 스테이징하고 commit할 수 있습니다.
- 지원하는 터미널(kitty, Ghostty, WezTerm, iTerm2, sixel을 쓰는 Windows Terminal과 foot)에서는 그래프가 픽셀로
  그려집니다. 그 밖의 터미널(일반 Windows 콘솔, tmux)에서는 자동으로
  문자로 대체됩니다.

단계별 안내는 [시작하기](../../docs/user/ko/getting-started.md)에 있고,
[인터페이스 둘러보기](../../docs/user/ko/interface-tour.md)는 화면의 모든 영역에 이름을
붙여 설명합니다.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

AI는 선택 사항이며 기본적으로 꺼져 있습니다: *설정 > AI*에서 GitPoulpi 자체 AI를
켜십시오([AI 지원](../../docs/user/ko/ai.md)). 외부 에이전트가 GitPoulpi를 조작하게 하려면
[MCP와 에이전트](../../docs/user/ko/mcp-and-agents.md)(권한, 모든 도구)와 에이전트 스킬
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md)를 참고하십시오.

## 문서

전체 사용자 문서는 **[docs/user/ko/README.md](../../docs/user/ko/README.md)**에서 시작합니다:

| | |
|---|---|
| [시작하기](../../docs/user/ko/getting-started.md) | 설치, 저장소 열기, 첫 commit 만들기 |
| [인터페이스 둘러보기](../../docs/user/ko/interface-tour.md) | 창의 모든 영역과 그 이름 |
| [Log 다루기](../../docs/user/ko/log-and-history.md) | 그래프, ref, branch, tag, stash, rebase, cherry-pick |
| [스테이징과 commit](../../docs/user/ko/staging-and-committing.md) | 상태, diff, hunk와 줄, composer, 편집기 |
| [설정](../../docs/user/ko/settings.md) | 모든 설정 페이지와 옵션 |
| [AI 지원](../../docs/user/ko/ai.md) | 내장 AI 설정, 모든 AI 동작, 개인정보 |
| [호스팅과 이슈 트래커](../../docs/user/ko/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: 토큰, 저장소 연결, pull request와 merge request |
| [CLI](../../docs/user/ko/cli-reference.md)와 [MCP](../../docs/user/ko/mcp-and-agents.md) | 스크립트로 쓰거나 에이전트가 조작하게 하기(권한, 모든 도구) |
| [단축키](../../docs/user/ko/keybindings.md)(및 모든 팔레트 명령), [마우스](../../docs/user/ko/mouse-and-gestures.md), [구성](../../docs/user/ko/config-reference.md), [LFS](../../docs/user/ko/lfs.md), [실행 취소](../../docs/user/ko/undo-redo.md), [문제 해결](../../docs/user/ko/troubleshooting.md) | 참조와 도움말 |

개발자: [CLAUDE.md](../../CLAUDE.md)와 `docs/architecture/overview.md`가 크레이트와
설계를 설명하고, [CHANGELOG.md](../../CHANGELOG.md)에 릴리스별 변경 내용이 있습니다.

## 라이선스

[MIT](../../LICENSE).

## 크레딧 및 서드파티 라이선스

GitPoulpi는 오픈 소스 라이브러리를 기반으로 하며 공개된 테마의 색상 값을 사용합니다. 테마 제작자를 포함한 라이선스와 크레딧은 [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt)에 있습니다. 이 파일은 모든 릴리스에 포함되며, 정보 창과 `gitpoulpi licenses`로도 볼 수 있습니다.

## 상표

GitKraken은 해당 소유자의 상표입니다. GitPoulpi는 독립 프로젝트이며 해당 소유자와 제휴, 승인 또는 후원 관계가 없습니다. 이 이름은 호환성과 영감을 설명하기 위해서만 언급됩니다.

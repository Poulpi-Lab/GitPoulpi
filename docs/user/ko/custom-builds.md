[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · **한국어** · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# 사용자 지정 빌드

더 작은 바이너리를 원하는 분을 위한 기능입니다: 대화형 스크립트가 어떤 언어, 호스팅 연동, 테마, 로고 색상, 기능을 포함할지 묻고, 선택한 것만 최적화 프로필로 빌드합니다. 일반 빌드와 공식 릴리스는 달라지지 않습니다(모든 것이 켜져 있음).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd or PowerShell (scripts/build-custom.ps1)
```

결과물은 `target/custom/`에 생기며, 공식 `target/dist/`를 덮어쓰는 일은 없습니다. 스크립트는 각 바이너리의 크기를 출력하고, `target/dist/` 빌드가 있으면 그것과의 차이도 출력합니다. 바이너리 옆에 `BUILD-FEATURES.txt`도 씁니다.

## 선택하는 것

| 선택 | 끄면 |
|---|---|
| 언어 | 영어(항상 빌드됨)와 선택한 언어만 포함됩니다. 설정의 선택기에는 빌드된 언어만 나열됩니다. |
| 테마 | GitPoulpi Dark, GitPoulpi Light, Terminal은 항상 빌드됩니다(`system`에는 Dark와 Light가 필요). 다른 모든 테마는 계열별로 고릅니다(`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes`가 출력). 선택하지 않은 테마는 컴파일되지 않습니다: 설정 콤보와 `gitpoulpi theme list`에서 빠지고, 그런 테마를 지정한 설정은 알 수 없는 테마처럼 처리되어 평소처럼 경고가 표시됩니다. |
| 로고 색상 | 정보 창의 픽셀 문어는 20가지 색 구성으로 제공됩니다(`--list-logo-colors`). 컴파일해 넣을 것(하나 이상), 기본값(정보 창이 가장 먼저 보여 주며, 컴파일되지 않은 `ui.logo_color`가 저장되어 있을 때 대체되는 값이기도 합니다. 컴파일된 것 중 하나여야 하며, 공식 기본값: lavender), Windows 실행 파일 아이콘의 색상(아무 색 구성이나 가능, 기본값: 로고 기본값)을 고르세요. 색 구성이 하나뿐이면 로고는 클릭하면 여전히 회전하지만 그 색으로 남고, 색 목록(`c`, 오른쪽 클릭)은 제공되지 않습니다. |
| 연동(GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | 아무것도 없으면: URL로만 클론하며, 이슈와 pull/merge request가 없습니다. |
| AI | AI 버튼, 명령, `ai` CLI/MCP 도구가 없습니다. |
| MCP | `gitpoulpi-mcp` 바이너리와 `gitpoulpi mcp` 명령이 없습니다. `ui` 에이전트 명령은 남습니다. |
| LFS, Worktree, 서브모듈 | 해당 관리 화면과 명령이 사라지고, 형식을 갖춘 "이 바이너리에 포함되지 않음" 응답이 반환됩니다. |
| 아바타 다운로드 | 이니셜 배지만 표시되고 네트워크 가져오기가 없습니다. |
| 이미지 미리 보기 | 이미지 파일에 일반 바이너리 안내가 표시되며, **편집 프로그램**은 그대로 동작합니다. |
| Radar | Radar 탭, `Alt+R`, 팔레트 명령이 없으며 Radar는 열리지 않습니다. 연동이 필요합니다(연동이 없으면 어차피 동작하지 않습니다). |
| Mermaid | Mermaid gitGraph로 복사(Log 메뉴, `Shift+Y`, 팔레트)가 없고 CLI/MCP에서 `log --format mermaid`도 없습니다. 또한 [대화](hosting-integration.md#5c-대화)의 `mermaid` 블록 아래에 **다이어그램 열기** 버튼이 없고(블록은 코드로 남음) *대화: 항목의 다이어그램 열기* 팔레트 행도 없습니다. |
| Board | 칸반 보드 보기(이슈 헤더 버튼, 팔레트)와 `hosting board` 명령이 없습니다. 연동이 필요합니다. |
| Conversation | 이슈나 요청의 앱 안 [대화](hosting-integration.md#5c-대화) 보기가 없습니다: 이슈 행, 보드 카드, Radar 행에서 Enter를 누르면 이전처럼 브라우저가 열리고, *대화 열기* 행도 *대화: ...* 팔레트 명령도 없으며, 대화 안의 **AI로 쓰기**도 없습니다 (리뷰, 이슈 폼, pull request 폼의 입력창은 자기 것을 유지). CLI와 MCP의 대화 명령(`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`)과 `ai hosting-reply`는 그대로 있습니다. 연동이 필요합니다. |
| Review | 리뷰의 개요 페이지가 대화 보기이므로, Review가 있는 빌드에는 *Conversation*과 관계없이 대화 보기도 들어 있습니다. 앱 안의 코드 리뷰 보기(행 메뉴, pull request 행에서 Enter, 팔레트, Radar `Review`)와 AI 리뷰 보조가 없습니다. `hosting review` CLI/MCP 명령은 남습니다(Gerrit과 에이전트가 사용). 연동이 필요합니다. |
| 기록 워크벤치(고급 기록 다시 쓰기: 작성자, 날짜, 메시지, 내용) | 기록 다시 쓰기 워크벤치가 없습니다. `다시 쓰기:` 팔레트 명령, 워크벤치 코드 키, 키, 도움말 섹션이 없고, Log 메뉴에 "기록 다시 쓰기" 행이 없으며, 수정 대화상자에 "기록 더 깊이 수정..." 버튼이 없고, "AI로 규칙 만들기"나 스크립트 AI가 없으며, 동기화 설정 페이지에 스크립트 한도가 없고, AI 설정에 기록 다시 쓰기 AI 블록이 없습니다. `rewrite` CLI/MCP 명령, `ai rewrite-script`, `ai rewrite-rules`, `ui propose rewrite`는 명령 트리에서 사라집니다. `.mailmap` 신원(`mailmap`, `log.use_mailmap`)은 이에 포함되지 않으며 그대로 남습니다. |
| 전체 검색 | 제목 표시줄에 전체 검색 버튼이 없고, `Ctrl+Alt+F`와 `검색: 워크스페이스...` 팔레트 명령, `gitpoulpi search` CLI/MCP 명령이 없습니다. Log 찾기 막대(`ui show search`)는 다른 기능이므로 그대로 남습니다. |
| 워크스페이스 | 창은 하나뿐인 기본 탭 목록("워크스페이스 없음")으로 동작합니다. 워크스페이스 칩, 워크스페이스 탭, `Alt+W` / `Alt+J`, `워크스페이스:` 팔레트 명령, "워크스페이스에 추가" 탭 행, `gitpoulpi workspace` CLI/MCP 명령, `commit`과 `search`의 `--workspace` 플래그가 없습니다. 탭과 세션은 평소처럼 동작합니다. 다른 빌드가 쓴 `workspaces.toml`은 읽지도 바꾸지도 않으며, 워크스페이스의 기본 프로필과 AI 지침은 적용되지 않습니다. Radar와 전체 검색은 열린 탭을 대상으로 합니다. |
| 다시 쓰기 스크립트(내장 Lua) | 기록 다시 쓰기의 Script 규칙은 형식이 지정된 "이 바이너리에 포함되지 않음" 응답으로 거부되며 Lua는 링크되지 않습니다. 기록 워크벤치 자체는 그대로 남습니다(별도의 선택입니다). 기록 워크벤치가 꺼져 있으면 의미가 없으며 경고가 표시됩니다. |

구성 파일은 빌드 사이에 공유됩니다: 빌드되지 않은 기능에 대한 설정은 유지되며 무시됩니다.

## 대화형

Bash 스크립트는 `whiptail` 또는 `dialog` 체크리스트(화면 세 개, 이어서 요약)를 쓰고, 둘 다 없으면 번호 프롬프트를 씁니다. PowerShell 스크립트는 가능하면 `Out-GridView`를 쓰고, 아니면 프롬프트를 씁니다. 이전 선택이 미리 체크되어 있습니다.

## 비대화형

인수가 하나라도 있으면 질문을 건너뛰고 곧바로 `cargo run -p xtask -- build-custom`으로 갑니다:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| 옵션 | 의미 |
|---|---|
| `--langs fr,de` / `all` / `en` | 영어 외의 언어. 코드는 로캘 파일 이름입니다(`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | 제공자: GitHub, GitLab(자체 호스팅 포함), Azure DevOps(Services와 Server), Bitbucket(Cloud와 Data Center)(호스팅), Codeberg를 포함한 Gitea와 Forgejo(`gitea`)(호스팅), Jira, Trello, Linear, YouTrack(Cloud와 Server), Redmine(이슈 트래커), Gerrit Code Review(`gerrit`, 리뷰 전용), 로컬 Markdown 이슈(`local-issues`, 폴더 또는 Obsidian 볼트). `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack`은 각각 독립된 cargo 기능입니다. 크기는 [binary-size](../../developer/binary-size.md)에 있습니다. |
| `--themes all` / `none` / `nord,catppuccin` | 컴파일해 넣을 테마 계열(기본값 `all`). `--list-themes`가 계열을 출력합니다. |
| `--logo-colors all` / `violet,teal` | 컴파일해 넣을 로고 색 구성(기본값 `all`이며 표의 순서, 즉 회전 순서를 유지). `--list-logo-colors`가 출력합니다(`*` = 공식 기본값). |
| `--logo-default teal` | 기본 색 구성. 컴파일된 것 중 하나여야 합니다(기본값: lavender, 없으면 컴파일된 첫 번째). |
| `--icon-color sun` | Windows 실행 파일 아이콘의 색상(기본값: 로고 기본값). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | 각각 기본값은 켜짐입니다. |
| `--[no-]history-rewrite` | 기록 워크벤치(고급 기록 다시 쓰기: 작성자, 날짜, 메시지, 내용). 기본값은 켜짐. |
| `--[no-]global-search` | 전체 검색(제목 표시줄 버튼, `Ctrl+Alt+F`, `gitpoulpi search`). 기본값은 켜짐. |
| `--[no-]workspaces` | 워크스페이스(이름 붙인 저장소 그룹). 기본값은 켜짐. 끄면 기본 탭 목록만 남습니다. |
| `--[no-]rewrite-scripts` | 다시 쓰기의 Script 규칙을 위한 내장 Lua. 기본값은 켜짐. 기록 워크벤치가 꺼져 있으면 경고와 함께 무시됩니다. |
| `--save <name\|file>` | 확정된 선택을 저장된 선택으로 저장합니다(이름 또는 파일 경로). |
| `--preset <name\|file>` | 저장된 선택에서 시작합니다(이름 또는 파일 경로). 다른 옵션은 줄의 어디에 있든 이를 덮어씁니다. |
| `--list-presets`, `--delete-preset <name>` | 저장된 선택과 그 폴더를 출력 / 이름으로 하나를 삭제합니다. |
| `--last` | `target/custom-build/last-choices.env`를 재사용합니다. 다른 옵션이 이를 덮어씁니다. |
| `--full` | 선택 없음(모두 켜짐). |
| `--profile custom\|dist\|release` | 기본값 `custom`(`dist` 설정에 별도의 출력 디렉터리). |
| `--dry-run` | 정확한 cargo 명령을 출력하고 멈춥니다. |
| `-- <cargo args>` | cargo에 그대로 전달됩니다. 예: `--target x86_64-unknown-linux-musl`. |

PowerShell 이름: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

테마 선택은 환경 변수 `GITPOULPI_THEMES`(`all`, `none` 또는 쉼표로 구분한 계열 목록)로 빌드에 전달되며, `gitpoulpi-config`의 빌드 스크립트가 이를 읽어 컴파일 시점에 다른 팔레트를 제외합니다. 원한다면 일반 `cargo build`에서 직접 설정해도 됩니다. `gitpoulpi
--version`은 빌드된 집합을 담은 `themes:` 줄을 출력합니다.

로고 선택은 환경 변수 `GITPOULPI_LOGO_COLORS`(`all` 또는 slug를 쉼표로 구분한 목록), `GITPOULPI_LOGO_DEFAULT`, `GITPOULPI_ICON_COLOR`이며, `gitpoulpi-tui`의 빌드 스크립트(`assets/logo/colourways.json`을 걸러 정보 창이 내장하는 표로 만듦)와 세 바이너리 크레이트의 빌드 스크립트(아이콘)가 읽습니다. 색상의 원본은 계속 `assets/logo/colourways.json`입니다.

### Windows 실행 파일 아이콘

대상이 Windows이면 `gitpoulpi`, `gitpoulpi-tui`, `gitpoulpi-mcp` 실행 파일이 문어를 아이콘으로 갖게 되며, 이 아이콘은 빌드할 때 선택한 아이콘 색상으로 16~128 px 크기로 생성됩니다(Python이나 ImageMagick 불필요). `winresource` 크레이트로 내장되며 플랫폼의 리소스 컴파일러(MSVC에서는 Windows SDK의 `rc.exe`, GNU에서는 `windres`)가 필요합니다. 없으면 빌드가 경고만 출력하고 실행 파일은 기본 아이콘을 유지합니다. GNU 툴체인에서는 `gitpoulpi`만 아이콘을 갖습니다(그렇지 않으면 리소스가 두 번 링크됩니다). Linux와 macOS용 빌드는 색상 표를 제외하고 이 모든 것을 무시합니다.

## 저장된 선택

선택 전체를 다시 하지 않고 새 버전을 빌드하려면 선택을 이름으로 저장해 두었다가 나중에 거기서 시작하세요. `--preset`과 함께 준 옵션은 저장된 값을 여전히 덮어씁니다.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

저장된 선택은 단순한 `KEY=value` 파일(`last-choices.env`와 같은 형식이며 셸에서 읽을 수 있음)입니다. GitPoulpi 설정 디렉터리의 `custom-builds` 폴더(`target/` 바깥)에 있으므로 `cargo clean`으로 지워지지 않습니다.

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (기본값은 `~/.config/gitpoulpi/custom-builds/`)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR`은 이 폴더를 설정과 함께 옮깁니다. `--preset`과 `--save`는 파일 경로(`/` 또는 `\`를 포함하거나 `.env`로 끝나는 값)도 받으므로 선택을 dotfiles 저장소에 보관할 수 있습니다.

다른 버전의 GitPoulpi가 쓴 선택도 불러올 수 있습니다. 알 수 없는 키나 더 이상 없는 언어, 통합, 테마, 색은 경고와 함께 무시됩니다. 파일에 없는 키(나중에 추가된 기능)는 기본값을 사용하고, 나중에 추가된 통합은 포함됩니다. 둘 다 표준 오류 출력으로 알려 줍니다.

저장된 선택이 있으면 대화형 모드는 먼저 "시작 기준"(이전 선택, 저장된 선택, 또는 모두 켜기)을 묻고, 마지막에 현재 선택을 이름으로 저장할지 묻습니다.

## 절약되는 크기

2026-10-29, 커밋 `65999ef5c`의 Linux에서 전체 빌드를 기준으로 측정했으며, 괄호 안은 전체 빌드와의 차이입니다
(자세한 내용은 `docs/developer/binary-size.md`):

| 빌드(별도 표시가 없으면 `full` 기준) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| 전체(모두 켜짐) | 64.6 | 50.5 | 31.7 |
| 영어만 | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP 끔 | 61.6 (-3.0) | 49.6 (-0.8) | 빌드되지 않음 |
| AI 끔 | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| 연동 모두 끔 | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| AI + 연동 + 아바타 가져오기 끔 | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| 기록 워크벤치 끔 (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| 다시 쓰기 스크립트 끔 | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| 전체 검색 끔 | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| 워크스페이스 끔 | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar 끔 | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board 끔 | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review 끔 | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid 끔 | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS 끔 | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree 끔 | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| 서브모듈 끔 | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| 아바타 가져오기 끔 | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| 이미지 보기 끔 | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| 모두 끔, 영어만 (LFS 유지) | 35.2 (-29.4) | 28.9 (-21.5) | 빌드되지 않음 |
| **모두 끔, 영어만** | **34.9 (-29.7)** | **28.8 (-21.6)** | 빌드되지 않음 |

## 참고

- 사용자 지정 빌드는 다른 `dist` 빌드처럼 느립니다(fat LTO, 몇 분).
- `build-opti`처럼 sccache 없이 실행됩니다.
- 언어 코드는 제공되는 로캘과 대조해 확인하며, 알 수 없는 코드는 유효한 코드 목록과 함께 거부됩니다.

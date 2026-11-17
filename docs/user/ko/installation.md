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
- 마우스를 지원하는 **터미널**. kitty, Ghostty, WezTerm, Rio, iTerm2, sixel을 쓰는 Windows Terminal, foot, Tabby는
  commit 그래프를 그림으로 그립니다. 그 밖의 터미널(Windows 콘솔, zellij, screen 및 대부분의
  다른 터미널)은 문자 그래프를 표시합니다. [터미널 고르기](terminals.md)와
  [멀티플렉서](multiplexers.md)를 참고하세요.
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
폴더에 넣으세요(`gitpoulpi-updater`와 `.gitpoulpi-dist` 표시 파일은 그 옆에 두세요.
표시 파일에 `channel=archive`가 적혀 있어 이 빌드가 스스로 업데이트할 수 있습니다.
[업데이트](updates.md) 참고). 아카이브 안의 `INSTALL.txt`에도 같은 단계가 있습니다.

### 다운로드 검증하기

모든 릴리스 파일은 프로젝트의 [minisign](https://jedisct1.github.io/minisign/) 키로 서명되어 있으며, GitPoulpi는 업데이트를 설치하기 전에 이 서명을 직접 검사합니다. 직접 내려받은 파일은 옆에 있는 `.minisig`가 서명입니다. 공개 키(키 id `B461FFCBF6F6A4F2`)는 저장소에 `packaging/minisign.pub`로 커밋되어 있습니다.

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

명령이 출력하는 신뢰된 주석(trusted comment)을 읽으세요. `file:<파일 이름> version:<예상하는 버전>`이어야 합니다. 유효하지만 다른 파일이나 다른 버전을 명시한 서명은 해당 파일의 서명이 아닙니다.

릴리스가 빌드 출처 증명(저장소의 `ATTESTATIONS` 변수)과 함께 빌드된 경우, GitHub CLI로 파일이 어디서 빌드되었는지 확인할 수 있습니다.

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

`.sha256` 파일은 손상된 다운로드만 잡아냅니다. 파일이 프로젝트의 것임을 보여 주는 것은 서명입니다.

## 설치 프로그램으로

릴리스마다 OS별 설치 프로그램도 제공합니다. 둘 다 같은 선택(바로 가기, PATH,
`gp` 같은 명령 별칭)을 설치된 `gitpoulpi` 자체를 호출해 처리합니다. 설정 > 일반 >
런처의 동작과 같은 코드이므로, 설치 프로그램이 해 둔 일과 설정에 표시되는 내용은
언제나 일치합니다.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

첫 대화상자가 누구를 위한 설치인지 묻습니다.

| 선택 | 위치 | 관리자 권한 | 업데이트에 대한 의미 |
|---|---|---|---|
| **나만 사용**(기본값) | `%LOCALAPPDATA%\Programs\GitPoulpi` | 없음 | GitPoulpi가 자신의 파일을 교체합니다: [업데이트 패널](updates.md)의 **업데이트 후 재시작**, 확인 창 없음. |
| **모든 사용자** | `Program Files\GitPoulpi` | 설치할 때 한 번 확인 | 업데이트마다 관리자 승인(Windows 확인 창)을 요청하며, 이 컴퓨터에서 다른 사용자가 실행 중인 GitPoulpi 세션이 먼저 종료됩니다. |

같은 선택을 자동으로도 할 수 있습니다: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` 또는 `/ALLUSERS`.

체크박스: 시작 메뉴 바로 가기, 바탕 화면 바로 가기, PATH에 추가. 한 페이지에서 **명령 별칭**을 묻습니다. 기본값은 `gp`이고 원하는 이름을 쓸 수 있으며, 비워 두면 별칭을 만들지 않습니다(명령줄에서는 `/ALIAS=name` 또는 `/ALIAS=`). PowerShell은 이미 `gp`를 `Get-ItemProperty`에 쓰므로 PowerShell에서는 다른 이름을 고르거나 `gitpoulpi`를 쓰세요.

설치 프로그램은 실행 파일(파일을 교체하는 작은 프로그램인 `gitpoulpi-updater` 포함)과 `.gitpoulpi-dist` 표시 파일을 복사합니다. 표시 파일에는 GitPoulpi가 설치된 방법(`inno-user` 또는 `inno-machine`)이 기록되며, 업데이트 패널이 이를 **설치 방법**으로 표시합니다. 설치 프로그램을 다시 실행해도 선택(범위, 바로 가기, 별칭)은 유지됩니다.

제거는 Windows 설정 > 앱에서 합니다. `gitpoulpi uninstall`(별칭, PATH 항목, 바로 가기)을 실행한 뒤 업데이트 프로그램이 남긴 `.old` 파일을 포함해 파일을 삭제합니다. 설정과 저장소는 그대로 남습니다.

### Linux와 macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

OS와 CPU에 맞는 아카이브를 고르고 `.sha256`을 확인한 뒤 `~/.local/opt/gitpoulpi`에
설치하고(채널 `archive`: GitPoulpi가 제자리에서 스스로 업데이트), `gitpoulpi`와 `gitpoulpi-mcp`를 `~/.local/bin`에 링크합니다. root 권한이
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
## 패키지 관리자로

아래의 모든 채널은 프로그램 옆에 같은 `.gitpoulpi-dist` 표시 파일을 쓰며 채널 이름만 다릅니다. GitPoulpi는 이를 읽어 누가 업데이트를 맡는지 알아냅니다. 패키지 관리자가 맡으므로 GitPoulpi는 그 몰래 파일을 교체하지 않습니다. 그러면 업데이트 패널(제목 표시줄의 버전을 클릭)이 패키지 관리자 자체의 명령을 실행하거나, 복사할 수 있도록 표시합니다. [업데이트](updates.md)를 참고하세요.

| 채널 | 설치 | 비고 |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | 모든 사용자용으로 설치되므로 업데이트에 관리자 승인이 필요합니다. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` 다음 `scoop install gitpoulpi` | 사용자별, 관리자 권한 없음. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | tap은 `Poulpi-Lab/homebrew-gitpoulpi`입니다. |
| **apt** (Debian, Ubuntu) | [apt 저장소](#apt-debian-ubuntu) 참고 | 서명된 저장소. 모든 릴리스에 `.deb`도 첨부됩니다. |
| **MSI** (Windows, 관리형) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | 관리자용: 모든 사용자, 앱 내 업데이트 없음. [엔터프라이즈 배포](enterprise-distribution.md#msi-intune-또는-그룹-정책)를 참고하세요. |
| **cargo** | `cargo binstall gitpoulpi-cli` | 컴파일하지 않고 릴리스 아카이브를 내려받습니다. |
| **cargo, crates.io에서** | `cargo install --locked gitpoulpi-cli` | 컴파일합니다. Rust 툴체인이 필요합니다. 하위 명령 없이 실행하면 터미널 UI를 여는 `gitpoulpi`를 설치하며, MCP(`gitpoulpi mcp`)도 제공합니다. `gitpoulpi-tui`와 `gitpoulpi-mcp`는 같은 프로그램을 독립 크레이트로 낸 것입니다. |
| **cargo, 소스에서** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | 컴파일합니다. Rust 툴체인이 필요합니다. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | [Docker에서 실행하기](docker.md)를 참고하세요. |

위의 이름은 릴리스 절차가 게시하는 대상입니다([릴리스](../../developer/releasing.md) 참고). 아직 게시되지 않은 채널은 "찾을 수 없음"으로 응답하며, 릴리스 아카이브나 설치 프로그램은 항상 동작합니다.

패키지 관리자 자체의 PATH 처리가 적용됩니다. Chocolatey, Scoop, Homebrew는 프로그램을 PATH에 직접 올려 주며, 어느 것도 `gp` 별칭을 만들지 않습니다(`gitpoulpi alias create gp`를 실행하거나 설정 > 런처를 사용하세요).

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

저장소는 `amd64`와 `arm64`를 지원하며 프로젝트 소유의 키로 서명됩니다. 프로그램은 `/usr/lib/gitpoulpi`에 설치되고 `/usr/bin`에 링크가 만들어집니다. 패키지는 `git`에 의존하고 `git-lfs`를 권장합니다. 저장소 없이 릴리스의 `.deb`를 설치하려면 `sudo apt install ./gitpoulpi_<version>_<arch>.deb`를 사용하세요. 이 경우 GitPoulpi는 다음 `.deb`를 직접 내려받으라고 안내합니다. `/etc/apt/sources.list.d/gitpoulpi.*`에서 저장소를 찾았을 때만 `apt` 명령을 제안하기 때문입니다.

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

제목 표시줄의 앱 이름과 저장소 경로 사이에 실행 중인 버전(`v1.0.0`)이 표시됩니다. 더 새로운 릴리스가 알려져 있거나(또는 업데이트가 진행 중일 때) 이를 클릭하면 버전, GitPoulpi가 설치된 방법, 새 릴리스에 대해 할 일이 담긴 업데이트 패널이 열리고, 그렇지 않으면 클릭 시 릴리스 노트 탭이 열립니다. `Ctrl+N` 다음 `u`, 또는 명령 팔레트(`Ctrl+P`)의 **업데이트: 업데이트 패널 보기**는 항상 업데이트 패널을 열고, **도움말: 릴리스 노트**는 항상 변경 로그를 엽니다.

하루에 한 번 백그라운드에서 GitPoulpi가 릴리스가 게시되는 곳(기본값은 GitHub)에 새 릴리스가 있는지 확인합니다(`updates.mode`, [설정 > 동기화](settings.md#동기화)). 있으면 칩이 강조색의 `v0.2.0 사용 가능`으로 바뀝니다.

패널의 동작은 GitPoulpi가 설치된 방법에 따라 다릅니다. 릴리스 아카이브나 Windows 설치 프로그램은 자신의 파일을 교체하고, 패키지 관리자는 자체 명령을 실행하며, 관리형 또는 컨테이너 설치는 업데이트가 전달되는 방법을 알려 줍니다. `cargo build`로 만든 빌드는 업데이트하지 않습니다. 되돌리기, `auto`와 `notify`의 차이, 관리자가 잠글 수 있는 것을 포함한 전체 설명은 [업데이트](updates.md)에 있습니다.

## 잘 동작하는지 확인하기

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version`은 버전과 빌드에 사용된 commit을 출력합니다. 그런 다음
[시작하기](getting-started.md)를 따르세요.

## 제거

`gitpoulpi uninstall`(두 설치 프로그램이 실행하는 것)은 GitPoulpi가 만든 명령 별칭, PATH 항목, 바로 가기, 파일 관리자 메뉴 항목(Windows에서는 App Paths 키 포함)을 제거한 뒤 릴리스 설치라면 프로그램 파일도 제거합니다. `.gitpoulpi-dist` 표시가 없는 빌드(cargo 빌드)에서는 거부하며 `--keep-files`는 프로그램 파일을 남겨 둡니다. MCP 에이전트는 사용할 수 없습니다. 설치 프로그램 없이 설치했다면 실행 파일 세 개를 삭제하세요. GitPoulpi는 다음 파일도 보관합니다:

| 항목 | 위치 |
|---|---|
| 설정(`config.toml`)과 세션 | OS의 구성 폴더. `GITPOULPI_CONFIG_DIR`로 바꿀 수 있습니다. |
| 저장소별 상태 | 기본적으로 저장소의 `.git` 폴더 안 `gitpoulpi/`(`session.per_repo_state_dir`로 작업 트리의 `.gitpoulpi/`나 사용자 데이터 폴더로 옮길 수 있음). |
| 캐시(아바타) | OS의 캐시 폴더. |
| 토큰과 API 키 | credential helper. **설정 > SSH 및 자격 증명 > 모든 사용자 이름과 비밀번호 삭제**로 제거합니다. |

정확한 경로는 [설정 참조](config-reference.md)를 참고하세요.

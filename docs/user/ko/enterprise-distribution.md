[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · **한국어** · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# 엔터프라이즈 배포

자체 서버에서 GitPoulpi를 여러 컴퓨터에 배포하는 관리자를 위한 문서입니다. 사내 패키지 피드, 미러, 컴퓨터 전체 업데이트 정책, 기본 설정을 다룹니다. 먼저 [설치](installation.md)와 [업데이트](updates.md)를 읽으세요.

모든 내용은 두 가지 사실에서 출발합니다.

- 모든 설치는 프로그램 옆의 `.gitpoulpi-dist` 마커(`channel=...`)에 **설치된 방법**을 기록합니다. GitPoulpi는 그 채널이 허용하는 방식으로만 스스로 업데이트합니다. 패키지 관리자가 소유한 파일은 교체하지 않으며 MSI는 절대 건드리지 않습니다.
- **정책**(파일이며 Windows에서는 레지스트리도)은 관리자의 것입니다. 사용자의 `config.toml`이 정책을 완화할 수는 없습니다.

## Chocolatey, 사설 피드

공개 패키지는 GitHub에서 설치 프로그램을 내려받습니다. 폐쇄망에서는 내부화하세요.

1. 패키지를 가져옵니다(라이선스가 있는 Chocolatey 도구의 `choco download gitpoulpi --internalize`, 또는 `.nupkg`를 내려받아 압축 해제).
2. `tools\chocolateyinstall.ps1`에서 `url64bit`이 `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`의 자체 사본을 가리키게 하고, `checksum64`는 릴리스의 `.sha256` 파일 첫 번째 필드(또는 사본의 체크섬)와 같게 유지합니다.
3. `choco pack`을 실행한 뒤 `.nupkg`를 피드(Nexus, Artifactory, ProGet, 파일 공유)에 푸시합니다.

패키지 매개변수와 함께 설치합니다.

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| 매개변수 | 효과 |
|---|---|
| `/Channel:choco-internal` | 마커에 `channel=choco-internal`을 기록하여, GitPoulpi가 이 피드가 Chocolatey 커뮤니티 피드가 아닌 관리자의 피드임을 알게 합니다. |
| `/Feed:<url>` | 피드 주소(마커의 `feed=`)를 기록합니다. GitPoulpi는 이 주소에서 새 버전을 확인하고 `choco upgrade gitpoulpi -y --source=<feed>`를 실행합니다. |

매개변수가 없으면 채널은 `choco`이고 확인은 Chocolatey 커뮤니티 피드로 갑니다. 패키지는 모든 사용자용으로 Program Files에 설치되며 `gp` 별칭은 없습니다.

GitPoulpi의 업데이트는 배포한 것이 건드려지지 않도록 `--params` 없이 `choco upgrade`를 실행합니다. 따라서 Chocolatey가 첫 설치의 패키지 매개변수를 기억해야 합니다(`useRememberedArgumentsForUpgrades`, Chocolatey 2.0부터 기본값 켜짐. 이전 버전에서는 `choco feature enable -n useRememberedArgumentsForUpgrades`). 그렇지 않으면 업그레이드가 일반 `choco` 채널을 기록하게 됩니다. 정책의 `check_url`(아래)은 마커의 내용과 관계없이 확인과 업그레이드가 관리자의 피드를 향하게 하며, 마커의 피드보다 우선합니다.

**패키지는 `policy.toml`이나 `config.toml`을 쓰지 않습니다.** 배포한 것은 설치 때나 모든 업그레이드 때나 그대로 남습니다. [기본 config.toml 배포](#기본-configtoml-배포)를 참고하세요.

## Scoop, 사설 버킷

공개 버킷의 `gitpoulpi.json`(매니페스트)을 직접 호스팅하는 Git 저장소의 `bucket/`에 넣고, `url`을 릴리스 zip의 자체 사본으로 바꾸고 `hash`를 그 sha256으로 설정합니다(사본이 릴리스 이름을 따른다면 `autoupdate`는 그대로 둘 수 있습니다). 그다음:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

매니페스트의 `post_install`이 마커를 `channel=scoop`으로 다시 씁니다. 그러면 GitPoulpi가 `scoop update gitpoulpi`를 직접 실행하며 Scoop 모르게 파일을 교체하지 않습니다. Scoop은 사용자별로 설치되며 관리자 권한이 필요 없습니다.

## apt, 미러

저장소는 HTTPS로 제공되는 평범한 디렉터리 트리입니다(`dists/`, `pool/`, 서명 키의 `gitpoulpi-archive-keyring.gpg`와 `.asc`, 예제 `gitpoulpi.sources`). `wget --mirror`, `rsync` 또는 아티팩트 관리자로 미러링하여 내부에서 제공하고, 각 컴퓨터에 키와 소스를 설치합니다.

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`:

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

파일 이름 `gitpoulpi.sources`(또는 `gitpoulpi.list`)를 유지하세요. GitPoulpi는 `/etc/apt/sources.list.d/gitpoulpi.*`를 찾았을 때만 `apt` 명령을 표시합니다. 프로젝트의 키를 사용하세요. 미러는 그 키로 서명된 상태로 유지되며 `Signed-By`는 이 저장소에 한해 그 키 하나만 고정합니다. 저장소는 각 패키지의 최근 다섯 버전을 보관합니다.

**오프라인 `.deb`:** 모든 릴리스에는 `gitpoulpi_<version>_<arch>.deb`가 첨부됩니다. 복사한 뒤 `sudo apt install ./gitpoulpi_<version>_<arch>.deb`를 실행하세요. 마커에는 `channel=apt`가 기록됩니다. 소스 파일이 없으면 GitPoulpi는 `apt` 명령을 보여 주는 대신 다음 `.deb`를 내려받으라고 안내합니다.

## MSI, Intune 또는 그룹 정책

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi`는 모든 사용자용으로 설치되며 관리되는 것을 전제로 합니다. GitPoulpi의 패널은 "관리자가 관리함"(관리자의 `message`와 함께)이라고만 알리며 MSI 설치는 절대 업데이트하지 않습니다. 새 MSI를 배포하여 업그레이드하세요. 메이저 업그레이드이므로 이전 버전이 교체됩니다.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| 속성 | 효과 |
|---|---|
| `INSTALLFOLDER` | 설치 폴더(기본값 `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1`(기본값)은 폴더를 컴퓨터 PATH에 추가하고, `0`은 추가하지 않습니다. |
| `UPDATES_MODE` | 정책 `mode`: `off`, `notify` 또는 `auto`. |
| `UPDATES_PIN` | 정책 `pin`. 예: `0.2` 또는 `0.2.4`. |
| `UPDATES_CHECK` | 정책 `check`: `github`, `nuget`, `manifest` 또는 `none`. |
| `UPDATES_CHECK_URL` | 정책 `check_url`. |
| `UPDATES_MESSAGE` | 업데이트 패널에 표시되는 정책 `message`. |

속성은 설정했을 때만 그 값을 `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`에 기록하므로, 설정하지 않은 속성은 레지스트리(와 정책 파일)를 건드리지 않습니다. **Intune:** 위 명령으로 MSI를 기간 업무용(LOB) 앱 또는 Win32 앱으로 배포하세요. **그룹 정책:** 소프트웨어 설치는 속성을 전달할 수 없으므로 변환(`.mst`)을 사용하거나 그룹 정책 기본 설정으로 레지스트리 값을 설정하세요(아래 참고).

## Homebrew, 내부 tap

공개 tap의 `gitpoulpi.rb`를 직접 호스팅하는 Git 저장소의 `Formula/`에 복사하고, `url` 줄을 릴리스 아카이브의 자체 사본으로 바꾸고 각 `sha256`(각 아카이브 옆 `.sha256` 파일의 첫 번째 필드)을 갱신합니다. 그다음:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

formula는 `channel=homebrew`가 담긴 마커를 설치합니다. GitPoulpi가 `brew upgrade gitpoulpi`를 직접 실행하며, 이 명령이 관리자의 tap을 읽습니다.

## Docker 레지스트리 미러

이미지 이름은 `poulpilab/gitpoulpi`(그리고 `ghcr.io/poulpi-lab/gitpoulpi`)입니다. 패널은 항상 이 이름을 표시하므로 미러는 투명해야 합니다.

- **풀스루 캐시:** Docker Hub의 프록시로 레지스트리를 실행하고(`registry` 이미지에 `proxy.remoteurl: https://registry-1.docker.io`), Docker 데몬의 `daemon.json`에 `"registry-mirrors": ["https://registry.example"]`을 설정합니다. 그러면 `docker pull poulpilab/gitpoulpi`가 캐시를 거칩니다.
- **복사:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`를 실행한 뒤 그 이름으로 실행합니다. 패널은 계속 공개 이름을 표시하지만, 버전을 알려 주는 표시로는 문제가 없습니다.

실행 방법은 [Docker에서 실행하기](docker.md)를 참고하세요.

## 정책 파일과 레지스트리

정책은 컴퓨터에서 업데이트가 동작하는 방식을 정하며 사용자가 바꿀 수 없습니다. 다음 위치에서 읽습니다.

| OS | 정책 파일 |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, 추가로 | 레지스트리 키 `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, 아래 각 키마다 같은 이름의 `REG_SZ` 값 하나(그룹 정책, Intune, MSI 속성용) |

파일에는 테이블이 하나 있습니다.

```toml
[updates]
mode = "notify"
release_channel = "stable"
pin = "1.4"
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
repo = "Poulpi-Lab/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "Updates are deployed by IT."
```

| 키 | 값 | 효과 |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | 상한입니다: `off` < `notify` < `auto`. 사용자는 더 낮은 값을 고를 수 있지만 더 높은 값은 고를 수 없습니다. `off`는 확인을 숨기고 요청을 보내지 않습니다. |
| `release_channel` | `stable`, `prerelease` | 업데이트 채널을 강제합니다. |
| `pin` | `1.4` 또는 `1.4.2` | 제공하는 최고 버전: `1.4`는 모든 `1.4.x`를 허용하고 `1.4.2`는 최대 `1.4.2`까지 허용합니다. |
| `check` | `github`, `nuget`, `manifest`, `none` | "최신 릴리스가 무엇인가"를 묻는 곳. 설치별 기본값은 GitHub이며 Chocolatey 채널은 Chocolatey 피드입니다. `none`은 확인하지 않습니다. |
| `check_url` | URL | `nuget`일 때: 피드. `manifest`일 때: `latest.json`의 주소. `github`일 때: GitHub Enterprise Server용 GitHub API 기본 주소(`repo` 추가). |
| `repo` | `owner/name` | `check = "github"`로 확인하는 저장소. |
| `download_base` | URL | 아카이브와 설치 프로그램 채널을 위한 릴리스 파일의 미러(같은 파일 이름). 이 호스트에서의 다운로드가 GitHub에 더해 허용됩니다. |
| `allow_self_update` | `true`, `false` | `false`이면 모든 "GitPoulpi의 파일 교체"와 "패키지 관리자 실행" 동작이 안내 문구나 복사할 명령으로 바뀝니다. 기본값 `true`. |
| `message` | 텍스트 | 업데이트 패널에 표시됩니다("관리자 메시지: ..."). |

**우선순위**(강한 순): 레지스트리, 정책 파일, 사용자의 `config.toml` `[updates]`, 마커의 `feed=`, 내장 기본값. 정책이 설정한 키는 설정에서 **관리자가 설정함**으로 표시되며 거기서 편집할 수 없습니다.

**안전하게 거부:** 읽을 수 없는 정책 파일이나 레지스트리 값(TOML 구문 오류, `mode = "sometimes"`, 버전이 아닌 `pin`)은 `mode = notify`와 `allow_self_update = false`로 처리되며 애플리케이션 로그에 한 줄이 남습니다. 오타 때문에 전체 컴퓨터가 스스로 업데이트하는 일은 없습니다. 알 수 없는 키는 경고와 함께 무시됩니다. Windows에서 관리자 권한을 가진 단계는 컴퓨터 정책만 읽으며 사용자의 설정은 읽지 않습니다.

### 예제

업데이트 끄기(잠긴 이미지, 망 분리 네트워크):

```toml
[updates]
mode = "off"
```

전체 컴퓨터를 1.4 계열로 고정하고, 알림만 제공하며, 문의할 곳을 알리기:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

사내 Chocolatey(NuGet) 피드 확인:

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Windows에서 레지스트리를 통한 같은 종류의 정책(PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json과 download_base: 릴리스 미러

릴리스의 파일(아카이브, 설치 프로그램, `.sha256` 파일, `latest.json`, **모든 `.minisig` 서명 파일**, `security.json`)을 이름을 유지한 채 사내 웹 서버의 폴더 하나에 복사합니다. `latest.json`은 모든 릴리스에 첨부됩니다.

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

그러면 정책은 다음과 같습니다.

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

아카이브와 설치 프로그램 채널은 매니페스트를 확인하고 `download_base`에서 내려받으며, GitHub에서 받을 때와 같은 검증을 거칩니다. 미러는 파일을 복사하기만 하면 되며 API가 필요 없습니다.

서명이 파일과 함께 전달되므로 미러가 서명을 약화할 수 없습니다.

- `latest.json`은 옆에 `latest.json.minisig`가 있고 GitPoulpi에 내장된 프로젝트 키로 검증될 때만 신뢰됩니다. 서명이 없거나 서명 뒤에 수정된 매니페스트는 거부됩니다(다운로드는 그 안에 있는 해시와 대조하여 검사합니다).
- 각 아카이브는 같은 호스트의 `<archive>.minisig`와 함께 내려받으며, 그 서명이 검증되고 이 파일과 이 버전을 명시할 때만 설치됩니다. 아카이브는 복사했지만 `.minisig`는 복사하지 않은 미러에서는 "서명 없음"으로 업데이트가 실패하고 아무것도 바뀌지 않습니다.
- `security.json`(보안 공지, [업데이트](updates.md#보안-공지) 참고)과 `security.json.minisig`는 매니페스트 옆에서 찾습니다. 이를 복사하지 않은 미러는 단순히 공지를 표시하지 않습니다.

파일은 변경 없이 복사하세요. 서명은 정확한 바이트를 대상으로 합니다. 보안 공지는 새 빌드 없이 최신 릴리스에 대해 다시 게시되므로, 미러를 새로 고칠 때마다 릴리스 페이지에서 `latest.json`, `security.json`과 그 `.minisig` 파일도 새로 고치세요.

## 기본 config.toml 배포

GitPoulpi에는 컴퓨터 전체 `config.toml`이 없습니다. 설정은 각 사용자의 폴더(`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`)에 있습니다. 새 사용자에게 기본값을 주려면, 사용자에게 설정이 없을 때 원하는 키만 담은 간단한 `config.toml`([설정 참조](config-reference.md) 참고)을 그 폴더에 복사하세요(`ui.theme`나 `ui.language` 같은 인터페이스 기본값은 그 옆의 간단한 `tui.toml`에 넣으며, 터미널 UI가 따로 읽습니다. [두 파일](config-reference.md#두-파일-configtoml과-tuitoml) 참고). Windows에서는 로그온 또는 Intune 스크립트, Linux에서는 `/etc/skel/.config/gitpoulpi/`, macOS에서는 첫 로그인 스크립트를 사용합니다. GitPoulpi의 패키지는 설정을 덮어쓰지 않으므로 관리는 관리자의 몫입니다. 반드시 지켜져야 하는 업데이트 규칙은 여기에 넣지 말고, 사용자가 재정의할 수 없는 정책을 사용하세요. 이식 가능한 설정이나 작업별 설정은 `GITPOULPI_CONFIG_DIR`로 선택할 수도 있습니다.

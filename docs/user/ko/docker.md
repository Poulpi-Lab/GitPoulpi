[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · **한국어** · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · [简体中文](../zh-CN/docker.md)

# Docker에서 실행하기

`poulpilab/gitpoulpi` 이미지에는 `debian:bookworm-slim` 위에 공식 릴리스 바이너리(`gitpoulpi`와 릴레이 `gitpoulpi-tui`, `gitpoulpi-mcp`)와 `git`, `git-lfs`, `openssh-client`, `less`, `tini`가 들어 있습니다. `linux/amd64`와 `linux/arm64`에서 실행됩니다. Docker Hub에 있습니다.

## UI 시작하기

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it`는 UI에 터미널을 제공합니다. 없으면 프로그램이 그릴 곳이 없습니다.
- `-v "$PWD":/work`는 현재 폴더를 작업 디렉터리(`/work`)로 마운트합니다. 그곳에서 저장소를 열거나, `/work` 아래에 저장소 여러 개를 마운트하세요.
- `-v gitpoulpi-config:/config`는 실행 사이에 설정과 세션을 보관하는 이름 있는 볼륨입니다.
- `--user "$(id -u):$(id -g)"`는 본인 계정으로 실행하므로, GitPoulpi가 저장소에 만드는 파일의 소유자가 `root`가 아니라 호스트의 본인으로 유지됩니다. 이미지가 쓰는 모든 폴더는 누구나 쓸 수 있으므로 어떤 사용자 id도 동작합니다.

## 볼륨과 경로

| 경로 | 내용 |
|---|---|
| `/work` | 작업 디렉터리. 저장소를 여기에 바인드 마운트하세요. |
| `/config` | 설정(`config.toml`), 세션, 그리고 하위 폴더로 데이터와 캐시(`GITPOULPI_CONFIG_DIR=/config`). 보관하려면 이름 있는 볼륨이나 바인드 마운트를 사용하세요. |
| `/data`, `/cache` | XDG 변수를 따르는 도구를 위해 볼륨으로도 선언되어 있습니다(`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. 볼륨이 아니며 컨테이너마다 새로 만들어집니다. |

`/config`에 볼륨이 없으면 `--rm` 실행마다 기본 설정으로 시작합니다. 자신의 `config.toml`로 시작하려면 마운트하세요: `-v "$PWD/config.toml":/config/config.toml:ro` (읽기 전용이면 설정을 저장할 수 없습니다).

## 다른 사용자가 소유한 저장소

Git은 소유자가 실행하는 사용자와 다른 저장소를 거부합니다("dubious ownership"). 위처럼 `--user`를 쓰면 소유자가 일치합니다. 일치하지 않을 때(다른 계정에서 바인드 마운트, Docker Desktop 공유)는 다음으로 모든 디렉터리를 신뢰하세요.

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

그러면 진입점이 UI를 시작하기 전에 `git config --global --add safe.directory '*'`를 실행합니다. 컨테이너 전체에 대해 git의 보호를 해제하므로 신뢰하는 폴더에만 사용하세요.

## SSH

키를 컨테이너에 복사하지 말고 SSH 에이전트를 전달하세요.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop(macOS, Windows)은 호스트 에이전트를 VM 안의 고정 경로에 노출합니다.

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

`--user`를 쓸 때는 소켓을 사용자 id가 읽을 수 있어야 하며, 본인 소유이면 그렇습니다. 컨테이너에는 `known_hosts`가 없습니다. 호스트에 처음 연결하면 키를 수락할지 묻거나, `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`로 자신의 파일을 마운트하세요.

## Git 신원과 자격 증명

- **신원과 설정:** Git 설정을 읽기 전용으로 마운트하세요: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. 또는 `/config`에 저장되는 GitPoulpi 프로필(설정 > 프로필)에서 commit 신원을 설정하세요.
- **HTTPS 자격 증명:** 이미지에는 키체인 같은 자격 증명 도우미가 없습니다. 자격 증명 저장소를 마운트하거나(`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, 마운트한 `.gitconfig`에 `credential.helper = store`), 에이전트를 전달한 SSH remote를 사용하세요.
- **호스팅 토큰**(pull request, 이슈): 자격 증명 도우미가 없으면 GitPoulpi는 토큰을 세션 동안 메모리에만 보관합니다. 지속적으로 쓰려면 `-e GITPOULPI_TOKEN_<ID>=...`로 환경 변수를 통해 전달하세요. `<ID>`는 연동의 id를 대문자로 바꾸고 영숫자가 아닌 문자를 `_`로 바꾼 것입니다([호스팅 및 이슈 트래커 연동](hosting-integration.md) 참고). 연동 자체는 `/config`의 `config.toml`에 있습니다.

## 터미널, 색상, 마우스

색상과 마우스 지원은 터미널 에뮬레이터에서 옵니다. Docker는 `-t`에서 `TERM`을 전달합니다. 색상이나 트루컬러 테마가 이상하면 `-e TERM -e COLORTERM`을 추가하세요. 예: `-e COLORTERM=truecolor`. 마우스 클릭, 드래그, 휠은 평소처럼 터미널을 통해 전달됩니다. commit 그래프는 문자로 그려집니다. 그림(kitty, iTerm2, sixel)은 모든 터미널과 멀티플렉서를 거쳐 전달되지는 않으므로 [터미널 고르기](terminals.md)를 참고하세요. 터미널 창 크기를 바꾸면 UI 크기도 바뀝니다. 컨테이너가 크기 변경을 전달받습니다.

## CLI와 MCP

진입점은 UI를 시작합니다. 다른 프로그램은 진입점을 재정의하여 사용할 수 있습니다.

```
# 스크립트용 CLI (스크립트에는 --json 추가)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# 표준 입출력을 통한 MCP 서버, 이를 실행하는 에이전트용
docker run -i --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi mcp serve
```

각 `docker run`은 자체 GitPoulpi 백그라운드 데몬을 가진 별도의 컨테이너이므로, CLI로 한 변경이 다른 컨테이너에서 실행 중인 UI에 실시간으로 나타나지 않습니다. CLI로 UI 하나를 조작하려면 둘을 같은 컨테이너에서 실행하세요. UI를 `--name gp`로 시작하고 `docker exec gp gitpoulpi ...`를 사용합니다. 에이전트에는 `docker run -i --rm --entrypoint gitpoulpi ... mcp serve`를 MCP 서버 명령으로 등록하세요([MCP와 에이전트](mcp-and-agents.md) 참고). 마운트한 `config.toml`의 MCP 권한 스위치는 평소처럼 적용됩니다.

## 태그와 업데이트

| 태그 | 의미 |
|---|---|
| `X.Y.Z` | 정확히 하나의 릴리스. |
| `X.Y` | 해당 마이너 버전의 최신 패치(안정 릴리스만). |
| `latest` | 최신 안정 릴리스. |

프리릴리스는 정확한 `X.Y.Z-...` 태그만 받습니다.

이미지는 제자리에서 업데이트하지 않고 교체합니다. 컨테이너 안의 GitPoulpi도 이를 압니다(설치 기록이 "Docker 이미지"라고 말합니다). [업데이트 패널](updates.md)은 버전과 명령을 표시할 뿐 아무것도 교체하지 않습니다. 업데이트하려면:

```
docker pull poulpilab/gitpoulpi
```

그다음 새 컨테이너를 시작하세요. 설정은 `/config` 볼륨에 그대로 남습니다. 이전으로 돌아가려면 이전 태그를 실행하세요: `docker pull poulpilab/gitpoulpi:<previous>`. 스크립트에서 버전을 고정하려면 `latest` 대신 정확한 태그를 사용하세요. 조직은 자체 레지스트리에 이미지를 미러링할 수 있습니다. [엔터프라이즈 배포](enterprise-distribution.md#docker-레지스트리-미러)를 참고하세요.

[English](../docker.md) · [Čeština](../cs/docker.md) · [Deutsch](../de/docker.md) · [Español](../es/docker.md) · [Français](../fr/docker.md) · [Italiano](../it/docker.md) · [日本語](../ja/docker.md) · [한국어](../ko/docker.md) · [Nederlands](../nl/docker.md) · [Polski](../pl/docker.md) · [Português (Brasil)](../pt-BR/docker.md) · [Português (Portugal)](../pt-PT/docker.md) · [Русский](../ru/docker.md) · [Svenska](../sv/docker.md) · [Türkçe](../tr/docker.md) · [Українська](../uk/docker.md) · **简体中文**

# 在 Docker 中运行

`poulpilab/gitpoulpi` 镜像基于 `debian:bookworm-slim`，内含官方发布的可执行文件（`gitpoulpi-tui`、`gitpoulpi`、`gitpoulpi-mcp`），以及 `git`、`git-lfs`、`openssh-client`、`less` 和 `tini`。它可运行在 `linux/amd64` 和 `linux/arm64` 上。镜像位于 Docker Hub；相同的标签也在 `ghcr.io/poulpi-lab/gitpoulpi` 上，下面的所有内容用任一名称都适用。

## 启动界面

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` 给界面一个终端。没有它，程序就没有可以绘制的地方。
- `-v "$PWD":/work` 把当前文件夹挂载为工作目录（`/work`）。从那里打开仓库，或在 `/work` 下挂载多个仓库。
- `-v gitpoulpi-config:/config` 是一个具名卷，让你的设置和会话在多次运行之间得以保留。
- `--user "$(id -u):$(id -g)"` 以你的身份运行，这样 GitPoulpi 在你的仓库中创建的文件会保留你在主机上的所有者，而不是 `root`。镜像写入的每个文件夹都是所有人可写的，所以任何用户 id 都能用。

## 卷和路径

| 路径 | 内容 |
|---|---|
| `/work` | 工作目录。把你的仓库绑定挂载到这里。 |
| `/config` | 设置（`config.toml`）、会话，以及作为子文件夹的数据和缓存（`GITPOULPI_CONFIG_DIR=/config`）。请使用具名卷或绑定挂载来保留它们。 |
| `/data`、`/cache` | 同样声明为卷，供遵循 XDG 变量的工具使用（`XDG_DATA_HOME=/data`、`XDG_CACHE_HOME=/cache`）。 |
| `/home/gitpoulpi` | `HOME`。它不是卷：每个容器都是全新的。 |

如果 `/config` 没有卷，每次 `--rm` 运行都从默认设置开始。要从你自己的 `config.toml` 开始，请挂载它：`-v "$PWD/config.toml":/config/config.toml:ro`（只读意味着设置无法保存）。

## 由其他用户拥有的仓库

当仓库的所有者与运行 git 的用户不同时，git 会拒绝它（“dubious ownership”）。像上面那样使用 `--user` 时，所有者是一致的。如果不一致（来自另一个账户的绑定挂载、Docker Desktop 的共享），可以用下面的方式信任所有目录：

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

入口程序随后会在启动界面之前运行 `git config --global --add safe.directory '*'`。只对你信任的文件夹使用它，因为它会解除整个容器中 git 的保护。

## SSH

请转发你的 SSH 代理，而不要把密钥复制到容器中。

Linux：

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop（macOS、Windows）在其虚拟机内的固定路径上提供主机的代理：

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

使用 `--user` 时，该套接字必须能被你的用户 id 读取，属于你自己的套接字就可以。容器中没有 `known_hosts`：第一次连接某台主机时会请你接受它的密钥，或者用 `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro` 挂载你自己的文件。

## Git 身份和凭据

- **身份和设置：** 以只读方式挂载你的 Git 配置：`-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`。或者在 GitPoulpi 的配置文件中设置 commit 身份（设置 > 配置），它保存在 `/config` 中。
- **HTTPS 凭据：** 镜像没有钥匙串之类的凭据助手。要么挂载一个凭据存储（`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`，并在挂载的 `.gitconfig` 中设置 `credential.helper = store`），要么使用 SSH 远程并转发代理。
- **托管平台令牌**（拉取请求、议题）：没有凭据助手时，GitPoulpi 只在会话期间把令牌保存在内存中。要长期使用，请通过环境变量传入：`-e GITPOULPI_TOKEN_<ID>=...`，其中 `<ID>` 是集成的 id 转为大写，非字母数字字符换成 `_`（见[托管平台与议题跟踪器集成](hosting-integration.md)）。集成本身在 `/config` 下的 `config.toml` 中。

## 终端、颜色和鼠标

颜色和鼠标支持来自你的终端模拟器。Docker 会沿用 `-t` 传递 `TERM`；当颜色或真彩色主题看起来不对时，请加上 `-e TERM -e COLORTERM`，例如 `-e COLORTERM=truecolor`。鼠标点击、拖动和滚轮照常通过终端送达。提交图用字符绘制；图片（kitty、iTerm2、sixel）并非在每个终端和多路复用器中都会被转发，见[选择终端](terminals.md)。调整终端窗口大小会同步调整界面：容器会收到大小变化。

## CLI 和 MCP

入口程序启动的是界面。其他程序可以通过覆盖入口来使用：

```
# 可编写脚本的 CLI（脚本中加 --json）
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# MCP 服务器，通过标准输入和输出，供启动它的代理使用
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

每次 `docker run` 都是一个独立的容器，有它自己的 GitPoulpi 后台守护进程，所以通过 CLI 所做的更改不会实时显示在另一个容器中运行的界面里。要用 CLI 操控某个界面，请在同一个容器中运行两者：用 `--name gp` 启动界面，然后使用 `docker exec gp gitpoulpi ...`。对于代理，请把 `docker run -i --rm --entrypoint gitpoulpi-mcp ...` 注册为 MCP 服务器命令，见 [MCP 与代理](mcp-and-agents.md)；挂载的 `config.toml` 中的 MCP 权限开关照常生效。

## 标签和更新

| 标签 | 含义 |
|---|---|
| `X.Y.Z` | 某一个确切的发布版本。 |
| `X.Y` | 该次版本的最新补丁（仅限稳定版本）。 |
| `latest` | 最新的稳定版本。 |

预发布版本只有它确切的 `X.Y.Z-...` 标签。

镜像是被替换的，绝不原地更新。容器内的 GitPoulpi 知道这一点（它的安装记录写的是“Docker 镜像”）：[更新面板](updates.md)会显示版本和命令，不会替换任何内容。要更新：

```
docker pull poulpilab/gitpoulpi
```

然后启动一个新容器。你的设置保留在 `/config` 卷中。要回退，请运行较旧的标签：`docker pull poulpilab/gitpoulpi:<previous>`。要在脚本中固定版本，请使用确切的标签而不是 `latest`。组织可以在自己的镜像仓库中镜像该镜像，见[企业分发](enterprise-distribution.md#docker-镜像仓库镜像)。

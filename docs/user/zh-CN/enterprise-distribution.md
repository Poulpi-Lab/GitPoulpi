[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · [Русский](../ru/enterprise-distribution.md) · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · **简体中文**

# 企业分发

面向从自己的服务器向许多台电脑部署 GitPoulpi 的管理员：内部软件包源、镜像、整台机器范围的更新策略和默认配置。请先阅读[安装](installation.md)和[更新](updates.md)。

这里的一切都由两个事实决定：

- 每次安装都会在程序旁的 `.gitpoulpi-dist` 标记中记录**它是怎样安装的**（`channel=...`）。GitPoulpi 只按该通道允许的方式更新自己：它绝不会替换由包管理器拥有的文件，也绝不会碰 MSI。
- **策略**（一个文件，在 Windows 上还有注册表）由你掌控。用户自己的 `config.toml` 永远无法放宽它。

## Chocolatey，使用私有源

公共软件包会从 GitHub 下载安装程序。对于封闭网络，请把它内部化：

1. 获取软件包（用带许可的 Chocolatey 工具运行 `choco download gitpoulpi --internalize`，或者下载 `.nupkg` 并解包）。
2. 在 `tools\chocolateyinstall.ps1` 中，把 `url64bit` 指向你自己保存的 `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` 副本，并让 `checksum64` 等于该发布版本 `.sha256` 文件的第一个字段（或你那份副本的校验和）。
3. 运行 `choco pack`，然后把 `.nupkg` 推送到你的源（Nexus、Artifactory、ProGet、文件共享）。

用软件包参数安装：

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| 参数 | 作用 |
|---|---|
| `/Channel:choco-internal` | 在标记中记录 `channel=choco-internal`，让 GitPoulpi 知道这个源是你的，而不是 Chocolatey 的社区源。 |
| `/Feed:<url>` | 记录源地址（标记中的 `feed=`）。GitPoulpi 会到该地址检查新版本，并运行 `choco upgrade gitpoulpi -y --source=<feed>`。 |

不带参数时，通道是 `choco`，检查会去 Chocolatey 社区源。该软件包为所有用户安装到 Program Files，不带 `gp` 别名。

GitPoulpi 的更新运行 `choco upgrade` 时不带 `--params`，这样你部署的任何内容都不会被动到。因此 Chocolatey 必须记住第一次安装时的软件包参数（`useRememberedArgumentsForUpgrades`，自 Chocolatey 2.0 起默认开启；在更旧的版本上用 `choco feature enable -n useRememberedArgumentsForUpgrades`）。否则升级会记录成普通的 `choco` 通道。策略的 `check_url`（见下文）会让检查和升级无论标记怎么写都指向你的源，并且它优先于标记中的源。

**软件包中没有任何内容会写入 `policy.toml` 或 `config.toml`。** 你部署的内容在安装时和每次升级时都会保留。见[部署默认 config.toml](#部署默认-configtoml)。

## Scoop，使用私有 bucket

把 `gitpoulpi.json`（公共 bucket 中的清单）放进你托管的某个 Git 仓库的 `bucket/` 中，把 `url` 改成你保存的发布 zip 副本，并把 `hash` 设为它的 sha256（如果你的副本沿用发布版本的命名，可以保留 `autoupdate`）。然后：

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

清单的 `post_install` 会用 `channel=scoop` 重写标记；之后 GitPoulpi 自己运行 `scoop update gitpoulpi`，绝不会在 Scoop 背后替换文件。Scoop 按用户安装，不需要管理员权限。

## apt，使用镜像

该软件源是一棵普通的目录树（`dists/`、`pool/`、签名密钥的 `gitpoulpi-archive-keyring.gpg` 和 `.asc`，以及示例 `gitpoulpi.sources`），通过 HTTPS 提供。用 `wget --mirror`、`rsync` 或你的制品管理器镜像它，在内部提供服务，并在每台电脑上安装密钥和源：

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`：

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

请保留文件名 `gitpoulpi.sources`（或 `gitpoulpi.list`）：只有找到 `/etc/apt/sources.list.d/gitpoulpi.*` 时，GitPoulpi 才会显示 `apt` 命令。请使用项目的密钥：镜像仍由它签名，而 `Signed-By` 只为这个软件源固定那一把密钥。软件源会保留每个软件包最近五个版本。

**离线 `.deb`：** 每个发布版本都附带 `gitpoulpi_<version>_<arch>.deb`。把它复制过去，然后运行 `sudo apt install ./gitpoulpi_<version>_<arch>.deb`。标记中写的是 `channel=apt`；没有源文件时，GitPoulpi 会提示用户下载下一个 `.deb`，而不是显示 `apt` 命令。

## MSI，使用 Intune 或组策略

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` 为所有用户安装，设计上就是用来被管理的：GitPoulpi 的面板只会说“由你的管理员管理”（附上你的 `message`），绝不会更新 MSI 安装。升级请部署新的 MSI；它是主要升级，所以旧版本会被替换。

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| 属性 | 作用 |
|---|---|
| `INSTALLFOLDER` | 安装文件夹（默认 `Program Files\GitPoulpi`）。 |
| `ADDTOPATH` | `1`（默认）把该文件夹加入机器的 PATH，`0` 则不加。 |
| `UPDATES_MODE` | 策略 `mode`：`off`、`notify` 或 `auto`。 |
| `UPDATES_PIN` | 策略 `pin`，例如 `0.2` 或 `0.2.4`。 |
| `UPDATES_CHECK` | 策略 `check`：`github`、`nuget`、`manifest` 或 `none`。 |
| `UPDATES_CHECK_URL` | 策略 `check_url`。 |
| `UPDATES_MESSAGE` | 策略 `message`，显示在更新面板中。 |

只有当你设置某个属性时，它的值才会写入 `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`，所以未设置的属性不会动注册表（和策略文件）。**Intune：** 用上面的命令把 MSI 部署为业务线应用或 Win32 应用。**组策略：** 软件安装无法传递属性，所以要么使用转换文件（`.mst`），要么用组策略首选项设置注册表值，见下文。

## Homebrew，使用内部 tap

把公共 tap 中的 `gitpoulpi.rb` 复制到你托管的某个 Git 仓库的 `Formula/` 中，把 `url` 行改成你保存的发布压缩包副本，并更新每个 `sha256`（每个压缩包旁 `.sha256` 文件的第一个字段）。然后：

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

该 formula 会用 `channel=homebrew` 安装标记：GitPoulpi 自己运行 `brew upgrade gitpoulpi`，它读取的是你的 tap。

## Docker 镜像仓库镜像

镜像名称是 `poulpilab/gitpoulpi`（以及 `ghcr.io/poulpi-lab/gitpoulpi`）。面板始终显示这个名称，所以镜像必须是透明的。

- **拉取式缓存：** 运行一个作为 Docker Hub 代理的镜像仓库（`registry` 镜像，带 `proxy.remoteurl: https://registry-1.docker.io`），并在 Docker 守护进程的 `daemon.json` 中设置 `"registry-mirrors": ["https://registry.example"]`。这样 `docker pull poulpilab/gitpoulpi` 就会经过你的缓存。
- **复制：** 依次运行 `docker pull poulpilab/gitpoulpi:<v>`、`docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`、`docker push registry.example/tools/gitpoulpi:<v>`，然后运行那个名称。面板仍显示公共名称，作为版本的指引这没有问题。

如何运行见[在 Docker 中运行](docker.md)。

## 策略文件和注册表

策略设定更新在一台电脑上的行为，用户无法更改。它从以下位置读取：

| 操作系统 | 策略文件 |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows，另外 | 注册表项 `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`，下面每个键对应一个同名的 `REG_SZ` 值（用于组策略、Intune 和 MSI 属性） |

该文件只有一个表：

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

| 键 | 取值 | 作用 |
|---|---|---|
| `mode` | `off`、`notify`、`auto` | 上限：`off` < `notify` < `auto`。用户可以选择更低的值，但不能更高。`off` 会隐藏检查并且不发出任何请求。 |
| `release_channel` | `stable`、`prerelease` | 强制指定更新通道。 |
| `pin` | `1.4` 或 `1.4.2` | 提供的最高版本：`1.4` 允许任何 `1.4.x`，`1.4.2` 最多允许到 `1.4.2`。 |
| `check` | `github`、`nuget`、`manifest`、`none` | 向哪里询问“最新发布版本是什么”。各安装方式的默认值：GitHub，Chocolatey 通道则是 Chocolatey 源。`none` 从不检查。 |
| `check_url` | URL | 与 `nuget` 搭配：源地址。与 `manifest` 搭配：`latest.json` 的地址。与 `github` 搭配：GitHub API 的基础地址，用于 GitHub Enterprise Server（同时加上 `repo`）。 |
| `repo` | `owner/name` | 用 `check = "github"` 检查的仓库。 |
| `download_base` | URL | 发布文件的镜像（文件名相同），用于压缩包和安装程序通道。除 GitHub 的主机外，也允许从该主机下载。 |
| `allow_self_update` | `true`、`false` | `false` 会把每个“替换 GitPoulpi 的文件”和“运行包管理器”的操作变成一条说明或一条可复制的命令。默认 `true`。 |
| `message` | 文本 | 显示在更新面板中（“来自你的管理员：……”）。 |

**优先级**，从强到弱：注册表、策略文件、用户 `config.toml` 的 `[updates]`、标记中的 `feed=`、内置默认值。策略设定的键在设置中显示为**由管理员设定**，且无法在那里编辑。

**失败即关闭：** 无法读取的策略文件或注册表值（TOML 语法错误、`mode = "sometimes"`、不是版本号的 `pin`）会被当作 `mode = notify` 和 `allow_self_update = false`，并在应用日志中记一行。一个拼写错误绝不会让整批电脑自行更新。未知的键会被忽略并给出警告。在 Windows 上，持有管理员权限的那一步只读取机器策略，绝不读取用户的配置。

### 示例

关闭更新（锁定的镜像、物理隔离的网络）：

```toml
[updates]
mode = "off"
```

把整批电脑固定在 1.4 系列，只通知，并说明该找谁：

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

检查内部 Chocolatey（NuGet）源：

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

在 Windows 上通过注册表设置同类策略（PowerShell）：

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json 与 download_base：发布版本的镜像

把一个发布版本的文件（压缩包、安装程序、`.sha256` 文件、`latest.json`、**每一个 `.minisig` 签名文件**以及 `security.json`）复制到内部网络服务器的一个文件夹中，保持文件名不变。每个发布版本都附带 `latest.json`：

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

然后策略如下：

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

压缩包和安装程序通道会检查该清单，并从 `download_base` 下载，校验方式与从 GitHub 下载时相同。镜像只需复制文件，不需要 API。

签名随文件一起传递，所以镜像无法削弱它们：

- 只有当 `latest.json.minisig` 就在它旁边，并且能用内置在 GitPoulpi 中的项目密钥校验通过时，`latest.json` 才会被信任。没有签名的清单，或在签名之后被编辑过的清单，会被拒绝（清单里的哈希是下载内容的校验依据）。
- 每个压缩包都会与来自同一主机的 `<archive>.minisig` 一起下载，只有当该签名校验通过并写明此文件和此版本时才会安装。复制了压缩包却没有复制其 `.minisig` 的镜像，会使更新以“没有签名”失败，且不会更改任何内容。
- `security.json`（安全公告，见[更新](updates.md#安全公告)）和 `security.json.minisig` 会在清单旁边查找。没有复制它们的镜像只是不显示公告。

请原样复制文件：签名覆盖的是确切的字节。安全公告会在不重新构建的情况下为最新发布版本重新发布，所以每次刷新镜像时，都要从发布页面刷新 `latest.json`、`security.json` 及其 `.minisig` 文件。

## 部署默认 config.toml

GitPoulpi 没有整台机器范围的 `config.toml`：设置保存在每个用户的文件夹中（`%APPDATA%\GitPoulpi\config.toml`、`~/.config/gitpoulpi/config.toml`、`~/Library/Application Support/GitPoulpi/config.toml`）。要给新用户你的默认值，请在用户还没有配置时，把一份精简的 `config.toml`（只含你想要的键，见[配置参考](config-reference.md)）复制到该文件夹中（界面默认值，例如 `ui.theme` 或 `ui.language`，放进它旁边一份精简的 `tui.toml`，终端界面会单独读取它；见[两个文件](config-reference.md#两个文件configtoml-和-tuitoml)）：Windows 上用登录脚本或 Intune 脚本，Linux 上用 `/etc/skel/.config/gitpoulpi/`，macOS 上用首次登录脚本。GitPoulpi 的软件包绝不会覆盖配置，所以它由你来管理。必须始终生效的更新规则不要放在那里：请使用用户无法覆盖的策略。便携的或按任务区分的配置也可以用 `GITPOULPI_CONFIG_DIR` 来选择。

**English** · [Čeština](cs/enterprise-distribution.md) · [Deutsch](de/enterprise-distribution.md) · [Español](es/enterprise-distribution.md) · [Français](fr/enterprise-distribution.md) · [Italiano](it/enterprise-distribution.md) · [日本語](ja/enterprise-distribution.md) · [한국어](ko/enterprise-distribution.md) · [Nederlands](nl/enterprise-distribution.md) · [Polski](pl/enterprise-distribution.md) · [Português (Brasil)](pt-BR/enterprise-distribution.md) · [Português (Portugal)](pt-PT/enterprise-distribution.md) · [Русский](ru/enterprise-distribution.md) · [Svenska](sv/enterprise-distribution.md) · [Türkçe](tr/enterprise-distribution.md) · [Українська](uk/enterprise-distribution.md) · [简体中文](zh-CN/enterprise-distribution.md)

# Enterprise distribution

For administrators who deploy GitPoulpi to many computers from their own servers: internal package feeds, mirrors, a machine-wide update policy, and a default configuration. Read [Installation](installation.md) and [Updates](updates.md) first.

Two facts drive everything here:

- Every install records **how it was installed** in the `.gitpoulpi-dist` marker next to the programs (`channel=...`). GitPoulpi updates itself only the way that channel allows: it never replaces files that a package manager owns, and an MSI is never touched.
- The **policy** (a file, and on Windows the registry) is yours. A user's own `config.toml` can never loosen it.

## Chocolatey, with a private feed

The public package downloads the installer from GitHub. For a closed network, internalize it:

1. Get the package (`choco download gitpoulpi --internalize` with the licensed Chocolatey tooling, or download the `.nupkg` and unpack it).
2. In `tools\chocolateyinstall.ps1` point `url64bit` at your own copy of `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` and keep `checksum64` equal to the first field of the release's `.sha256` file (or the checksum of your copy).
3. `choco pack`, then push the `.nupkg` to your feed (Nexus, Artifactory, ProGet, a file share).

Install it with package parameters:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Parameter | Effect |
|---|---|
| `/Channel:choco-internal` | Records `channel=choco-internal` in the marker, so GitPoulpi knows the feed is yours and not Chocolatey's community feed. |
| `/Feed:<url>` | Records the feed address (`feed=` in the marker). GitPoulpi checks it for new versions and runs `choco upgrade gitpoulpi -y --source=<feed>`. |

Without parameters the channel is `choco` and the check goes to the Chocolatey community feed. The package installs for all users into Program Files, with no `gp` alias.

GitPoulpi's update runs `choco upgrade` without `--params`, so that nothing you deployed is touched. Chocolatey must therefore remember the package parameters of the first install (`useRememberedArgumentsForUpgrades`, on by default since Chocolatey 2.0; on older versions `choco feature enable -n useRememberedArgumentsForUpgrades`). Otherwise the upgrade would record the plain `choco` channel. A policy `check_url` (below) keeps the check and the upgrade pointed at your feed whatever the marker says, and it takes precedence over the marker's feed.

**Nothing in the package writes `policy.toml` or `config.toml`.** Whatever you deployed stays, on install and on every upgrade. See [Deploying a default config.toml](#deploying-a-default-configtoml).

## winget, with an internal source

winget updates GitPoulpi with `winget upgrade --id GitPoulpi.GitPoulpi --exact`, using whichever source the package came from. For an internal source, host a Windows Package Manager REST source (any implementation of the winget REST API), publish the three manifests of the release (`GitPoulpi.GitPoulpi.yaml`, `...installer.yaml`, `...locale.en-US.yaml`) with `InstallerUrl` pointing at your copy of the setup program and the matching `InstallerSha256`, then:

```
winget source add --name corp --arg https://winget.example/api --type Microsoft.Rest
winget install --id GitPoulpi.GitPoulpi --source corp
```

Keep the installer switches of the manifest: `/CHANNEL=winget` makes the marker say `winget`, and `/ALIAS=` leaves out the `gp` alias. winget does not tell GitPoulpi what is new, so the check still asks GitHub for the version. On a network without GitHub, point it at your own `latest.json` with a policy: `check = "manifest"` (see [latest.json](#latestjson-and-download_base-a-mirror-of-the-release)).

## Scoop, with a private bucket

Put `gitpoulpi.json` (the manifest from the public bucket) into `bucket/` of a Git repository you host, change `url` to your copy of the release zip and set `hash` to its sha256 (`autoupdate` can stay if your copy follows the release names). Then:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

The manifest's `post_install` rewrites the marker with `channel=scoop`; GitPoulpi then runs `scoop update gitpoulpi` itself and never swaps files behind Scoop. Scoop installs per user and needs no administrator rights.

## apt, with a mirror

The repository is a plain directory tree (`dists/`, `pool/`, the signing key's `gitpoulpi-archive-keyring.gpg` and `.asc`, and the sample `gitpoulpi.sources`) served over HTTPS. Mirror it with `wget --mirror`, `rsync` or your artifact manager, serve it internally, and install the key and the source on each computer:

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

Keep the file name `gitpoulpi.sources` (or `gitpoulpi.list`): GitPoulpi shows `apt` commands only when it finds `/etc/apt/sources.list.d/gitpoulpi.*`. Use the project's key: the mirror stays signed by it, and `Signed-By` pins that one key for this repository only. The repository keeps the last five versions of each package.

**Offline `.deb`:** every release attaches `gitpoulpi_<version>_<arch>.deb`. Copy it over and `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. The marker says `channel=apt`; without the source file GitPoulpi tells the user to download the next `.deb` instead of showing an `apt` command.

## MSI, with Intune or Group Policy

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` installs for all users and is meant to be managed: GitPoulpi's panel only says "managed by your administrator" (with your `message`), it never updates an MSI install. Upgrade by deploying the new MSI; it is a major upgrade, so the old version is replaced.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Property | Effect |
|---|---|
| `INSTALLFOLDER` | Install folder (default `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (default) adds the folder to the machine PATH, `0` does not. |
| `UPDATES_MODE` | Policy `mode`: `off`, `notify` or `auto`. |
| `UPDATES_PIN` | Policy `pin`, for example `0.2` or `0.2.4`. |
| `UPDATES_CHECK` | Policy `check`: `github`, `nuget`, `manifest` or `none`. |
| `UPDATES_CHECK_URL` | Policy `check_url`. |
| `UPDATES_MESSAGE` | Policy `message`, shown in the update panel. |

A property writes its value to `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` only when you set it, so an unset one leaves the registry (and a policy file) alone. **Intune:** deploy the MSI as a line-of-business or Win32 app with the command above. **Group Policy:** software installation cannot pass properties, so either use a transform (`.mst`) or set the registry values with Group Policy Preferences, see below.

## Homebrew, with an internal tap

Copy `gitpoulpi.rb` from the public tap into `Formula/` of a Git repository you host, change the `url` lines to your copies of the release archives and update each `sha256` (the first field of the `.sha256` file next to each archive). Then:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

The formula installs the marker with `channel=homebrew`: GitPoulpi runs `brew upgrade gitpoulpi` itself, which reads your tap.

## Docker registry mirror

The image name is `poulpilab/gitpoulpi` (and `ghcr.io/poulpi-lab/gitpoulpi`). The panel always shows that name, so a mirror has to be transparent.

- **Pull-through cache:** run a registry as a proxy of Docker Hub (the `registry` image with `proxy.remoteurl: https://registry-1.docker.io`) and set `"registry-mirrors": ["https://registry.example"]` in the Docker daemon's `daemon.json`. `docker pull poulpilab/gitpoulpi` then goes through your cache.
- **Copy:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, then run that name. The panel still shows the public one, which is fine as a pointer to the version.

See [Running in Docker](docker.md) for how to run it.

## The policy file and the registry

The policy sets how updates behave on a computer and cannot be changed by users. It is read from:

| OS | Policy file |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, also | the registry key `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, one `REG_SZ` value per key below, with the same names (for Group Policy, Intune and MSI properties) |

The file has one table:

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

| Key | Values | Effect |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | A ceiling: `off` < `notify` < `auto`. Users may choose a lower value, never a higher one. `off` hides the check and makes no request. |
| `release_channel` | `stable`, `prerelease` | Forces the update channel. |
| `pin` | `1.4` or `1.4.2` | The highest version offered: `1.4` allows any `1.4.x`, `1.4.2` allows at most `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Where "what is the latest release" is asked. Default per installation: GitHub, the Chocolatey feed for the Chocolatey channels. `none` never checks. |
| `check_url` | URL | With `nuget`: the feed. With `manifest`: the address of `latest.json`. With `github`: the GitHub API base, for GitHub Enterprise Server (add `repo`). |
| `repo` | `owner/name` | The repository checked with `check = "github"`. |
| `download_base` | URL | A mirror of the release files (same file names) for the archive and installer channels. Downloads from this host are allowed in addition to GitHub's. |
| `allow_self_update` | `true`, `false` | `false` turns every "replace GitPoulpi's files" and "run the package manager" action into a note or a command to copy. Default `true`. |
| `message` | text | Shown in the update panel ("From your administrator: ..."). |

**Precedence**, strongest first: the registry, the policy file, the user's `config.toml` `[updates]`, the marker's `feed=`, the built-in defaults. A key the policy sets is **Set by your administrator** in Settings and cannot be edited there.

**Fail closed:** a policy file or registry value that cannot be read (a TOML syntax error, `mode = "sometimes"`, a `pin` that is not a version) is treated as `mode = notify` and `allow_self_update = false`, and one line goes to the application log. A typo never leaves a fleet updating itself. An unknown key is ignored with a warning. On Windows the step that holds administrator rights reads the machine policy only, never the user's config.

### Examples

Turn updates off (a locked image, an air-gapped network):

```toml
[updates]
mode = "off"
```

Pin the fleet to the 1.4 line, notify only, and say who to ask:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Check an internal Chocolatey (NuGet) feed:

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

The same kind of policy on Windows through the registry (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json and download_base: a mirror of the release

Copy a release's files (the archives, the setup, the `.sha256` files, `latest.json`, **every `.minisig` signature file** and `security.json`) into one folder of an internal web server, keeping the names. `latest.json` is attached to every release:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Then the policy is:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Archive and installer channels check the manifest and download from `download_base`, with the same verification as from GitHub. A mirror only has to copy files; it needs no API.

The signatures travel with the files, so a mirror cannot weaken them:

- `latest.json` is trusted only when `latest.json.minisig` sits next to it and verifies against the project key built into GitPoulpi. A manifest without its signature, or edited after it was signed, is refused (the hashes inside it are what the downloads are checked against).
- Each archive is downloaded together with `<archive>.minisig` from the same host and is installed only when that signature verifies and names this file and this version. A mirror that copied the archive but not its `.minisig` makes the update fail with "no signature", and nothing is changed.
- `security.json` (the security notices, see [Updates](updates.md#security-notices)) and `security.json.minisig` are looked for next to the manifest. A mirror that does not copy them simply shows no notices.

Copy the files unchanged: a signature covers the exact bytes. The security notices are republished for the latest release without a new build, so refresh `latest.json`, `security.json` and their `.minisig` files from the release page whenever you refresh the mirror.

## Deploying a default config.toml

GitPoulpi has no machine-wide `config.toml`: settings live in each user's folder (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). To give new users your defaults, copy a sparse `config.toml` (only the keys you want, see the [Configuration reference](config-reference.md)) into that folder when the user has none: a logon or Intune script on Windows, `/etc/skel/.config/gitpoulpi/` on Linux, a first-login script on macOS. Nothing in GitPoulpi's packages ever overwrites a config, so it is yours to manage. Do not put update rules there when they must hold: use the policy, which a user cannot override. A portable or per-job config can also be chosen with `GITPOULPI_CONFIG_DIR`.

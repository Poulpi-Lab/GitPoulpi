**English** · [Čeština](cs/docker.md) · [Deutsch](de/docker.md) · [Español](es/docker.md) · [Français](fr/docker.md) · [Italiano](it/docker.md) · [日本語](ja/docker.md) · [한국어](ko/docker.md) · [Nederlands](nl/docker.md) · [Polski](pl/docker.md) · [Português (Brasil)](pt-BR/docker.md) · [Português (Portugal)](pt-PT/docker.md) · [Русский](ru/docker.md) · [Svenska](sv/docker.md) · [Türkçe](tr/docker.md) · [Українська](uk/docker.md) · [简体中文](zh-CN/docker.md)

# Running in Docker

The `poulpilab/gitpoulpi` image holds the official release binaries (`gitpoulpi-tui`, `gitpoulpi`, `gitpoulpi-mcp`) on `debian:bookworm-slim`, with `git`, `git-lfs`, `openssh-client`, `less` and `tini`. It runs on `linux/amd64` and `linux/arm64`. It is on Docker Hub.

## Start the UI

```
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

- `-it` gives the UI a terminal. Without it the program has nothing to draw on.
- `-v "$PWD":/work` mounts the current folder as the working directory (`/work`). Open a repository from there, or mount several repositories under `/work`.
- `-v gitpoulpi-config:/config` is a named volume that keeps your settings and sessions between runs.
- `--user "$(id -u):$(id -g)"` runs as you, so the files GitPoulpi creates in your repositories keep your host owner instead of `root`. Every folder the image writes to is world-writable, so any user id works.

## Volumes and paths

| Path | What it holds |
|---|---|
| `/work` | The working directory. Bind-mount your repositories here. |
| `/config` | Settings (`config.toml`), sessions and, as sub-folders, data and cache (`GITPOULPI_CONFIG_DIR=/config`). Use a named volume or a bind mount to keep them. |
| `/data`, `/cache` | Declared as volumes too, for tools that follow the XDG variables (`XDG_DATA_HOME=/data`, `XDG_CACHE_HOME=/cache`). |
| `/home/gitpoulpi` | `HOME`. It is not a volume: a fresh one per container. |

Without a volume for `/config` every `--rm` run starts with default settings. To start from your own `config.toml`, mount it: `-v "$PWD/config.toml":/config/config.toml:ro` (read-only means Settings cannot save).

## Repositories owned by another user

Git refuses a repository whose owner differs from the user running it ("dubious ownership"). With `--user` as above the owner matches. When it does not (a bind mount from another account, a Docker Desktop share), trust every directory with:

```
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

The entry point then runs `git config --global --add safe.directory '*'` before it starts the UI. Only use it for folders you trust, since it lifts git's protection for the whole container.

## SSH

Forward your SSH agent instead of copying keys into the container.

Linux:

```
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

Docker Desktop (macOS, Windows) exposes the host agent at a fixed path inside its VM:

```
docker run -it --rm \
  -v /run/host-services/ssh-auth.sock:/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

With `--user` the socket must be readable by your user id, which it is when it belongs to you. The container has no `known_hosts`: the first connection to a host asks you to accept its key, or mount your own file with `-v ~/.ssh/known_hosts:/home/gitpoulpi/.ssh/known_hosts:ro`.

## Git identity and credentials

- **Identity and settings:** mount your Git configuration read-only: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`. Or set the commit identity in a GitPoulpi profile (Settings > Profiles), which lives in `/config`.
- **HTTPS credentials:** the image has no credential helper such as a keychain. Either mount a credential store (`-v ~/.git-credentials:/home/gitpoulpi/.git-credentials:ro`, with `credential.helper = store` in the mounted `.gitconfig`), or use an SSH remote with the agent forwarded.
- **Hosting tokens** (pull requests, issues): without a credential helper GitPoulpi keeps a token in memory for the session only. For a lasting setup pass it through the environment with `-e GITPOULPI_TOKEN_<ID>=...`, where `<ID>` is the integration's id uppercased with non-alphanumerics as `_` (see [Hosting and issue trackers](hosting-integration.md)). The integrations themselves are in `config.toml` under `/config`.

## Terminal, colors and mouse

Colors and mouse support come from your terminal emulator. Docker passes `TERM` on from `-t`; add `-e TERM -e COLORTERM` when colors or the true-color theme look wrong, for example `-e COLORTERM=truecolor`. Mouse clicks, drags and the wheel arrive through the terminal as usual. The commit graph is drawn with characters; pictures (kitty, iTerm2, sixel) are not forwarded through every terminal and multiplexer, see [Choosing a terminal](terminals.md). Resizing the terminal window resizes the UI: the container receives the size change.

## CLI and MCP

The entry point starts the UI. The other programs are reachable by overriding it:

```
# the scriptable CLI (add --json for scripts)
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# the MCP server, over standard input and output, for an agent that launches it
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

Each `docker run` is its own container with its own GitPoulpi background daemon, so changes made through the CLI do not show up live in a UI running in another container. To drive one UI from the CLI, run both in the same container: start the UI with `--name gp` and use `docker exec gp gitpoulpi ...`. For an agent, register `docker run -i --rm --entrypoint gitpoulpi-mcp ...` as the MCP server command, see [MCP and agents](mcp-and-agents.md); the MCP permission switches in the mounted `config.toml` apply as usual.

## Tags and updating

| Tag | Meaning |
|---|---|
| `X.Y.Z` | One exact release. |
| `X.Y` | The latest patch of that minor version (stable releases only). |
| `latest` | The latest stable release. |

A pre-release only gets its exact `X.Y.Z-...` tag.

An image is replaced, never updated in place. GitPoulpi inside a container knows that (its install record says "a Docker image"): the [update panel](updates.md) shows the version and the commands and replaces nothing. To update:

```
docker pull poulpilab/gitpoulpi
```

Then start a new container. Your settings stay in the `/config` volume. To go back, run an older tag: `docker pull poulpilab/gitpoulpi:<previous>`. To pin a version in scripts, use the exact tag instead of `latest`. An organization can mirror the image on its own registry, see [Enterprise distribution](enterprise-distribution.md#docker-registry-mirror).

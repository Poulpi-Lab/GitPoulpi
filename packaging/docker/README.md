# GitPoulpi

A Git client for the terminal that feels like a desktop app: a commit graph you can click, drag and right-click, a command palette that reaches every feature, pull and merge requests and issues from the main code hosts and trackers, and the same operations as a scriptable CLI and an MCP server for AI agents.

This image holds the official release binaries on `debian:bookworm-slim`, with `git`, `git-lfs`, `openssh-client` and `tini`. It runs on `linux/amd64` and `linux/arm64`.

This page is pushed from the repository on every release. Do not edit it on Docker Hub.

## Quick start

```sh
docker run -it --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD":/work \
  -v gitpoulpi-config:/config \
  poulpilab/gitpoulpi
```

The terminal UI opens in `/work`, which is the folder you mounted. Open a repository from there.

## Tags

| Tag | Meaning |
|---|---|
| `X.Y.Z` | One exact release. |
| `X.Y` | The latest patch of that minor version (stable releases only). |
| `latest` | The latest stable release. |

Pre-releases only get their exact `X.Y.Z-...` tag. An image is replaced, never updated in place: run `docker pull poulpilab/gitpoulpi` to update.

## Volumes and paths

| Path | Content |
|---|---|
| `/work` | The working directory. Bind-mount your repositories here. |
| `/config` | Settings, sessions, and (as sub-folders) data and cache. Use a named volume to keep them. |
| `/data`, `/cache` | Declared as volumes as well, for tools that follow the XDG variables. |

All of them are writable by any user, so `--user "$(id -u):$(id -g)"` makes the files you create keep your host owner.

## Repositories owned by another user

Git refuses a repository whose owner differs from the current user. When your bind mount shows that error, trust every directory with an environment variable:

```sh
docker run -it --rm -e GITPOULPI_SAFE_DIRECTORY='*' -v "$PWD":/work poulpilab/gitpoulpi
```

## SSH and credentials

Forward your SSH agent into the container:

```sh
docker run -it --rm \
  -v "$SSH_AUTH_SOCK":/ssh-agent -e SSH_AUTH_SOCK=/ssh-agent \
  -v "$PWD":/work poulpilab/gitpoulpi
```

On Docker Desktop, mount `/run/host-services/ssh-auth.sock` instead of `$SSH_AUTH_SOCK`. To reuse your Git identity, mount your configuration read-only: `-v ~/.gitconfig:/home/gitpoulpi/.gitconfig:ro`.

## CLI and MCP

The entry point starts the terminal UI. The other programs are reachable by overriding it:

```sh
# the scriptable CLI
docker run --rm --entrypoint gitpoulpi -v "$PWD":/work poulpilab/gitpoulpi status

# the MCP server, over standard input and output
docker run -i --rm --entrypoint gitpoulpi-mcp -v "$PWD":/work poulpilab/gitpoulpi
```

## Terminal tips

Use `-it` so the UI gets a terminal. Colors and mouse support depend on your terminal emulator, which sets `TERM` and `COLORTERM`; pass them with `-e TERM -e COLORTERM` if they are missing. Resizing the window resizes the UI.

## More

The full guide covers volumes, credentials and updates in detail: https://github.com/Poulpi-Lab/GitPoulpi/blob/main/docs/user/docker.md

Source and issues: https://github.com/Poulpi-Lab/GitPoulpi

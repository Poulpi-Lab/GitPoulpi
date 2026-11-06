# syntax=docker/dockerfile:1
#
# GitPoulpi image (Docker Hub poulpilab/gitpoulpi). It packs the release binaries, it compiles nothing. The
# build context is prepared by .github/workflows/release.yml (the `docker` job):
#
#   dist/amd64/{gitpoulpi,gitpoulpi-tui,gitpoulpi-mcp,gitpoulpi-launch,gitpoulpi-updater}   x86_64-unknown-linux-gnu
#   dist/arm64/...                                                                           aarch64-unknown-linux-gnu
#   entrypoint.sh                                                                            from packaging/docker/
#
#   docker buildx build -f packaging/docker/Dockerfile --build-arg VERSION=0.2.0 <context>
FROM debian:bookworm-slim

ARG TARGETARCH
ARG VERSION=0.0.0
ARG SOURCE_URL=https://github.com/Poulpi-Lab/GitPoulpi
ARG LICENSES=MIT
ARG DESCRIPTION="Git client for the terminal with a clickable commit graph, a CLI and an MCP server"

LABEL org.opencontainers.image.title="GitPoulpi" \
      org.opencontainers.image.description="${DESCRIPTION}" \
      org.opencontainers.image.source="${SOURCE_URL}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.licenses="${LICENSES}"

RUN apt-get update \
    && apt-get install -y --no-install-recommends git git-lfs openssh-client ca-certificates tini less \
    && rm -rf /var/lib/apt/lists/*

COPY --chmod=755 dist/${TARGETARCH}/gitpoulpi dist/${TARGETARCH}/gitpoulpi-tui dist/${TARGETARCH}/gitpoulpi-mcp dist/${TARGETARCH}/gitpoulpi-launch dist/${TARGETARCH}/gitpoulpi-updater /opt/gitpoulpi/
COPY --chmod=755 entrypoint.sh /usr/local/bin/entrypoint.sh

# The marker records the install channel: an image is replaced with `docker pull`, never updated in place.
RUN case "${TARGETARCH}" in \
        amd64) target=x86_64-unknown-linux-gnu ;; \
        arm64) target=aarch64-unknown-linux-gnu ;; \
        *) echo "unsupported architecture ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
    && printf 'version=%s\ntarget=%s\nchannel=docker\n' "${VERSION}" "${target}" > /opt/gitpoulpi/.gitpoulpi-dist \
    && for bin in gitpoulpi gitpoulpi-tui gitpoulpi-mcp gitpoulpi-launch; do ln -s "/opt/gitpoulpi/${bin}" "/usr/local/bin/${bin}"; done \
    && mkdir -p /config /data /cache /work /home/gitpoulpi \
    && chmod 1777 /config /data /cache /work /home/gitpoulpi

# GITPOULPI_CONFIG_DIR moves the config and, as sub-folders, the data and cache; the XDG variables cover
# anything that still asks the OS. Every directory is mode 1777 so any `--user uid:gid` can write.
ENV GITPOULPI_CONFIG_DIR=/config \
    XDG_DATA_HOME=/data \
    XDG_CACHE_HOME=/cache \
    HOME=/home/gitpoulpi

VOLUME /config /data /cache
WORKDIR /work
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

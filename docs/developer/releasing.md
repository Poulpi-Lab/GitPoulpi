# Releasing

How a release reaches every install channel. The workflow is `.github/workflows/release.yml`; the reasons behind the channels are in the install-channels design. The user side is in [Installation](../user/installation.md), [Updates](../user/updates.md) and [Enterprise distribution](../user/enterprise-distribution.md). Version bumping, the pre-tag checks and the sizes are in [release-process.md](release-process.md).

## Nothing publishes before a tag

The workflow runs only when a tag `releases/v<version>` is pushed (tags are prefixed by purpose, see `CONTRIBUTING.md`), or on a manual dispatch with an existing tag. Day to day it costs nothing. The tag's version must equal the workspace version in `Cargo.toml`: the first step of every build checks it, because the archive names come from `Cargo.toml` and the installers and the update check come from the tag.

A tag with a `-` in the version (`releases/v0.2.0-rc.1`) is published as a GitHub pre-release and reaches **no** channel. Only a stable tag, with the repository variable `PUBLISH_PACKAGES` set to `true`, feeds the channel jobs.

## Jobs

| Job | What it does |
|---|---|
| `preflight` | `cargo-deny` (`advisories bans sources licenses`, policy in `deny.toml`) and `xtask lock-age` against the previous `releases/v*` tag. CI is disabled, so this is where they run before a build. |
| `build` (matrix, after `preflight`) | One row per target: Linux x86_64 and arm64 (the arm64 row feeds the multi-arch image and the arm64 `.deb`), Windows x86_64, macOS arm64 and x86_64. `cargo run -p xtask -- dist` builds the archive and its `.sha256`. Windows also builds the Inno Setup installer and the MSI (WiX v5, `packaging/msi/gitpoulpi.wxs`); Linux also builds the `.deb` (`cargo deb --no-build`). Every file has a `.sha256`. |
| `publish` | One GitHub release with every archive, installer, `.deb`, `install.sh`, `latest.json` (generated from the `.sha256` files, with the current advisories), `security.json`, the SBOM, the `.sha256` files and a `.minisig` for every asset except the `.sha256` files (see "Signing and supply chain"). Notes are the tag's `## [x.y.z]` section of `CHANGELOG.md`, else GitHub's generated notes. Skipped by a dry run. |
| `chocolatey` | Fills the templates in `packaging/chocolatey/`, `choco pack`, `choco push`. |
| `winget` | `komac update GitPoulpi.GitPoulpi --version <v> --urls <setup> --submit`, from the fork `Poulpi-Lab/winget-pkgs`, as the App's bot. |
| `scoop`, `homebrew` | Fill the manifest or formula and commit it to `Poulpi-Lab/scoop-gitpoulpi` and `Poulpi-Lab/homebrew-gitpoulpi`, as the App's bot. |
| `aur` | In an Arch container fills `gitpoulpi-bin` and `gitpoulpi`, runs `makepkg --printsrcinfo` and pushes over SSH. |
| `apt` | Checks out `Poulpi-Lab/apt-gitpoulpi` (the `gh-pages` branch), adds the `.deb` files, runs `packaging/apt/build-repo.sh` (signs `Release` into `InRelease` and `Release.gpg`, keeps the last 5 versions) and commits. |
| `docker` | One multi-arch buildx build (`linux/amd64`, `linux/arm64`) from `packaging/docker/Dockerfile`, pushed to Docker Hub as `poulpilab/gitpoulpi` and, when enabled, to `ghcr.io/poulpi-lab/gitpoulpi`. Tags `<v>`, `<major.minor>` and `latest`; a pre-release would only get `<v>`. Then the Docker Hub overview. |
| `crates` | `scripts/publish-crates.sh`: every published crate in dependency order, opt-in (see "crates.io"). |

Every channel job starts with an `env` flag that says whether its secrets exist (`HAS`, or `HAS_DOCKERHUB` and `HAS_GHCR`) and each publishing step carries `if: env.HAS... == 'true'`. A missing secret skips the step and the job stays green with a notice, so a half-configured release never fails the whole run.

Every push to another repository and every pull request goes out as the Poulpi-Lab GitHub App's bot (`actions/create-github-app-token`), never as a personal account.

## Secrets and variables

The owner creates them in the repository settings. Signing keys and accounts belong to Poulpi-Lab, never to a person.

| Name | Kind | Used by |
|---|---|---|
| `POULPI_BOT_APP_ID` | variable | The App token for `winget`, `scoop`, `homebrew` and `apt`. |
| `POULPI_BOT_PRIVATE_KEY` | secret | The same App token. The App needs contents and pull-requests write on `scoop-gitpoulpi`, `homebrew-gitpoulpi`, `apt` and the `winget-pkgs` fork. |
| `CHOCO_API_KEY` | secret | `chocolatey`: the project's Chocolatey account. |
| `AUR_SSH_PRIVATE_KEY` | secret | `aur`: the project's AUR account. |
| `APT_GPG_PRIVATE_KEY`, `APT_GPG_PASSPHRASE` | secrets | `apt`: the repository signing key and its passphrase (none when the key has none). |
| `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN` | secrets | `docker`: the Docker Hub organisation or its service account, and a read-write access token limited to `poulpilab/gitpoulpi`. Both are needed to push and to update the overview. |
| `CARGO_REGISTRY_TOKEN` | secret | `crates` (optional): a crates.io API token, needed for the first publish and as the fallback when Trusted Publishing is off. |
| `PUBLISH_PACKAGES` | variable | `true` enables every channel job. Anything else, and the release still publishes on GitHub. |
| `PUBLISH_GHCR` | variable | `true` also pushes the image to GHCR with `GITHUB_TOKEN` (the job has `packages: write`). No extra secret. |
| `PUBLISH_CRATES` | variable | `true` enables the `crates` job (with `PUBLISH_PACKAGES`, a stable tag only). |
| `CRATES_TRUSTED_PUBLISHING` | variable | `true` makes `crates` authenticate with crates.io Trusted Publishing (OIDC) instead of `CARGO_REGISTRY_TOKEN`. Set it only once every crate has its trusted publisher configured. |
| `MINISIGN_SECRET_KEY`, `MINISIGN_PASSWORD` | secrets of the `release` environment | `publish` and `security-advisory.yml`: the release signing key box and its password. |
| `ATTESTATIONS` | variable | `true` enables the build provenance attestation step of `publish`. |
| `DTRACK_URL`, `DTRACK_API_KEY` | secrets (optional) | `publish`: uploads the SBOM to a Dependency-Track server. Skipped with a notice when absent. |

## The first winget submission is by hand

`komac update` needs the package to exist in `microsoft/winget-pkgs`. Submit the first version yourself (the three manifests of `packaging/winget/`, filled in, as a pull request from the fork), wait for it to be merged, and from then on the `winget` job opens one pull request per release. An App installation token may be refused for cross-repository pull requests to `microsoft/winget-pkgs`; the fallback is a token of a dedicated Poulpi-Lab machine account, an owner decision.

## The Docker Hub overview comes from the repository

`packaging/docker/README.md` is the Docker Hub page. The `docker` job pushes it on every release (`peter-evans/dockerhub-description`), replacing whatever is there, so it is never edited on Docker Hub. The test `xtask/tests/installers.rs` keeps it under Docker Hub's 25,000-byte limit, free of relative links (they break there), and pins the short description (at most 100 characters) and the image name `DOCKER_IMAGE` in `crates/gitpoulpi-config/src/release.rs`. A namespace change is that constant, the workflow and the README.

## crates.io

`cargo install --locked gitpoulpi-cli` installs the `gitpoulpi` program (it is the whole product: the terminal UI when run without a subcommand, the CLI, and `gitpoulpi mcp`). That crate is named after its role, not `gitpoulpi`, on purpose: the package name is what `cargo binstall` and the install-channel metadata already use, a second package carrying the same `gitpoulpi` binary would collide with it, and a `gitpoulpi` crate that only forwards to it would need the whole CLI moved into a library for no gain. `gitpoulpi-tui` and `gitpoulpi-mcp` are standalone binaries of the same code. The default features of those crates are the full set the release archives ship (`xtask dist` builds with the defaults too).

**What is published.** Every crate except `xtask`, `gitpoulpi-testkit` and `gitpoulpi-updater` (`publish = false`; the updater is the install helper of the archive, installer and package channels, useless from crates.io, and `xtask` is a name somebody else owns). The testkit and `xtask` appear in `[dev-dependencies]` of published crates without a version, which cargo drops from the published manifest. The order, which `scripts/publish-crates.sh` follows and `xtask/tests/crates_io.rs` checks against `cargo metadata`:

```
gitpoulpi-core, -i18n, -logo-build, -pty, -secrets, -cli-backend, -config, -git2-backend,
-hosting, -ipc, -log, -session, -update, -ai, -daemon, -tui, -commands, -mcp, -cli
```

**Packaging rules.** A package cannot read a file outside its own directory, so nothing in `src` or `build.rs` may `include_str!("../../..")`. Three files the repository keeps at its root have a copy inside the crate that embeds them, and `xtask/tests/crates_io.rs` fails when a copy differs: `CHANGELOG.md` (`gitpoulpi-tui`, copy it after editing the root file), `THIRD-PARTY-LICENSES.txt` (`gitpoulpi-core`, `cargo run -p xtask -- licenses` writes both) and `packaging/minisign.pub` (`gitpoulpi-update`). The logo table and the build-script code shared by the three binaries live in `gitpoulpi-logo-build`, a build-dependency. `[workspace.package]` carries the metadata crates.io asks for and an `exclude` that keeps tests, snapshots and golden files out of the packages (`gitpoulpi-daemon` keeps `tests/support/credential_stub.rs`, a binary its manifest names). Every path dependency of a published crate also carries a `version` in `[workspace.dependencies]`: **bumping the workspace version means bumping those too** (the test says which).

**Check before a release.** `cargo publish --workspace --dry-run --exclude xtask --exclude gitpoulpi-testkit --exclude gitpoulpi-updater` packages and builds every crate in order, against the packaged sources (needs network for the index, publishes nothing). `cargo package -p gitpoulpi-cli --list` shows what a crate holds; each `.crate` must stay far below crates.io's 10 MB limit.

**First publish (by token).** Trusted Publishing can only be configured on a crate that already exists, so the first release of the workspace uses an API token:

1. The owner creates the crates.io account (sign in with GitHub) and verifies its email.
2. Create an API token with the scopes `publish-new` and `publish-update` and, if possible, the crate pattern `gitpoulpi-*`. Store it as the secret `CARGO_REGISTRY_TOKEN` of the `release` environment.
3. Set the variables `PUBLISH_PACKAGES` and `PUBLISH_CRATES` to `true` and push a stable tag. The `crates` job runs `scripts/publish-crates.sh`: crates one by one, each retried (the index needs a moment to show a dependency just published), and a version already on crates.io is skipped, so a failed run is restarted with "Re-run failed jobs". Pre-releases (a `-` in the tag) are never published.

**Then Trusted Publishing**, so no long-lived token stays in the repository: on crates.io, for each crate, Settings, Trusted Publishing, add a GitHub publisher with repository `Poulpi-Lab/GitPoulpi`, workflow `release.yml` and environment `release`. When all nineteen are configured, set the variable `CRATES_TRUSTED_PUBLISHING` to `true` and delete the `CARGO_REGISTRY_TOKEN` secret. The job then gets a short-lived token from `rust-lang/crates-io-auth-action` (pinned by SHA like every action; the job has `id-token: write`). A new crate added later again needs one token publish before its publisher can be configured.

**Ownership.** Crates published by the owner's account belong to that account. To avoid a single person owning them, add a GitHub team as owner: `cargo owner --add github:Poulpi-Lab:<team> <crate>` for each crate (the team must be visible to crates.io: the organisation grants the crates.io application access). Do it right after the first publish.

**Yanking.** A bad release is yanked, never deleted: `cargo yank --version <v> <crate>` for each crate (`--undo` reverts). Yanking keeps existing lockfiles building and stops new resolutions. Publish a fixed patch version for all crates; versions move together.

## Dry run

A dry run builds every archive and installer and publishes nothing:

```
git tag dryrun/v0.1.0 && git push origin dryrun/v0.1.0
gh workflow run release.yml -f tag=dryrun/v0.1.0 -f dry_run=true
```

Download the run's artifacts and check them: the archives' `.sha256`, the setup and MSI installing in a clean Windows machine, the `.deb` with `dpkg -c`. The tag's version must still match `Cargo.toml`. A dry run skips `publish`, so the channel jobs (which need it) do not run either.

## Before the first public tag

Check by hand (the installers are not exercised by CI):

- Windows: a per-user and an all-users install, an update through the panel (the administrator prompt, other users' processes closed), `choco upgrade` and `winget upgrade` re-running the installer and keeping the channel in the marker, and a revert. These steps are listed in [where-we-are.md](where-we-are.md).
- A `gitpoulpi --version` from the Docker image, `docker run -it` drawing the UI, and the SSH agent forwarding.
- The apt repository: add the source on a clean Debian, `apt install`, then `apt install --only-upgrade` with a second version.

## Signing and supply chain

### The release signing key

Every release asset is signed with [minisign](https://jedisct1.github.io/minisign/), so an update, a mirror or a manual download can be checked against a key the project controls, independently of GitHub. The signature of each file carries the trusted comment `file:<name> version:<v>`: a valid signature cannot be moved onto another file or onto an older release.

The key is generated once, offline, by the owner, as a project key and never a personal one:

```
minisign -G -p packaging/minisign.pub -s gitpoulpi-release.key
```

- Use a strong password. Keep the key file and the password in the organization's password vault.
- Create a GitHub environment named `release` and store `MINISIGN_SECRET_KEY` (the whole text of `gitpoulpi-release.key`) and `MINISIGN_PASSWORD` there. Only the `publish` job and `security-advisory.yml` use that environment.
- Commit only `packaging/minisign.pub`. The update client compiles it in (`gitpoulpi_update::signing::release_keys()`, a list, so a rotation ships the next key in a release before the old one retires: add the new public key file to `KEY_FILES` there). A release is only installable by a client that has its key, and only releases signed by this workflow can be installed through the updater.

`cargo run -p xtask -- sign --dir dist --version <v>` is what the workflow runs. It signs every file of the directory except `*.sha256` and `*.minisig` (archives, installers, `.deb`, `install.sh`, `latest.json`, `security.json`, the SBOM) and reads the key from the two environment variables; it never prompts. To check a signature by hand: `minisign -Vm <file> -p packaging/minisign.pub`, then read the trusted comment it prints.

### Rotation

The client accepts a list of compiled-in keys (the current one and the next). To rotate: generate the new key, ship a release that lists both public keys and is still signed with the old one, switch the `release` environment secrets to the new key, and drop the old public key from the list in a later release. Keep a list of keys with their key ids and dates in the password vault.

### Attestations

`actions/attest-build-provenance` signs a build provenance statement for `dist/*`. On a private repository this needs GitHub Enterprise Cloud, so the step runs only when the repository variable `ATTESTATIONS` is `true`. Users verify with `gh attestation verify <file> --repo <owner>/gitpoulpi`.

### SBOM

`publish` runs `cargo cyclonedx --format json` for the binary crates, merges the results into `gitpoulpi-<v>-sbom.cdx.json`, signs it and attaches it. When the secrets `DTRACK_URL` and `DTRACK_API_KEY` exist, the SBOM is also posted to `<DTRACK_URL>/api/v1/bom` (`autoCreate`, project `GitPoulpi`, the release version). Without them the step prints a notice and skips.

### Dependencies

- `.github/dependabot.yml` proposes `cargo` and `github-actions` updates weekly with `cooldown: default-days: 3`. The cooldown does not delay security updates. `xtask/tests/workflow_pins.rs` fails if the cooldown drops below 2 days.
- Every third-party `uses:` is pinned by its full commit SHA with a `# vX.Y.Z` comment; Dependabot keeps both current. The same test checks every `uses:`.
- `deny.toml` holds the `cargo-deny` policy: advisories (yanked releases denied, every ignore needs a reason), bans (no `openssl-sys`, no `native-tls`, wildcard versions denied) and sources (crates.io only). It runs in `ci.yml` and in the release `preflight`. Run `cargo deny check advisories bans sources licenses` locally when it is installed.
- **Advisories between releases.** `.github/workflows/advisories.yml` (not disabled like `ci.yml`) runs daily, on demand and on every push to `main` that touches `Cargo.lock` or `deny.toml`: `cargo deny check advisories` plus `cargo run -p xtask -- advisories`, which asks OSV (RustSec and GitHub advisories) about every crates.io crate of `Cargo.lock` and drops the ids or aliases that `deny.toml` ignores. A finding opens or updates one issue, "Dependency security advisories", and posts a comment whenever the findings change (an edited body notifies nobody, a comment notifies watchers); a clean scan closes it. Locally: `cargo run -p xtask -- advisories` (`--json`, `--markdown` for an issue body, `--no-paths` to skip the `cargo tree -i` path). It exits 1 on any open advisory. Fix by upgrading the crate, or add an `ignore` entry to `deny.toml` with a reason that says why the flagged code is not reached and what upgrade is tracked.
- **Dependabot alerts** on a private organization repository are not on by default: an administrator enables "Dependency graph", "Dependabot alerts" and "Dependabot security updates" in Settings, Code security. None of them needs GitHub Advanced Security.
- **Minimum age.** `cargo run -p xtask -- lock-age --base <ref> --min-days 2` fails when `Cargo.lock` adds or changes a crates.io version published less than two days before. It runs in `ci.yml` (pull requests and pushes that change `Cargo.lock`) and in the release `preflight` against the previous release tag. For an urgent fix, add a trailer to a commit of the range: `Lock-Age-Override: <name>@<version> <reason>` (the reason is mandatory).

### Security notices

`security/advisories.json` is the source of truth: a list of `{id, affected, fixed, severity, url, summary}` with ids `GPSA-YYYY-NNN`, a semver range in `affected`, and an https `url`. Add one with `cargo run -p xtask -- advisory add --affected ">=1.2.0, <1.4.3" --fixed 1.4.3 --severity high --url https://... --summary "..."`, and validate with `cargo run -p xtask -- advisory check`. Merge it by pull request.

The workflow `.github/workflows/security-advisory.yml` (manual dispatch, and every push to `main` that touches the file) validates the file, builds `security.json`, adds the advisories to the latest release's `latest.json`, signs both files and replaces them on the release with `gh release upload --clobber`, as the Poulpi-Lab GitHub App's bot (`POULPI_BOT_APP_ID`, `POULPI_BOT_PRIVATE_KEY`). `release.yml` copies the current file into every new release. Reporting of vulnerabilities is described in `SECURITY.md`.

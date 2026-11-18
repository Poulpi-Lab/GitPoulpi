//! The installers (`packaging/`) and the release workflow name the same
//! files. Inno Setup and the workflow cannot run here, so these tests pin what
//! can drift: file names, the version placeholder, and the CLI commands the
//! scripts call.

use std::path::{Path, PathBuf};

use xtask::dist::{archive_name, exe_name, installer_name};

const WINDOWS: &str = "x86_64-pc-windows-msvc";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[test]
fn the_inno_script_installs_the_archives_files_under_the_expected_name() {
    let iss = read("packaging/windows/gitpoulpi.iss");
    for bin in gitpoulpi_config::release::BINARIES {
        let exe = exe_name(bin, WINDOWS);
        assert!(
            iss.contains(&format!("{{#SourceDir}}\\{exe}")),
            "{exe} is not installed by the script"
        );
    }
    assert!(iss.contains(&format!(
        "{{#SourceDir}}\\{}",
        gitpoulpi_config::release::MARKER_FILE
    )));
    // The output name is the shared installer name with the version as a
    // placeholder: `installer_name` minus the extension.
    let expected = installer_name("{#AppVersion}", WINDOWS);
    let base = expected.strip_suffix(".exe").unwrap();
    assert!(
        iss.contains(&format!("OutputBaseFilename={base}")),
        "OutputBaseFilename must be {base}"
    );
    assert!(iss.contains("AppVersion={#AppVersion}"));
    assert!(iss.contains("#ifndef AppVersion"));
}

#[test]
fn the_inno_script_installs_per_user_and_shares_the_cli_with_settings() {
    let iss = read("packaging/windows/gitpoulpi.iss");
    assert!(iss.contains("PrivilegesRequired=lowest"));
    // `commandline` lets winget (/CURRENTUSER, /ALLUSERS) pick the scope, `dialog` lets the user.
    assert!(iss.contains("PrivilegesRequiredOverridesAllowed=commandline dialog"));
    assert!(iss.contains("UsePreviousPrivileges=yes"));
    for text in [
        "PrivilegesRequiredOverrideText1=",
        "PrivilegesRequiredOverrideText2=",
        "Only for me (no administrator rights; updates in place)",
        "For all users, in Program Files (updates ask for administrator approval)",
    ] {
        assert!(iss.contains(text), "override text `{text}`");
    }
    assert!(iss.contains("DefaultDirName={autopf}\\GitPoulpi"));
    for task in ["startmenu", "desktopicon", "addtopath"] {
        assert!(iss.contains(&format!("Name: \"{task}\"")), "task {task}");
    }
    for command in [
        "desktop shortcut install --where startmenu",
        "desktop shortcut install --where desktop",
        "path add",
        "alias create {code:GetAliasName}",
        "uninstall --keep-files",
    ] {
        assert!(iss.contains(command), "the script never runs `{command}`");
    }
    assert!(iss.contains("{param:ALIAS|gp}"), "default alias");
}

/// The channel the package or installer records, spelled as the marker's `channel=` value.
fn assert_channel(channel: &str) {
    assert!(
        gitpoulpi_config::release::InstallChannel::parse(channel).is_some(),
        "unknown channel {channel}"
    );
}

/// Every program a package ships. `gitpoulpi-updater` is listed ahead of `BINARIES` gaining it
/// (slice D of the install-channels design), so the templates are ready before the first release
/// that has it.
fn shipped_binaries() -> Vec<&'static str> {
    let mut all: Vec<&str> = gitpoulpi_config::release::BINARIES.to_vec();
    if !all.contains(&"gitpoulpi-updater") {
        all.push("gitpoulpi-updater");
    }
    all
}

use gitpoulpi_config::release::DOCKER_IMAGE;

#[test]
fn the_inno_script_writes_the_channel_marker_after_the_copy() {
    let iss = read("packaging/windows/gitpoulpi.iss");
    // The channel comes from /CHANNEL=, else from the privileges the user chose.
    assert!(iss.contains("{param:CHANNEL|}"));
    assert!(iss.contains("{param:FEED|}"));
    assert!(iss.contains("CurStep = ssPostInstall"));
    assert!(iss.contains("IsAdminInstallMode"));
    for channel in ["inno-user", "inno-machine"] {
        assert_channel(channel);
        assert!(iss.contains(&format!("'{channel}'")), "{channel}");
    }
    assert!(iss.contains("'channel=' + GetChannel"));
    assert!(iss.contains("'feed=' + Feed"));
    assert!(iss.contains(gitpoulpi_config::release::MARKER_FILE));
    // The old switch is gone: the channel, not a missing marker, decides what the app may do.
    for file in [
        "packaging/windows/gitpoulpi.iss",
        "packaging/chocolatey/tools/chocolateyinstall.ps1",
        "packaging/winget/GitPoulpi.GitPoulpi.installer.yaml",
    ] {
        assert!(!read(file).contains("NOSELFUPDATE"), "{file}");
    }
    assert!(iss.contains("gitpoulpi-updater.exe"));
}

#[test]
fn the_msi_ships_every_program_and_the_policy_properties() {
    let wxs = read("packaging/msi/gitpoulpi.wxs");
    for bin in shipped_binaries() {
        assert!(wxs.contains(&exe_name(bin, WINDOWS)), "{bin}");
    }
    assert!(wxs.contains(gitpoulpi_config::release::MARKER_FILE));
    assert!(wxs.contains("MajorUpgrade"));
    assert!(wxs.contains("UpgradeCode=\""));
    assert!(wxs.contains("HKLM"));
    for property in [
        "INSTALLFOLDER",
        "ADDTOPATH",
        "UPDATES_MODE",
        "UPDATES_PIN",
        "UPDATES_CHECK",
        "UPDATES_CHECK_URL",
        "UPDATES_MESSAGE",
    ] {
        assert!(wxs.contains(property), "{property}");
    }
    assert!(wxs.contains("SOFTWARE\\Policies\\GitPoulpi\\Updates"));
    assert_channel("msi");
}

#[test]
fn the_release_workflow_builds_and_uploads_what_the_installers_are_named() {
    let yml = read(".github/workflows/release.yml");
    assert!(yml.contains("packaging/windows/gitpoulpi.iss"));
    assert!(yml.contains("choco install innosetup"));
    assert!(yml.contains("/DAppVersion=$version"));
    // The workflow builds the setup name from the same stem as the zip.
    let stem = "gitpoulpi-$version-${{ matrix.target }}";
    assert!(yml.contains(&format!("$stem = \"{stem}\"")));
    assert!(yml.contains("\"$stem-setup.exe\""));
    assert_eq!(
        installer_name("1", WINDOWS),
        format!(
            "{}-setup.exe",
            archive_name("1", WINDOWS).trim_end_matches(".zip")
        )
    );
    // The setup and its checksum are written into target/dist, which is what
    // the build job uploads; install.sh joins them in the publish job.
    assert!(yml.contains("/DOutputDir=$PWD\\target\\dist"));
    assert!(yml.contains("path: target/dist/*"));
    assert!(yml.contains("cp packaging/install.sh dist/install.sh"));
    assert!(yml.contains("files: dist/*"));
}

#[test]
fn install_sh_calls_the_cli_commands_that_exist_and_knows_the_marker() {
    let sh = read("packaging/install.sh");
    for command in [
        "alias create",
        "path add",
        "desktop shortcut install",
        "uninstall",
    ] {
        assert!(sh.contains(command), "install.sh never runs `{command}`");
    }
    assert!(sh.contains(gitpoulpi_config::release::MARKER_FILE));
    for bin in gitpoulpi_config::release::BINARIES {
        assert!(sh.contains(bin), "{bin} is not mentioned");
    }
    // The archive name it downloads is the one `xtask dist` writes.
    assert!(sh.contains("gitpoulpi-$VERSION-$target.tar.gz"));
    assert_eq!(
        archive_name("1.2.3", "x86_64-unknown-linux-gnu"),
        "gitpoulpi-1.2.3-x86_64-unknown-linux-gnu.tar.gz"
    );
    for flag in [
        "--prefix",
        "--archive",
        "--alias",
        "--no-alias",
        "--no-path",
        "--desktop",
        "--yes",
        "--uninstall",
    ] {
        assert!(sh.contains(flag), "{flag}");
    }
}

#[test]
fn every_installer_ships_the_third_party_notices() {
    let notices = xtask::licenses::OUTPUT;
    assert!(read("packaging/install.sh").contains(notices));
    assert!(read("packaging/windows/gitpoulpi.iss").contains(notices));
    assert!(read("xtask/src/dist.rs").contains("licenses::OUTPUT"));
}

/// The package-manager templates (`packaging/{aur,scoop,winget,chocolatey,homebrew}`, plan
/// the release-and-packaging design) download release assets by name: a rename in
/// `gitpoulpi_config::release` must show up here, not in a broken package after a release.
#[test]
fn the_package_templates_name_the_release_assets_and_ship_every_binary() {
    const LINUX: &str = "x86_64-unknown-linux-gnu";
    const MAC_ARM: &str = "aarch64-apple-darwin";
    const MAC_INTEL: &str = "x86_64-apple-darwin";
    const DOWNLOAD: &str = "https://github.com/Poulpi-Lab/GitPoulpi/releases/download/releases/v";
    let marker = gitpoulpi_config::release::MARKER_FILE;
    let binaries = gitpoulpi_config::release::BINARIES;

    // (template, how it spells the version, the archives it downloads)
    let cases: [(&str, &str, &[&str]); 7] = [
        (
            "packaging/aur/gitpoulpi-bin/PKGBUILD",
            "${pkgver}",
            &[LINUX],
        ),
        ("packaging/scoop/gitpoulpi.json", "@VERSION@", &[WINDOWS]),
        ("packaging/scoop/gitpoulpi.json", "$version", &[WINDOWS]),
        (
            "packaging/homebrew/gitpoulpi.rb",
            "#{version}",
            &[MAC_ARM, MAC_INTEL, LINUX],
        ),
        (
            "packaging/winget/GitPoulpi.GitPoulpi.installer.yaml",
            "@VERSION@",
            &[],
        ),
        (
            "packaging/chocolatey/tools/chocolateyinstall.ps1",
            "@VERSION@",
            &[],
        ),
        ("packaging/aur/gitpoulpi/PKGBUILD", "${pkgver}", &[]),
    ];
    for (file, v, targets) in cases {
        let text = read(file);
        assert!(text.contains("TEMPLATE, NOT PUBLISHED"), "{file}");
        for target in targets {
            let asset = archive_name(v, target);
            assert!(
                text.contains(&format!("{DOWNLOAD}{v}/{asset}")),
                "{file}: {asset}"
            );
        }
    }
    let setup = installer_name("@VERSION@", WINDOWS);
    for file in [
        "packaging/winget/GitPoulpi.GitPoulpi.installer.yaml",
        "packaging/chocolatey/tools/chocolateyinstall.ps1",
    ] {
        assert!(
            read(file).contains(&format!("{DOWNLOAD}@VERSION@/{setup}")),
            "{file}"
        );
    }

    // Every package installs the marker with its own channel, and ships every program.
    // (file, the channel it records, a fragment that proves it)
    let recorded: [(&str, &str, &str); 5] = [
        ("packaging/aur/gitpoulpi-bin/PKGBUILD", "aur", "channel=aur"),
        ("packaging/aur/gitpoulpi/PKGBUILD", "aur", "channel=aur"),
        (
            "packaging/homebrew/gitpoulpi.rb",
            "homebrew",
            "channel=homebrew",
        ),
        ("packaging/scoop/gitpoulpi.json", "scoop", "channel=scoop"),
        ("packaging/docker/Dockerfile", "docker", "channel=docker"),
    ];
    for (file, channel, fragment) in recorded {
        assert_channel(channel);
        let text = read(file);
        let code: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(code.contains(fragment), "{file} never writes {fragment}");
        assert!(code.contains(marker), "{file} never installs {marker}");
    }
    for file in [
        "packaging/aur/gitpoulpi-bin/PKGBUILD",
        "packaging/aur/gitpoulpi/PKGBUILD",
        "packaging/homebrew/gitpoulpi.rb",
        "packaging/docker/Dockerfile",
    ] {
        let text = read(file);
        for bin in shipped_binaries() {
            assert!(text.contains(bin), "{file}: {bin}");
        }
    }
    // Scoop unpacks the whole zip (the updater with it) and rewrites the marker after install.
    let scoop = read("packaging/scoop/gitpoulpi.json");
    for bin in binaries {
        assert!(scoop.contains(&exe_name(bin, WINDOWS)), "scoop: {bin}");
    }
    assert!(
        !scoop.contains("Remove-Item"),
        "scoop must not delete the marker"
    );
    // Chocolatey and winget run the Inno installer with /CHANNEL=, which the script writes into the marker.
    for (file, channel) in [
        (
            "packaging/winget/GitPoulpi.GitPoulpi.installer.yaml",
            "winget",
        ),
        ("packaging/chocolatey/tools/chocolateyinstall.ps1", "choco"),
    ] {
        assert_channel(channel);
        let text = read(file);
        assert!(text.contains("/CHANNEL="), "{file}");
        assert!(text.contains(channel), "{file}: {channel}");
    }
    assert!(read("packaging/winget/GitPoulpi.GitPoulpi.installer.yaml").contains("/CHANNEL=winget"));
    let choco = read("packaging/chocolatey/tools/chocolateyinstall.ps1");
    assert!(choco.contains("$channel = 'choco'"));
    assert!(
        choco.contains("Channel") && choco.contains("Feed"),
        "choco-internal parameters"
    );
    assert_channel("choco-internal");
    // The CLI package and install.sh.
    assert_channel("archive");
    assert!(read("packaging/install.sh").contains("channel=archive"));
    assert_channel("apt");
    let cli = read("crates/gitpoulpi-cli/Cargo.toml");
    for bin in shipped_binaries() {
        assert!(cli.contains(&format!("deb-stage/{bin}\"")), "deb: {bin}");
    }
    assert!(cli.contains("deb-stage/.gitpoulpi-dist"));
    assert!(cli.contains("[package.metadata.binstall]"));
    assert!(cli.contains("bin-dir = \"gitpoulpi-{ version }-{ target }/{ bin }{ binary-ext }\""));
    // winget and Chocolatey run the Inno installer, whose AppId is the
    // uninstall key winget calls ProductCode.
    let iss = read("packaging/windows/gitpoulpi.iss");
    let winget = read("packaging/winget/GitPoulpi.GitPoulpi.installer.yaml");
    let app_id = "8C1D3E5A-6B7F-4A29-9E0C-47A1B2C3D4E5";
    assert!(iss.contains(&format!("AppId={{{{{app_id}}}")));
    assert!(winget.contains(&format!("ProductCode: \"{{{app_id}}}_is1\"")));
}

/// The text of one job of `release.yml`: from `  <name>:` to the next job.
fn job(yml: &str, name: &str) -> String {
    let start = yml
        .find(&format!("\n  {name}:\n"))
        .unwrap_or_else(|| panic!("release.yml has no job {name}"))
        + 1;
    let mut end = yml.len();
    let mut offset = start;
    for (n, line) in yml[start..].split_inclusive('\n').enumerate() {
        // The next job (two spaces, a name and a colon) or a comment block between jobs.
        let next_job = line.starts_with("  ")
            && !line.starts_with("   ")
            && !line.starts_with("  #")
            && line.trim_end().ends_with(':');
        if n > 0 && (next_job || line.starts_with("  # ")) {
            end = offset;
            break;
        }
        offset += line.len();
    }
    yml[start..end].to_string()
}

/// The steps of a job: the text from each `      - ` list entry to the next.
fn steps(job_text: &str) -> Vec<String> {
    job_text
        .split("\n      - ")
        .skip(1)
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn the_release_workflow_names_every_asset_and_runs_only_on_a_version_tag() {
    let yml = read(".github/workflows/release.yml");
    // Nothing runs without a pushed releases/v* tag (or a manual dispatch naming a tag).
    assert!(yml.contains("tags: ['releases/v*']"));
    assert!(!yml.contains("branches:"));
    assert!(!yml.contains("pull_request"));
    assert!(!yml.contains("schedule:"));
    // The matrix builds both Linux CPUs.
    assert!(yml.contains("target: aarch64-unknown-linux-gnu"));
    assert!(yml.contains("ubuntu-22.04-arm"));
    // The installers and packages of the build job, each with a checksum.
    assert!(yml.contains("packaging/msi/gitpoulpi.wxs"));
    assert!(yml.contains("wix build"));
    assert!(yml.contains("$stem.msi"));
    assert!(yml.contains("$msi.sha256"));
    assert!(yml.contains("cargo deb -p gitpoulpi-cli --no-build"));
    assert!(yml.contains("$deb.sha256"));
    assert!(yml.contains("--channel apt"));
    assert!(yml.contains("--channel msi"));
    assert!(yml.contains("package-marker"));
    // The publish job attaches latest.json next to install.sh, and the release uploads dist/*.
    assert!(yml.contains("latest.json"));
    assert!(yml.contains("files: dist/*"));
}

#[test]
fn every_channel_job_is_gated_and_skips_a_missing_secret() {
    let yml = read(".github/workflows/release.yml");
    for name in [
        "chocolatey",
        "winget",
        "scoop",
        "homebrew",
        "aur",
        "apt",
        "docker",
        "crates",
    ] {
        let text = job(&yml, name);
        assert!(text.contains("needs: publish"), "{name}");
        assert!(
            text.contains("vars.PUBLISH_PACKAGES == 'true'"),
            "{name}: no PUBLISH_PACKAGES gate"
        );
        assert!(
            text.contains("needs.publish.outputs.prerelease == 'false'"),
            "{name}: no pre-release gate"
        );
        assert!(text.contains("HAS"), "{name}: no secret flag");
        assert!(
            text.contains("::notice::"),
            "{name}: no notice for a missing secret"
        );
        // The docker job builds without a login (the gate is on its registry steps, tested below); in the
        // others every step is behind the flag, except the notice that says it is not set.
        if name == "docker" {
            continue;
        }
        for step in steps(&text) {
            let gated = step.contains("if: env.HAS");
            assert!(gated, "{name}: ungated step\n{step}");
        }
    }
    // Pushes to other repositories go out as the App bot, never as a personal account.
    assert!(yml.contains("actions/create-github-app-token@"));
    assert!(yml.contains("vars.POULPI_BOT_APP_ID"));
    assert!(yml.contains("secrets.POULPI_BOT_PRIVATE_KEY"));
    for forbidden in ["PERSONAL", "GH_PAT", "secrets.PAT"] {
        assert!(!yml.contains(forbidden), "{forbidden}");
    }
    // Third-party actions are pinned by commit SHA (workflow_pins.rs checks every `uses:` of every workflow).
    for line in yml.lines().filter(|l| l.contains("uses: ")) {
        let reference = line
            .split("uses: ")
            .nth(1)
            .unwrap()
            .split('#')
            .next()
            .unwrap()
            .trim();
        let version = reference.split_once('@').map_or("", |(_, v)| v);
        assert!(version.len() == 40, "unpinned action: {line}");
    }
}

#[test]
fn the_docker_image_goes_to_docker_hub_and_optionally_ghcr() {
    let yml = read(".github/workflows/release.yml");
    let docker = job(&yml, "docker");
    // Docker Hub is primary: the image name is in the tags, the overview and the README.
    assert!(docker.contains(DOCKER_IMAGE), "tags");
    assert!(
        docker.contains(&format!("repository: {DOCKER_IMAGE}")),
        "overview"
    );
    assert!(docker.contains("readme-filepath: packaging/docker/README.md"));
    assert!(read("packaging/docker/README.md").contains(DOCKER_IMAGE));
    // GHCR is the optional second registry, behind a variable and the job's GITHUB_TOKEN.
    assert!(docker.contains("ghcr.io/poulpi-lab/gitpoulpi"));
    assert!(docker.contains("vars.PUBLISH_GHCR == 'true'"));
    assert!(docker.contains("packages: write"));
    assert!(
        !docker.contains("PUBLISH_DOCKER"),
        "PUBLISH_DOCKER is replaced by the secrets"
    );
    assert!(docker.contains("secrets.DOCKERHUB_USERNAME != '' && secrets.DOCKERHUB_TOKEN != ''"));
    assert!(docker.contains("linux/amd64,linux/arm64"));
    // latest and major.minor only on a stable tag.
    assert!(docker.contains("type=semver,pattern={{major}}.{{minor}}"));
    assert!(docker.contains("type=raw,value=latest,enable=${{ !contains(env.VERSION, '-') }}"));
    // Every login, push or overview step has a secret or variable gate.
    let mut seen = 0;
    for step in steps(&docker) {
        let touches_a_registry = step.contains("docker/login-action")
            || step.contains("docker/build-push-action")
            || step.contains("dockerhub-description");
        if touches_a_registry {
            seen += 1;
            assert!(
                step.contains("env.HAS_DOCKERHUB") || step.contains("env.HAS_GHCR"),
                "ungated registry step\n{step}"
            );
        }
    }
    assert_eq!(
        seen, 4,
        "Docker Hub login, GHCR login, build and push, overview"
    );
}

#[test]
fn the_dockerfile_copies_the_binaries_writes_the_channel_and_declares_the_volumes() {
    let docker = read("packaging/docker/Dockerfile");
    for bin in shipped_binaries() {
        assert!(
            docker.contains(&format!("dist/${{TARGETARCH}}/{bin}")),
            "{bin}"
        );
    }
    assert!(docker.contains("channel=docker"));
    assert_channel("docker");
    assert!(docker.contains("VOLUME /config /data /cache"));
    assert!(docker.contains("GITPOULPI_CONFIG_DIR=/config"));
    assert!(docker.contains("ENTRYPOINT"));
    assert!(docker.contains("tini"));
    for label in ["source", "version", "licenses", "description"] {
        assert!(
            docker.contains(&format!("org.opencontainers.image.{label}")),
            "{label}"
        );
    }
    let entrypoint = read("packaging/docker/entrypoint.sh");
    assert!(entrypoint.contains("GITPOULPI_SAFE_DIRECTORY"));
    assert!(entrypoint.contains("exec tini -- gitpoulpi-tui"));
    // The context the workflow builds is the one the Dockerfile expects.
    let yml = read(".github/workflows/release.yml");
    assert!(yml.contains("docker-ctx/dist/$1"));
    assert!(yml.contains("packaging/docker/entrypoint.sh"));
}

#[test]
fn the_docker_hub_overview_fits_the_limits() {
    let readme = read("packaging/docker/README.md");
    assert!(
        readme.len() < 25_000,
        "Docker Hub's overview limit is 25,000 bytes, this is {}",
        readme.len()
    );
    // Relative links and images break on Docker Hub: every target is absolute.
    for (at, _) in readme.match_indices("](") {
        let target = &readme[at + 2..];
        assert!(
            target.starts_with("https://") || target.starts_with("http://"),
            "relative link: {}",
            target.lines().next().unwrap_or("")
        );
    }
    assert!(!readme.contains("src=\"") || readme.contains("src=\"https://"));
    // The short description is the Dockerfile's, at most 100 characters, and the workflow sends the same text.
    let docker = read("packaging/docker/Dockerfile");
    let description = docker
        .lines()
        .find_map(|l| l.strip_prefix("ARG DESCRIPTION=\""))
        .and_then(|l| l.strip_suffix('"'))
        .expect("ARG DESCRIPTION");
    assert!(description.len() <= 100, "{} characters", description.len());
    let yml = read(".github/workflows/release.yml");
    assert!(yml.contains(&format!("short-description: {description}")));
}

#[test]
fn the_apt_repository_script_signs_and_keeps_five_versions() {
    let script = read("packaging/apt/build-repo.sh");
    for needle in [
        "apt-ftparchive",
        "InRelease",
        "Release.gpg",
        "--clearsign",
        "KEEP:-5",
        "APT_GPG_KEY_ID",
    ] {
        assert!(script.contains(needle), "{needle}");
    }
    let sources = read("packaging/apt/gitpoulpi.sources");
    assert!(sources.contains("Signed-By:"));
    assert!(sources.contains("Types: deb"));
}

#[test]
fn the_packaging_files_hold_no_em_dash() {
    for file in [
        ".github/workflows/release.yml",
        ".github/workflows/release-cache.yml",
        ".github/workflows/ci.yml",
        "packaging/windows/gitpoulpi.iss",
        "packaging/msi/gitpoulpi.wxs",
        "packaging/docker/Dockerfile",
        "packaging/docker/entrypoint.sh",
        "packaging/docker/README.md",
        "packaging/apt/build-repo.sh",
        "packaging/apt/gitpoulpi.sources",
        "packaging/fill-templates.sh",
        "packaging/install.sh",
    ] {
        assert!(!read(file).contains('\u{2014}'), "{file}");
    }
}
#[test]
fn the_dist_profile_uses_thin_lto() {
    let cargo = read("Cargo.toml");
    let dist = cargo.split("[profile.dist]").nth(1).unwrap();
    let dist = dist.split("\n[").next().unwrap();
    assert!(dist.contains("lto = \"thin\""), "{dist}");
    assert!(dist.contains("codegen-units = 16"), "{dist}");
}
/// The `uses: Swatinem/rust-cache` step (with its `with:` block) of a job.
fn rust_cache_step(job_text: &str) -> String {
    steps(job_text)
        .into_iter()
        .find(|s| s.starts_with("uses: Swatinem/rust-cache@"))
        .unwrap_or_else(|| panic!("no rust-cache step in:\n{job_text}"))
}

#[test]
fn the_release_and_the_cache_workflows_share_the_cache_keys_and_only_main_saves() {
    let release = read(".github/workflows/release.yml");
    let warm = read(".github/workflows/release-cache.yml");
    // A tag run reads the cache but never saves it: a tag's cache is invisible to the next tag and would only
    // eat the repository quota.
    for (job_name, key) in [
        ("build", "shared-key: release-${{ matrix.target }}"),
        ("preflight", "shared-key: release-preflight"),
    ] {
        let step = rust_cache_step(&job(&release, job_name));
        assert!(step.contains(key), "release.yml {job_name}: {key}");
        assert!(step.contains("save-if: false"), "release.yml {job_name}");
        let warm_step = rust_cache_step(&job(&warm, job_name));
        assert!(
            warm_step.contains(key),
            "release-cache.yml {job_name}: {key}"
        );
        assert!(
            !warm_step.contains("save-if: false"),
            "release-cache.yml {job_name}"
        );
        // Without shared-key rust-cache appends the job id, and the keys would differ.
        assert!(!step.contains("\n          key:"), "release.yml {job_name}");
        assert!(
            !warm_step.contains("\n          key:"),
            "release-cache.yml {job_name}"
        );
    }
    // The warm workflow runs by hand and weekly, on main only, over the same matrix rows.
    assert!(warm.contains("workflow_dispatch:"));
    assert!(warm.contains("schedule:"));
    assert!(!warm.contains("tags:"));
    assert!(!warm.contains("pull_request"));
    assert!(warm.contains("github.ref == 'refs/heads/main'"));
    let rows = |yml: &str| -> Vec<String> {
        yml.lines()
            .filter(|l| l.trim_start().starts_with("- { os: "))
            .map(|l| l.trim().to_string())
            .collect()
    };
    assert_eq!(rows(&release), rows(&warm));
    assert_eq!(rows(&release).len(), 5);
    // It builds what `xtask dist` builds, with the same profile.
    assert!(warm.contains("cargo build --profile dist --target ${{ matrix.target }}"));
    for package in [
        "gitpoulpi-tui",
        "gitpoulpi-cli",
        "gitpoulpi-mcp",
        "gitpoulpi-updater",
    ] {
        assert!(warm.contains(&format!("-p {package}")), "{package}");
    }
    // The toolchain step (and its pinned SHA) is the same in both files.
    let toolchain = |yml: &str| -> String {
        yml.lines()
            .find(|l| l.contains("dtolnay/rust-toolchain@"))
            .unwrap()
            .trim()
            .to_string()
    };
    assert_eq!(toolchain(&release), toolchain(&warm));
}

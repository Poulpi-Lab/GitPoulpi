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
    assert!(iss.contains("PrivilegesRequiredOverridesAllowed=dialog"));
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
    assert!(iss.contains("AliasPage.Values[0] := 'gp'"), "default alias");
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

    // Unix packages install the four binaries and never the self-update
    // marker: the package manager stays the only updater.
    for file in [
        "packaging/aur/gitpoulpi-bin/PKGBUILD",
        "packaging/aur/gitpoulpi/PKGBUILD",
        "packaging/homebrew/gitpoulpi.rb",
    ] {
        let text = read(file);
        for bin in binaries {
            assert!(text.contains(bin), "{file}: {bin}");
        }
        let code: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect();
        assert!(!code.contains(marker), "{file} installs {marker}");
    }
    // Scoop unpacks the whole zip, so it deletes the marker after install.
    let scoop = read("packaging/scoop/gitpoulpi.json");
    for bin in binaries {
        assert!(scoop.contains(&exe_name(bin, WINDOWS)), "scoop: {bin}");
    }
    assert!(scoop.contains(&format!("Remove-Item \\\"$dir\\\\{marker}\\\"")));
    // winget and Chocolatey run the Inno installer, whose AppId is the
    // uninstall key winget calls ProductCode.
    let iss = read("packaging/windows/gitpoulpi.iss");
    let winget = read("packaging/winget/GitPoulpi.GitPoulpi.installer.yaml");
    let app_id = "8C1D3E5A-6B7F-4A29-9E0C-47A1B2C3D4E5";
    assert!(iss.contains(&format!("AppId={{{{{app_id}}}")));
    assert!(winget.contains(&format!("ProductCode: \"{{{app_id}}}_is1\"")));
}

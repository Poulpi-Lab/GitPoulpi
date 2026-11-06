[English](../installation.md) · [Čeština](../cs/installation.md) · **Deutsch** · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installation

GitPoulpi besteht aus drei ausführbaren Dateien:

| Programm | Was es ist |
|---|---|
| `gitpoulpi-tui` | Die Terminal-Oberfläche. |
| `gitpoulpi` | Die Kommandozeile (`--json` für Skripte). Ohne Unterbefehl öffnet sie die Oberfläche. |
| `gitpoulpi-mcp` | Der MCP-Server für KI-Agenten. Siehe [MCP und Agenten](mcp-and-agents.md). |

Neben diesen enthält das Archiv `gitpoulpi-launch`, ein winziges Programm, das die Verknüpfungen ausführen, um ein bereits geöffnetes GitPoulpi-Fenster zu zeigen (`gitpoulpi launch`). Unter Windows hat es kein Konsolenfenster, ein Klick lässt also nie eines aufblitzen. Es wird mit den anderen installiert, aktualisiert und entfernt.

## Voraussetzungen

- Ein **System-`git`** im `PATH`. GitPoulpi nutzt libgit2 für die meisten
  Vorgänge und Ihr `git` für den Rest.
  Erforderlich ist Version **2.34 oder neuer**. Unter macOS funktioniert das `git` aus den Command Line Tools von Apple ebenso wie das von Homebrew.
- Ein **Terminal** mit Mausunterstützung. kitty, Ghostty, WezTerm und iTerm2 (und Windows Terminal und foot über Sixel)
  zeichnen den Commit-Graphen als Bild. Andere Terminals (die
  Windows-Konsole, tmux und die meisten übrigen) zeigen einen Zeichen-Graphen.
  Siehe [Ein Terminal wählen](terminals.md).
- Eine **Nerd Font** oder eine Schrift mit Emoji, für die Symbole in der
  Symbolleiste und der Seitenleiste.
- Optional: **`git-lfs`** ab Version 3.0 für Git-LFS-Repositories. Ohne es
  erscheinen LFS-Dateien als kleine Zeigerdateien, und GitPoulpi erklärt
  Ihnen den Grund. Zur Installation siehe [Git LFS](lfs.md).
- Optional: ein **Credential-Helper** (Git Credential Manager, libsecret,
  `pass`, Keychain), um Hosting-Tokens und KI-API-Schlüssel zu speichern.
  Siehe [Hosting und Issue-Tracker](hosting-integration.md).

## Aus einem Release-Archiv

Ein Release-Archiv heißt `gitpoulpi-<version>-<target>.zip` (Windows) oder
`.tar.gz` (Linux, macOS) und liegt neben einer `.sha256`-Datei. Prüfen Sie das
Archiv anhand der `.sha256`-Datei, entpacken Sie es und legen Sie die drei
Programme in einen Ordner im `PATH` (lassen Sie `gitpoulpi-updater` und die Markierung
`.gitpoulpi-dist` daneben liegen: Die Markierung sagt `channel=archive`, damit sich der Build selbst
aktualisieren kann, siehe [Updates](updates.md)). `INSTALL.txt` im Archiv nennt dieselben
Schritte.

### Einen Download prüfen

Jede Release-Datei ist mit dem [minisign](https://jedisct1.github.io/minisign/)-Schlüssel des Projekts signiert, und GitPoulpi prüft diese Signatur selbst, bevor es ein Update installiert. Bei einer von Hand heruntergeladenen Datei ist die `.minisig` daneben die Signatur. Der öffentliche Schlüssel (Schlüssel-ID `B461FFCBF6F6A4F2`) liegt als `packaging/minisign.pub` im Repository:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Lesen Sie den vertrauenswürdigen Kommentar, den der Befehl ausgibt: Er muss `file:<der Name der Datei> version:<die erwartete Version>` lauten. Eine gültige Signatur, die eine andere Datei oder eine andere Version nennt, ist keine Signatur für Ihre.

Wurde das Release mit Build-Provenance-Attestierungen gebaut (die Variable `ATTESTATIONS` des Repositorys), prüft die GitHub-CLI, wo eine Datei gebaut wurde:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Die `.sha256`-Datei erkennt nur einen beschädigten Download. Die Signatur zeigt, dass die Datei vom Projekt stammt.

## Mit einem Installationsprogramm

Jedes Release enthält außerdem ein Installationsprogramm je Betriebssystem. Beide treffen dieselben Entscheidungen
(Verknüpfungen, PATH, ein Befehlsalias wie `gp`), indem sie das installierte `gitpoulpi` selbst aufrufen, also den Code
hinter Einstellungen > Allgemein > Starter. Was das Installationsprogramm einrichtet und was die Einstellungen zeigen,
stimmt deshalb immer überein.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Der erste Dialog fragt, für wen die Installation gilt:

| Auswahl | Wo | Administratorrechte | Was es für Updates bedeutet |
|---|---|---|---|
| **Nur für mich** (Standard) | `%LOCALAPPDATA%\Programs\GitPoulpi` | keine | GitPoulpi ersetzt seine eigenen Dateien: **Aktualisieren und neu starten** im [Update-Fenster](updates.md), ohne Abfrage. |
| **Für alle Benutzer** | `Program Files\GitPoulpi` | eine Abfrage bei der Installation | Jedes Update verlangt die Zustimmung eines Administrators (eine Windows-Abfrage), und die GitPoulpi-Sitzungen anderer Benutzer auf dem Computer werden zuvor beendet. |

Dieselbe Auswahl funktioniert still: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` oder `/ALLUSERS`.

Kontrollkästchen: Verknüpfung im Startmenü, Verknüpfung auf dem Desktop, zum PATH hinzufügen. Eine Seite fragt nach dem **Befehlsalias**: standardmäßig `gp`, ein beliebiger Name, oder leer für keinen (`/ALIAS=name` oder `/ALIAS=` in der Befehlszeile). PowerShell belegt `gp` bereits für `Get-ItemProperty`, wählen Sie dort also einen anderen Namen oder nutzen Sie `gitpoulpi`.

Das Installationsprogramm kopiert die Programme (auch `gitpoulpi-updater`, das kleine Programm, das bei einem Update die Dateien austauscht) und die Markierung `.gitpoulpi-dist`. Die Markierung hält fest, wie GitPoulpi installiert wurde (`inno-user` oder `inno-machine`), was das Update-Fenster als **Installiert über** zeigt. Wird das Setup erneut ausgeführt, bleiben Ihre Entscheidungen (Geltungsbereich, Verknüpfungen, Alias) erhalten.

Deinstalliert wird über die Windows-Einstellungen > Apps: Dort läuft `gitpoulpi uninstall` (Alias, PATH-Einträge, Verknüpfungen), und die Dateien werden gelöscht, auch die `.old`-Reste des Updaters. Ihre Einstellungen und Repositories bleiben.

### Linux und macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Es wählt das Archiv für Ihr Betriebssystem und Ihre CPU, prüft dessen `.sha256`, installiert (Kanal `archive`: GitPoulpi aktualisiert sich an Ort und Stelle selbst) nach
`~/.local/opt/gitpoulpi` und verlinkt `gitpoulpi` und `gitpoulpi-mcp` nach `~/.local/bin`. Root wird nicht gebraucht,
und außerhalb Ihres Home-Ordners wird nichts angefasst. Ohne `--yes` fragt es nach dem Alias (Standard `gp`), dem PATH
und den Verknüpfungen.

| Option | Wirkung |
|---|---|
| `--prefix DIR` | Installiert die Programmdateien in `DIR` (absolut). |
| `--version X.Y.Z` | Installiert dieses Release statt des neuesten. |
| `--archive FILE` | Nutzt ein lokales Archiv; `FILE.sha256` muss daneben liegen (`--no-verify` überspringt das). Das Netzwerk wird nicht benutzt. |
| `--alias NAME` / `--no-alias` | Lässt `NAME` GitPoulpi starten (`gitpoulpi alias create`), oder keinen Namen. |
| `--no-path` | Fügt GitPoulpi nicht zum PATH neuer Shells hinzu (sonst `gitpoulpi path add`). |
| `--desktop` | Legt zusätzlich die Verknüpfungen im Anwendungsmenü und auf dem Desktop an. |
| `--yes` | Fragt nie: Die Optionen und die Standardwerte gelten (Alias `gp`, PATH ja, Verknüpfungen nein). |
| `--uninstall` | Führt `gitpoulpi uninstall` aus und entfernt dann die Links und den Ordner. |

Die Suche nach dem neuesten Release setzt ein öffentliches Repository voraus; bei einem privaten laden Sie das Archiv im
Browser herunter und nutzen `--archive`.


## Mit einem Paketmanager

Jeder der folgenden Kanäle schreibt dieselbe Markierung `.gitpoulpi-dist` neben die Programme, mit seinem eigenen Kanalnamen. GitPoulpi liest sie, um zu wissen, wem die Updates gehören: dem Paketmanager, GitPoulpi tauscht daher nie Dateien hinter dessen Rücken aus. Das Update-Fenster (auf die Version in der Titelleiste klicken) führt dann den eigenen Befehl des Paketmanagers aus oder zeigt ihn zum Kopieren. Siehe [Updates](updates.md).

| Kanal | Installation | Hinweise |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Installiert für alle Benutzer, daher verlangen Updates die Zustimmung eines Administrators. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Mit `--scope user` oder `--scope machine` wählen; der Geltungsbereich Computer verlangt die Zustimmung eines Administrators bei der Installation und bei jedem Update. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi`, dann `scoop install gitpoulpi` | Pro Benutzer, keine Administratorrechte. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Der Tap ist `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | siehe [das apt-Repository](#apt-debian-und-ubuntu) | Signiertes Repository; jedem Release hängt außerdem eine `.deb` an. |
| **MSI** (Windows, verwaltet) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Für Administratoren: alle Benutzer, keine Updates in der App. Siehe [Unternehmensverteilung](enterprise-distribution.md#msi-mit-intune-oder-gruppenrichtlinie). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (oder `paru -S gitpoulpi-bin`) | Ohne Helfer: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, dann `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Lädt das Release-Archiv herunter, statt zu kompilieren. |
| **cargo, aus dem Quellcode** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Kompiliert; braucht eine Rust-Toolchain. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Siehe [In Docker ausführen](docker.md). Dasselbe Image liegt auf `ghcr.io/poulpi-lab/gitpoulpi`. |

Die obigen Namen sind die, zu denen der Release-Prozess veröffentlicht (siehe [Releasing](../../developer/releasing.md)): Ein Kanal, der noch nicht live ist, antwortet „not found“, und das Release-Archiv oder das Installationsprogramm funktioniert immer.

Es gilt die eigene PATH-Behandlung des Paketmanagers: Chocolatey, winget, Scoop und Homebrew legen die Programme selbst in den PATH, und keiner von ihnen erzeugt den Alias `gp` (führen Sie `gitpoulpi alias create gp` aus, oder Einstellungen > Allgemein > Starter).

### apt (Debian und Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Das Repository unterstützt `amd64` und `arm64` und ist mit einem Schlüssel signiert, der dem Projekt gehört. Die Programme liegen in `/usr/lib/gitpoulpi`, mit Links in `/usr/bin`. Das Paket hängt von `git` ab und empfiehlt `git-lfs`. Ohne das Repository installieren Sie die `.deb` eines Releases mit `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi sagt Ihnen dann, dass Sie die nächste `.deb` selbst herunterladen sollen, denn es bietet `apt`-Befehle nur an, wenn es das Repository in `/etc/apt/sources.list.d/gitpoulpi.*` findet.

## Aus dem Quellcode

Sie brauchen eine aktuelle stabile Rust-Toolchain (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Die Programme liegen in `target/release/`. Das Profil `release` baut schnell.
Für die kleinsten und schnellsten Programme nehmen Sie das Profil `dist`; es
braucht länger beim Bauen:

```
cargo build --profile dist
```

Die `build-opti`-Skripte im Wurzelverzeichnis des Repositorys führen diesen
Build für die drei Programme aus und geben ihre Größen aus:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd oder PowerShell)
```

Um nur die Sprachen und Funktionen zu bauen, die Sie nutzen, siehe
[Eigene Builds](custom-builds.md).

## Ein Release-Archiv selbst bauen

```
cargo run -p xtask -- dist
```

Das baut das Profil `dist` und schreibt
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) oder `.tar.gz`
(andere Systeme). Das Archiv enthält die drei Programme, `INSTALL.txt`,
`README.md`, `LICENSE` und `CHANGELOG.md`. Daneben entsteht eine
`.sha256`-Datei.

## Updates

Die Titelleiste zeigt die laufende Version (`v1.0.0`) zwischen dem Programmnamen und dem Repository-Pfad. Klicken Sie darauf, oder drücken Sie `Ctrl+N` und dann `u`, oder führen Sie **Updates: Update-Fenster anzeigen** in der Befehlspalette (`Ctrl+P`) aus: Das Update-Fenster öffnet sich mit der Version, dem Installationsweg von GitPoulpi und dem, was bei einem neueren Release zu tun ist. **Hilfe: Versionshinweise** öffnet das Änderungsprotokoll.

Einmal täglich prüft GitPoulpi im Hintergrund an dem Ort, an dem seine Releases veröffentlicht werden (standardmäßig GitHub), ob es ein neueres gibt (`updates.mode`, [Einstellungen > Synchronisierung](settings.md#synchronisierung)). Gibt es eines, steht auf dem Chip `v0.2.0 verfügbar` in der Akzentfarbe.

Was das Fenster tut, hängt vom Installationsweg von GitPoulpi ab: Ein Release-Archiv oder das Windows-Installationsprogramm ersetzt seine eigenen Dateien, ein Paketmanager führt seinen eigenen Befehl aus, eine verwaltete oder Container-Installation sagt, wie Updates sie erreichen. Ein mit `cargo build` erstellter Build aktualisiert sich nie. Die vollständige Beschreibung, mit Zurückgehen, `auto` gegenüber `notify` und dem, was ein Administrator sperren kann, steht unter [Updates](updates.md).

## Prüfen, ob es funktioniert

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` gibt die Version und den Commit aus, aus dem der Build entstand.
Weiter geht es mit [Erste Schritte](getting-started.md).

## Deinstallation

`gitpoulpi uninstall` (das führen beide Installationsprogramme aus) entfernt die von GitPoulpi angelegten
Befehlsaliasse, PATH-Einträge und Verknüpfungen und danach die Programmdateien einer Release-Installation; bei einem Build
ohne die Markierung `.gitpoulpi-dist` (ein cargo-Build) lehnt es ab, und `--keep-files` lässt die Programmdateien stehen.
Für MCP-Agenten ist es nicht verfügbar. Ohne Installationsprogramm löschen Sie die drei Programme. GitPoulpi legt außerdem
diese Dateien an:

| Was | Wo |
|---|---|
| Einstellungen (`config.toml`) und Sitzungen | Der Konfigurationsordner Ihres Betriebssystems. `GITPOULPI_CONFIG_DIR` ändert ihn. |
| Zustand pro Repository | `.gitpoulpi/` in jedem Repository. |
| Caches (Avatare) | Der Cache-Ordner Ihres Betriebssystems. |
| Tokens und API-Schlüssel | Ihr Credential-Helper. **Einstellungen > SSH & Zugangsdaten > Alle Benutzernamen und Passwörter vergessen** entfernt sie. |

Die genauen Pfade stehen in der [Konfigurationsreferenz](config-reference.md).

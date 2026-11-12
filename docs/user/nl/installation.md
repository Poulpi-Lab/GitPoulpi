[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · **Nederlands** · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installatie

GitPoulpi bestaat uit drie uitvoerbare bestanden:

| Uitvoerbaar bestand | Wat het is |
|---|---|
| `gitpoulpi-tui` | De terminal-UI. |
| `gitpoulpi` | De opdrachtregelinterface (`--json` voor scripts). Zonder subcommando opent hij de UI. |
| `gitpoulpi-mcp` | De MCP-server voor AI-agents. Zie [MCP en agents](mcp-and-agents.md). |

Naast die bestanden bevat het archief `gitpoulpi-launch`, een klein programma dat de snelkoppelingen uitvoeren om een al geopend GitPoulpi-venster te tonen (`gitpoulpi launch`). Op Windows heeft het geen consolevenster, dus een klik laat er nooit een oplichten. Het wordt samen met de andere geïnstalleerd, bijgewerkt en verwijderd.

## Wat je nodig hebt

- Een **`git` op je systeem**, vindbaar via je `PATH`. GitPoulpi gebruikt libgit2
  voor de meeste handelingen en jouw `git` voor de rest.
  Versie **2.34 of nieuwer** is nodig. Op macOS werkt de `git` uit Apple's Command Line Tools, en die van Homebrew ook.
- Een **terminal** met muisondersteuning. kitty, Ghostty, WezTerm, Rio en iTerm2 (en ook Windows Terminal, foot en Tabby, via sixel)
  tekenen de commit-grafiek als afbeeldingen. Andere terminals (de
  Windows-console, zellij, screen en de meeste andere) tonen een grafiek van tekens. Zie
  [Een terminal kiezen](terminals.md) en [Multiplexers](multiplexers.md).
- Een **Nerd Font** of een lettertype met emoji, voor de pictogrammen in de
  werkbalk en de zijbalk.
- Optioneel: **`git-lfs`** 3.0 of nieuwer, voor Git LFS-repositories. Zonder
  `git-lfs` verschijnen LFS-bestanden als kleine pointerbestanden en vertelt
  GitPoulpi je waarom. Zie [Git LFS](lfs.md) voor het installeren ervan.
- Optioneel: een **credential helper** (Git Credential Manager, libsecret,
  `pass`, Keychain), om hostingtokens en AI-API-sleutels op te slaan. Zie
  [Hosting en issuetrackers](hosting-integration.md).

## Vanuit een releasearchief

Een releasearchief heet `gitpoulpi-<version>-<target>.zip` (Windows) of
`.tar.gz` (Linux, macOS), met ernaast een `.sha256`-bestand. Controleer het
archief aan de hand van het `.sha256`-bestand, pak het uit en zet de drie
uitvoerbare bestanden in een map op je `PATH` (houd `gitpoulpi-updater` en de
markering `.gitpoulpi-dist` ernaast: de markering zegt `channel=archive`, zodat de build
zichzelf kan bijwerken, zie [Updates](updates.md)). `INSTALL.txt` in het archief geeft dezelfde stappen.

### Een download verifiëren

Elk releasebestand is ondertekend met de [minisign](https://jedisct1.github.io/minisign/)-sleutel van het project, en GitPoulpi controleert die handtekening zelf voordat het een update installeert. Voor een bestand dat je met de hand hebt gedownload, is de `.minisig` ernaast de handtekening. De publieke sleutel (sleutel-id `B461FFCBF6F6A4F2`) staat als `packaging/minisign.pub` in de repository:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Lees de vertrouwde opmerking die de opdracht afdrukt: die moet `file:<de naam van het bestand> version:<de versie die je verwacht>` zijn. Een handtekening die geldig is maar een ander bestand of een andere versie noemt, is geen handtekening voor het jouwe.

Als de release is gebouwd met build-provenance-attestaties (de variabele `ATTESTATIONS` van de repository), controleert de GitHub CLI waar een bestand is gebouwd:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Het `.sha256`-bestand vangt alleen een beschadigde download op. De handtekening laat zien dat het bestand van het project is.

## Met een installatieprogramma

Elke release levert ook een installatieprogramma per besturingssysteem. Beide
maken dezelfde keuzes (snelkoppelingen, PATH, een opdrachtalias zoals `gp`) door
het geïnstalleerde `gitpoulpi` zelf aan te roepen, de code achter Instellingen >
Starter. Wat het installatieprogramma instelt en wat Instellingen toont
komen dus altijd overeen.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Het eerste dialoogvenster vraagt voor wie de installatie is:

| Keuze | Waar | Beheerdersrechten | Wat het voor updates betekent |
|---|---|---|---|
| **Alleen voor mij** (standaard) | `%LOCALAPPDATA%\Programs\GitPoulpi` | geen | GitPoulpi vervangt zijn eigen bestanden: **Bijwerken en herstarten** in het [updatepaneel](updates.md), zonder vraag. |
| **Voor alle gebruikers** | `Program Files\GitPoulpi` | één vraag bij de installatie | Elke update vraagt om goedkeuring van een beheerder (een Windows-vraag), en de GitPoulpi-sessies van andere gebruikers op de computer worden eerst gesloten. |

Dezelfde keuze werkt stil: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` of `/ALLUSERS`.

Selectievakjes: snelkoppeling in het Startmenu, snelkoppeling op het Bureaublad, toevoegen aan PATH. Een pagina vraagt om het **opdrachtalias**: standaard `gp`, elke naam die je wilt, leeg voor geen (`/ALIAS=naam` of `/ALIAS=` op de opdrachtregel). PowerShell gebruikt `gp` al voor `Get-ItemProperty`, kies daar dus een andere naam of gebruik `gitpoulpi`.

Het installatieprogramma kopieert de uitvoerbare bestanden (inclusief `gitpoulpi-updater`, het kleine programma dat tijdens een update bestanden verwisselt) en de markering `.gitpoulpi-dist`. De markering legt vast hoe GitPoulpi is geïnstalleerd (`inno-user` of `inno-machine`), wat het updatepaneel toont als **Geïnstalleerd via**. De setup opnieuw uitvoeren behoudt je keuzes (bereik, snelkoppelingen, alias).

Verwijderen doe je via Windows-instellingen > Apps: dat voert `gitpoulpi uninstall` uit (alias, PATH-items, snelkoppelingen) en verwijdert de bestanden, inclusief de `.old`-restanten van de updater. Je instellingen en repositories blijven staan.

### Linux en macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Het script kiest het archief voor je besturingssysteem en processor, controleert het
`.sha256`-bestand, installeert (kanaal `archive`: GitPoulpi werkt zichzelf ter plekke bij) in `~/.local/opt/gitpoulpi` en koppelt `gitpoulpi` en
`gitpoulpi-mcp` in `~/.local/bin`. Er is geen root nodig en er wordt niets buiten je
thuismap aangeraakt. Zonder `--yes` vraagt het naar het alias (standaard `gp`), de PATH
en de snelkoppelingen.

| Vlag | Effect |
|---|---|
| `--prefix DIR` | Installeer de programmabestanden in `DIR` (absoluut pad). |
| `--version X.Y.Z` | Installeer deze release in plaats van de nieuwste. |
| `--archive FILE` | Gebruik een lokaal archief; `FILE.sha256` moet ernaast staan (`--no-verify` slaat die controle over). Er wordt geen netwerk gebruikt. |
| `--alias NAME` / `--no-alias` | Laat `NAME` GitPoulpi starten (`gitpoulpi alias create`), of geen alias. |
| `--no-path` | Voeg GitPoulpi niet toe aan de PATH van nieuwe shells (anders `gitpoulpi path add`). |
| `--desktop` | Maak ook de snelkoppelingen in het toepassingsmenu en op het Bureaublad. |
| `--yes` | Nooit vragen: de vlaggen en de standaardwaarden gelden (alias `gp`, PATH ja, snelkoppelingen nee). |
| `--uninstall` | Voer `gitpoulpi uninstall` uit en verwijder daarna de koppelingen en de map. |

Het opzoeken van de nieuwste release werkt alleen als de repository openbaar is; voor
een privérepository download je het archief in een browser en gebruik je `--archive`.

## Met een pakketbeheerder

Elk kanaal hieronder schrijft dezelfde markering `.gitpoulpi-dist` naast de programma's, met een eigen kanaalnaam. GitPoulpi leest die om te weten wie de updates beheert: de pakketbeheerder, dus GitPoulpi verwisselt nooit bestanden achter zijn rug om. Het updatepaneel (klik op de versie in de titelbalk) voert dan de eigen opdracht van de pakketbeheerder uit, of toont hem zodat je hem kunt kopiëren. Zie [Updates](updates.md).

| Kanaal | Installeren | Opmerkingen |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Installeert voor alle gebruikers, dus updates vragen om goedkeuring van een beheerder. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Voeg `--scope user` of `--scope machine` toe om te kiezen; machinebereik vraagt om goedkeuring van een beheerder bij de installatie en bij elke update. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` en dan `scoop install gitpoulpi` | Per gebruiker, geen beheerdersrechten. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | De tap is `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | zie [de apt-repository](#apt-debian-en-ubuntu) | Ondertekende repository; een `.deb` is ook bij elke release bijgevoegd. |
| **MSI** (Windows, beheerd) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Voor beheerders: alle gebruikers, geen updates vanuit de app. Zie [Distributie voor organisaties](enterprise-distribution.md#msi-met-intune-of-groepsbeleid). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (of `paru -S gitpoulpi-bin`) | Zonder helper: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, dan `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Downloadt het releasearchief in plaats van te compileren. |
| **cargo, vanuit de broncode** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Compileert; heeft een Rust-toolchain nodig. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Zie [GitPoulpi in Docker draaien](docker.md). Dezelfde image staat op `ghcr.io/poulpi-lab/gitpoulpi`. |

De namen hierboven zijn die waarnaar het releaseproces publiceert (zie [Releasen](../../developer/releasing.md)): een kanaal dat nog niet live is, antwoordt "not found", en het releasearchief of installatieprogramma werkt altijd.

De eigen PATH-afhandeling van een pakketbeheerder geldt: Chocolatey, winget, Scoop en Homebrew zetten de programma's zelf op de PATH, en geen ervan maakt het alias `gp` (voer `gitpoulpi alias create gp` uit, of Instellingen > Starter).

### apt (Debian en Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

De repository ondersteunt `amd64` en `arm64` en is ondertekend met een sleutel die van het project is. De programma's gaan naar `/usr/lib/gitpoulpi` met koppelingen in `/usr/bin`. Het pakket hangt af van `git` en beveelt `git-lfs` aan. Zonder de repository installeer je de `.deb` van een release met `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi zegt dan dat je de volgende `.deb` zelf moet downloaden, omdat het alleen `apt`-opdrachten aanbiedt als het de repository vindt in `/etc/apt/sources.list.d/gitpoulpi.*`.

## Vanuit de broncode

Je hebt een recente stabiele Rust-toolchain nodig (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

De uitvoerbare bestanden staan in `target/release/`. Het profiel `release`
bouwt snel. Voor de kleinste en snelste uitvoerbare bestanden gebruik je het
profiel `dist`. Dat duurt langer om te bouwen:

```
cargo build --profile dist
```

De `build-opti`-scripts in de root van de repository voeren deze build uit voor
de drie uitvoerbare bestanden en tonen hun grootte:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd of PowerShell)
```

Om alleen de talen en functies te bouwen die je gebruikt, zie
[Aangepaste builds](custom-builds.md).

## Zelf een releasearchief bouwen

```
cargo run -p xtask -- dist
```

Dit bouwt het profiel `dist` en schrijft
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) of `.tar.gz` (andere
systemen). Het archief bevat de drie uitvoerbare bestanden, `INSTALL.txt`,
`README.md`, `LICENSE` en `CHANGELOG.md`. Ernaast wordt een `.sha256`-bestand
geschreven.

## Updates

De titelbalk toont de draaiende versie (`v1.0.0`) tussen de naam van de applicatie en het repositorypad. Als er een nieuwere release bekend is (of een update bezig is), opent een klik erop het updatepaneel met de versie, hoe GitPoulpi is geïnstalleerd en wat je met de nieuwe release moet doen; anders opent een klik het tabblad Releasenotes. `Ctrl+N` en dan `u`, of **Updates: Updatepaneel tonen** in het opdrachtenpalet (`Ctrl+P`), opent altijd het updatepaneel, en **Help: Releasenotes** opent altijd het changelog.

Eenmaal per dag controleert GitPoulpi op de achtergrond op de plek waar zijn releases worden gepubliceerd (standaard GitHub) of er een nieuwere is (`updates.mode`, [Instellingen > Synchronisatie](settings.md#synchronisatie)). Als dat zo is, staat er in de chip `v0.2.0 beschikbaar` in de accentkleur.

Wat het paneel doet hangt af van hoe GitPoulpi is geïnstalleerd: een releasearchief of het Windows-installatieprogramma vervangt zijn eigen bestanden, een pakketbeheerder voert zijn eigen opdracht uit, een beheerde of containerinstallatie zegt hoe updates hem bereiken. Een build die met `cargo build` is gemaakt, werkt zichzelf nooit bij. De volledige beschrijving, met teruggaan, `auto` tegenover `notify` en wat een beheerder kan vergrendelen, staat in [Updates](updates.md).

## Controleren of het werkt

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` toont de versie en de commit waarvan de build gemaakt is. Volg
daarna [Aan de slag](getting-started.md).

## Verwijderen

`gitpoulpi uninstall` (wat beide installatieprogramma's uitvoeren) verwijdert de
opdrachtaliassen, de PATH-items en de snelkoppelingen die GitPoulpi heeft gemaakt, en
daarna de programmabestanden van een releaseinstallatie; het weigert op een build zonder
het bestand `.gitpoulpi-dist` (een cargo-build), en `--keep-files` laat de
programmabestanden staan. Het is niet beschikbaar voor MCP-agents. Zonder
installatieprogramma verwijder je de drie uitvoerbare bestanden. GitPoulpi bewaart ook
deze bestanden:

| Wat | Waar |
|---|---|
| Instellingen (`config.toml`) en sessies | De configuratiemap van je besturingssysteem. `GITPOULPI_CONFIG_DIR` wijzigt dit. |
| Status per repository | `.gitpoulpi/` in elke repository. |
| Caches (avatars) | De cachemap van je besturingssysteem. |
| Tokens en API-sleutels | Je credential helper. **Instellingen > SSH & inloggegevens > Alle gebruikersnamen en wachtwoorden vergeten** verwijdert ze. |

Zie [Configuratienaslag](config-reference.md) voor de exacte paden.

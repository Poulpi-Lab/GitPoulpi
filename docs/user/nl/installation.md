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
- Een **terminal** met muisondersteuning. kitty, Ghostty, WezTerm en iTerm2 (en ook Windows Terminal en foot, via sixel)
  tekenen de commit-grafiek als afbeeldingen. Andere terminals (de
  Windows-console, tmux en de meeste andere) tonen een grafiek van tekens. Zie
  [Een terminal kiezen](terminals.md).
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
uitvoerbare bestanden in een map op je `PATH`. `INSTALL.txt` in het archief
geeft dezelfde stappen.

## Met een installatieprogramma

Elke release levert ook een installatieprogramma per besturingssysteem. Beide
maken dezelfde keuzes (snelkoppelingen, PATH, een opdrachtalias zoals `gp`) door
het geïnstalleerde `gitpoulpi` zelf aan te roepen, de code achter Instellingen >
Algemeen > Starter. Wat het installatieprogramma instelt en wat Instellingen toont
komen dus altijd overeen.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Het installeert voor de huidige gebruiker, zonder beheerdersprompt, in
`%LOCALAPPDATA%\Programs\GitPoulpi` (het eerste dialoogvenster biedt een
installatie voor alle gebruikers en de keuze van de installatiemap). Selectievakjes:
snelkoppeling in het Startmenu, snelkoppeling op het Bureaublad, toevoegen aan PATH.
Een pagina vraagt om het **opdrachtalias**: standaard `gp`, elke naam die je wilt,
leeg voor geen. PowerShell gebruikt `gp` al voor `Get-ItemProperty`, kies daar dus
een andere naam of gebruik `gitpoulpi`.

Het installatieprogramma kopieert de drie uitvoerbare bestanden en het bestand
`.gitpoulpi-dist`, zodat [Nu bijwerken](#ter-plekke-bijwerken) precies zo werkt als bij
de zip. Een installatie per gebruiker is beschrijfbaar en kan zichzelf dus bijwerken;
een installatie voor alle gebruikers in Program Files heeft verhoogde rechten nodig
om vervangen te worden, installeer die dus opnieuw met de nieuwe setup. Verwijderen
doe je via Windows-instellingen > Apps: dat voert `gitpoulpi uninstall` uit (alias,
PATH-items, snelkoppelingen) en verwijdert de bestanden, inclusief de `.old`-restanten
van de updater. Je instellingen en repositories blijven staan.

### Linux en macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Het script kiest het archief voor je besturingssysteem en processor, controleert het
`.sha256`-bestand, installeert in `~/.local/opt/gitpoulpi` en koppelt `gitpoulpi` en
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

De titelbalk toont de draaiende versie (`v1.0.0`) tussen de naam van de applicatie en het repositorypad. Klik
erop, of voer **Help: Releasenotes** uit vanuit het opdrachtenpalet (`Ctrl+P`): er opent
een tabblad Releasenotes met het changelog, gescrold naar de draaiende versie.

Eenmaal per dag vraagt GitPoulpi op de achtergrond aan GitHub of er een nieuwere
release bestaat (`updates.mode`, [Instellingen > Synchronisatie](settings.md#synchronisatie)).
Als dat zo is, staat er in de chip `v0.2.0 beschikbaar` in de accentkleur en opent een
klik het tabblad Releasenotes met de notes van die release bovenaan. Het tabblad heeft
drie knoppen, elk met een toets en een opdracht in het palet:

- **Downloaden** (`d`, *Updates: Downloaden*) opent de releasepagina in je browser.
- **Deze versie overslaan** (`s`, *Updates: Deze versie overslaan*) verbergt de melding
  tot er een nieuwere release verschijnt.
- **Nu controleren** (`r`, *Updates: Nu op updates controleren*) vraagt het meteen opnieuw.

`updates.mode = "off"` doet helemaal geen verzoek. Zonder netwerkverbinding, of zolang de
repository privé is en er geen github.com-token is opgeslagen, mislukt de controle
stilletjes (één regel in het applicatielogboek, niets in de interface).
`GITPOULPI_UPDATE_API` richt de controle op een andere API-root (gebruikt door de tests).

### Ter plekke bijwerken

Een build die uit een releasearchief is geïnstalleerd, kan zichzelf vervangen. Het
archief bevat naast de uitvoerbare bestanden een klein bestand `.gitpoulpi-dist`; een
draaiende build is alleen "bijwerkbaar" als dat bestand naast zijn uitvoerbare bestand
staat (en zijn platform noemt). Een build die met `cargo` is gemaakt, een aangepaste
build die ergens anders naartoe is gekopieerd of een pakket dat het bestand wegliet,
werkt zichzelf nooit bij: het tabblad Releasenotes zegt dat en biedt alleen
**Downloaden** aan.

Voor een bijwerkbare build voegt het tabblad toe:

- **Nu bijwerken** (`u`, *Updates: Nu bijwerken*) downloadt het archief en het
  `.sha256`-bestand op de achtergrond (een taak in de indicator van de statusbalk, met
  percentage, te annuleren zoals elke taak), controleert de hash, pakt het archief uit en
  zet elk nieuw bestand naast het oude als `<naam>.new`. Niets wat draait wordt
  aangeraakt. In de modus `auto` gebeurt hetzelfde vanzelf zodra een controle een nieuwere
  release vindt.
- **Herstarten om bij te werken** (`R`, *Updates: Herstarten om bij te werken*; de chip
  zegt hetzelfde zodra de download klaarstaat) is altijd jouw actie, nooit automatisch. Het
  stopt de losse daemon van de sessie (een ingebedde verdwijnt met het venster), verwisselt
  de bestanden en start de nieuwe versie met dezelfde argumenten en sessie.
- **Terug naar de vorige versie** (`v`, *Updates: Terug naar de vorige versie*) zet de vorige
  bestanden terug en herstart. De bestanden van de vorige versie blijven als `<naam>.old`
  staan tot de nieuwe versie een minuut heeft gedraaid, daarna worden ze verwijderd; de
  knop is er dus voor die eerste run. Dit hoort niet bij het ongedaanmaakjournaal van git:
  het wijzigt de eigen bestanden van GitPoulpi, geen repository.

Hoe de bestanden worden vervangen: op Linux en macOS wordt elk nieuw bestand over het oude
heen hernoemd (atomair; een draaiend programma houdt zijn oude kopie tot het stopt). Op
Windows kan een draaiende `.exe` niet worden overschreven, dus het oude bestand wordt
hernoemd naar `<naam>.old` en het nieuwe ervoor in de plaats gezet; een `.old` die nog
draait, wordt door een latere start verwijderd. De wissel is alles of niets: als een
ander programma een van de uitvoerbare bestanden vasthoudt (bijvoorbeeld een MCP-client
die `gitpoulpi-mcp` draait), noemt GitPoulpi dat bestand, wijzigt niets en laat de
download klaarstaan voor een nieuwe poging.

Wat wordt geweigerd, steeds met een melding: een release zonder `.sha256`-bestand, een
hash die niet klopt, een archiefbestand waarvan de naam niet die voor dit platform is
(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), een archief waarvan de markering een
andere versie of een ander platform noemt, en elke downloadlink buiten deze hosts:
`github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` en `github-releases.githubusercontent.com` (het
eerste verzoek en elke redirect; een redirect naar ergens anders stopt de download). De
archieven van een privérepository hebben een browsersessie nodig, dus daar mislukt de
download tot de repository openbaar is.

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

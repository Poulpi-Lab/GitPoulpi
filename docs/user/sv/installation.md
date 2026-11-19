[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · **Svenska** · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installation

GitPoulpi är ett enda program, `gitpoulpi`. Körs det utan underkommando öppnar det terminalgränssnittet; med ett underkommando är det kommandoradsgränssnittet (`--json` för skript); `gitpoulpi mcp serve` är MCP-servern för AI-agenter (se [MCP och agenter](mcp-and-agents.md)). Bredvid det innehåller arkivet `gitpoulpi-updater`, det separata lilla programmet som uppdaterar en installation (se [Uppdateringar](updates.md)).

För kompatibilitet innehåller arkivet också tre små relän som bara startar `gitpoulpi`: `gitpoulpi-tui` (öppnar gränssnittet; det tas bort i en senare version), `gitpoulpi-mcp` (startar MCP-servern; det finns kvar under hela 1.x-linjen, så befintliga agentkonfigurationer fortsätter fungera) och `gitpoulpi-launch` (det genvägarna kör för att visa ett redan öppet GitPoulpi-fönster, `gitpoulpi launch`; i Windows har det inget konsolfönster, så ett klick får aldrig något att blinka till). Allt installeras, uppdateras och tas bort tillsammans.

## Vad du behöver

- Ett **`git` installerat i systemet**, i din `PATH`. GitPoulpi använder libgit2 för de flesta
  operationer och din `git` för resten.
  Version **2.34 eller nyare** krävs. På macOS fungerar `git` från Apples Command Line Tools, och likaså den från Homebrew.
- En **terminal** med musstöd. kitty, Ghostty, WezTerm, Rio, iTerm2 (och även Windows Terminal, foot och Tabby, via sixel) ritar
  commit-grafen som bilder. Andra terminaler (Windows-konsolen, zellij, screen och
  de flesta andra) visar en teckengraf. Se
  [Välja terminal](terminals.md) och [Multiplexers](multiplexers.md).
- Ett **Nerd Font**-typsnitt eller ett typsnitt med emoji, för ikonerna i verktygsfältet
  och sidopanelen.
- Valfritt: **`git-lfs`** 3.0 eller nyare, för Git LFS-repositories. Utan det
  visas LFS-filer som små pekarfiler och GitPoulpi berättar varför. Se
  [Git LFS](lfs.md) för hur du installerar det.
- Valfritt: en **credential helper** (Git Credential Manager, libsecret,
  `pass`, Keychain), för att lagra tokens för värdtjänster och API-nycklar för AI. Se
  [Värdtjänster och ärendehantering](hosting-integration.md).

## Från ett releasearkiv

Ett releasearkiv heter `gitpoulpi-<version>-<target>.zip` (Windows) eller
`.tar.gz` (Linux, macOS), med en `.sha256`-fil bredvid. Kontrollera arkivet mot
`.sha256`-filen, packa upp det och lägg dess körbara filer
(`gitpoulpi`, `gitpoulpi-updater` och relänen `gitpoulpi-tui`, `gitpoulpi-mcp` och `gitpoulpi-launch`) i en mapp i din
`PATH` (behåll `gitpoulpi-updater` och markeringen `.gitpoulpi-dist` bredvid dem:
markeringen säger `channel=archive`, så bygget kan uppdatera sig självt, se
[Uppdateringar](updates.md)). `INSTALL.txt` i arkivet ger samma steg.

### Verifiera en nedladdning

Varje releasefil är signerad med projektets [minisign](https://jedisct1.github.io/minisign/)-nyckel, och GitPoulpi kontrollerar den signaturen själv innan det installerar en uppdatering. För en fil som du har laddat ned för hand är `.minisig` bredvid den signaturen. Den publika nyckeln (nyckel-id `B461FFCBF6F6A4F2`) är incheckad som `packaging/minisign.pub` i repositoryt:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Läs den betrodda kommentaren som kommandot skriver ut: den måste vara `file:<filens namn> version:<den version du förväntar dig>`. En signatur som är giltig men namnger en annan fil eller en annan version är inte en signatur för din.

Där releasen byggdes med attesteringar av byggursprung (repositoryts variabel `ATTESTATIONS`) kontrollerar GitHub CLI var en fil byggdes:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Filen `.sha256` fångar bara en skadad nedladdning. Det är signaturen som visar att filen är projektets.

## Med ett installationsprogram

Varje release innehåller också ett installationsprogram per operativsystem. Båda gör samma val
(genvägar, PATH, ett kommandoalias som `gp`) genom att anropa det installerade
`gitpoulpi` självt, koden bakom Inställningar > Startare, så det som
installationsprogrammet ställer in och det som Inställningar visar stämmer alltid överens.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Den första dialogen frågar vem installationen är till för:

| Val | Var | Administratörsrättigheter | Vad det betyder för uppdateringar |
|---|---|---|---|
| **Bara för mig** (standard) | `%LOCALAPPDATA%\Programs\GitPoulpi` | inga | GitPoulpi ersätter sina egna filer: **Uppdatera och starta om** i [uppdateringspanelen](updates.md), utan fråga. |
| **För alla användare** | `Program Files\GitPoulpi` | en fråga vid installationen | Varje uppdatering ber om administratörsgodkännande (en Windows-fråga), och GitPoulpi-sessionerna för andra användare på datorn stängs först. |

Samma val fungerar tyst: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` eller `/ALLUSERS`.

Kryssrutor: genväg i Startmenyn, genväg på skrivbordet, lägg till i PATH. En sida frågar efter **kommandoaliaset**: `gp` som standard, valfritt namn, tomt för inget (`/ALIAS=name` eller `/ALIAS=` på kommandoraden). PowerShell använder redan `gp` för `Get-ItemProperty`, så välj ett annat namn där eller använd `gitpoulpi`.

Installationsprogrammet kopierar de körbara filerna (inklusive `gitpoulpi-updater`, det lilla programmet som byter filer under en uppdatering) och markeringen `.gitpoulpi-dist`. Markeringen registrerar hur GitPoulpi installerades (`inno-user` eller `inno-machine`), vilket uppdateringspanelen visar som **Installerat med**. Att köra setup igen behåller dina val (omfattning, genvägar, alias).

Avinstallera från Windows Inställningar > Appar: det kör `gitpoulpi uninstall` (alias, PATH-poster, genvägar) och raderar filerna, även uppdateringens kvarlämnade `.old`. Dina inställningar och repositories finns kvar.

### Linux och macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Skriptet väljer arkivet för ditt operativsystem och din processor, kontrollerar dess `.sha256`, installerar (kanalen `archive`: GitPoulpi uppdaterar sig självt på plats) i
`~/.local/opt/gitpoulpi` och länkar `gitpoulpi` (som öppnar terminalgränssnittet när det körs utan underkommando) och `gitpoulpi-mcp` (relät) till
`~/.local/bin`. Ingen root behövs och inget utanför din hemkatalog rörs.
Utan `--yes` frågar det efter alias (standard `gp`), PATH och genvägar.

| Flagga | Effekt |
|---|---|
| `--prefix DIR` | Installera programfilerna i `DIR` (absolut sökväg). |
| `--version X.Y.Z` | Installera den här releasen i stället för den senaste. |
| `--archive FILE` | Använd ett lokalt arkiv; `FILE.sha256` måste ligga bredvid (`--no-verify` hoppar över det). Inget nätverk används. |
| `--alias NAME` / `--no-alias` | Låt `NAME` starta GitPoulpi (`gitpoulpi alias create`), eller inget alias. |
| `--no-path` | Lägg inte till GitPoulpi i PATH för nya skal (annars körs `gitpoulpi path add`). |
| `--desktop` | Skapa också genvägarna i programmenyn och på skrivbordet. |
| `--yes` | Fråga aldrig: flaggorna och standardvärdena gäller (alias `gp`, PATH ja, genvägar nej). |
| `--uninstall` | Kör `gitpoulpi uninstall` och tar sedan bort länkarna och mappen. |

Att slå upp den senaste releasen kräver att repositoryt är publikt; för ett privat
repository, ladda ned arkivet i en webbläsare och använd `--archive`.

## Med en pakethanterare

Varje kanal nedan skriver samma markering `.gitpoulpi-dist` bredvid programmen, med sitt eget kanalnamn. GitPoulpi läser den för att veta vem som äger uppdateringarna: det gör pakethanteraren, så GitPoulpi byter aldrig filer bakom ryggen på den. Uppdateringspanelen (klicka på versionen i titelraden) kör då pakethanterarens eget kommando, eller visar det så att du kan kopiera det. Se [Uppdateringar](updates.md).

| Kanal | Installera | Anmärkningar |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Installerar för alla användare, så uppdateringar ber om administratörsgodkännande. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` och sedan `scoop install gitpoulpi` | Per användare, inga administratörsrättigheter. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Tapen är `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | se [apt-repositoryt](#apt-debian-och-ubuntu) | Signerat repository; en `.deb` bifogas också varje release. |
| **MSI** (Windows, hanterad) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | För administratörer: alla användare, inga uppdateringar i appen. Se [Företagsdistribution](enterprise-distribution.md#msi-med-intune-eller-grupprincip). |
| **cargo** | `cargo binstall gitpoulpi-cli` | Laddar ned releasearkivet i stället för att kompilera. |
| **cargo, från crates.io** | `cargo install --locked gitpoulpi-cli` | Kompilerar; kräver en Rust-toolchain. Installerar `gitpoulpi`, som öppnar terminalgränssnittet när det körs utan underkommando och även serverar MCP (`gitpoulpi mcp`). |
| **cargo, från källkod** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Kompilerar; kräver en Rust-toolchain. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Se [Köra i Docker](docker.md). |

Namnen ovan är de som releaseprocessen publicerar till (se [Släppa en release](../../developer/releasing.md)): en kanal som inte är live än svarar "not found", och releasearkivet eller installationsprogrammet fungerar alltid.

En pakethanterares egen PATH-hantering gäller: Chocolatey, Scoop och Homebrew lägger själva programmen i PATH, och ingen av dem skapar aliaset `gp` (kör `gitpoulpi alias create gp`, eller Inställningar > Startare).

### apt (Debian och Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Repositoryt stöder `amd64` och `arm64` och är signerat med en nyckel som tillhör projektet. Programmen hamnar i `/usr/lib/gitpoulpi` med länkar i `/usr/bin`. Paketet beror på `git` och rekommenderar `git-lfs`. Utan repositoryt installerar du `.deb` från en release med `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi säger då att du ska ladda ned nästa `.deb` själv, eftersom det bara erbjuder `apt`-kommandon när det hittar repositoryt i `/etc/apt/sources.list.d/gitpoulpi.*`.

## Från källkod

Du behöver en aktuell stabil Rust-toolchain (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

De körbara filerna finns i `target/release/`. Profilen `release` bygger snabbt.
För de minsta och snabbaste körbara filerna, använd profilen `dist`. Den tar
längre tid att bygga:

```
cargo build --profile dist
```

Skripten `build-opti` i repositoryts rot kör det här bygget och skriver ut de körbara
filernas storlekar:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd eller PowerShell)
```

För att bara bygga de språk och funktioner du använder, se
[Anpassade byggen](custom-builds.md).

## Bygg ett releasearkiv själv

```
cargo run -p xtask -- dist
```

Det här bygger profilen `dist` och skriver
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) eller `.tar.gz` (övriga
system). Arkivet innehåller de körbara filerna, `INSTALL.txt`,
`README.md`, `LICENSE` och `CHANGELOG.md`. En `.sha256`-fil skrivs bredvid.

## Uppdateringar

Titelraden visar den version som körs (`v1.0.0`) mellan programnamnet och repositoryts sökväg. När en nyare release är känd (eller en uppdatering pågår) öppnar ett klick på den uppdateringspanelen med versionen, hur GitPoulpi installerades och vad du kan göra åt den nya releasen; annars öppnar ett klick fliken Versionsanteckningar. `Ctrl+N` och sedan `u`, eller **Uppdateringar: Visa uppdatering** i kommandopaletten (`Ctrl+P`), öppnar alltid uppdateringspanelen, och **Hjälp: Versionsanteckningar** öppnar alltid ändringsloggen.

En gång per dygn, i bakgrunden, kontrollerar GitPoulpi platsen där dess releaser publiceras (GitHub som standard) efter en nyare (`updates.mode`, [Inställningar > Synkronisering](settings.md#synkronisering)). När det finns en står det `v0.2.0 tillgänglig` på chipet i accentfärgen.

Vad panelen gör beror på hur GitPoulpi installerades: ett releasearkiv eller Windows-installationsprogrammet ersätter sina egna filer, en pakethanterare kör sitt eget kommando, en hanterad installation eller en containerinstallation säger hur uppdateringar når den. Ett bygge gjort med `cargo build` uppdateras aldrig. Den fullständiga beskrivningen, med att gå tillbaka, `auto` mot `notify` och vad en administratör kan låsa, finns i [Uppdateringar](updates.md).

## Kontrollera att det fungerar

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi
```

`--version` skriver ut versionen och den commit bygget gjordes från. Följ sedan
[Komma igång](getting-started.md).

## Avinstallera

`gitpoulpi uninstall` (det som båda installationsprogrammen kör) tar bort kommandoaliasen,
PATH-posterna, genvägarna och filhanterarmenyns poster (med deras App Paths-nyckel i
Windows) som GitPoulpi har skapat, och sedan programfilerna i en
releaseinstallation; det vägrar köra på ett bygge utan markeringen `.gitpoulpi-dist`
(ett cargo-bygge), och `--keep-files` lämnar programfilerna. Det är inte
tillgängligt för MCP-agenter. Utan ett installationsprogram, ta bort programfilerna.
GitPoulpi sparar också följande:

| Vad | Var |
|---|---|
| Inställningar (`config.toml`) och sessioner | Operativsystemets konfigurationsmapp. `GITPOULPI_CONFIG_DIR` ändrar den. |
| Tillstånd per repository | `.gitpoulpi/` i varje repository. |
| Cacher (avatarer) | Operativsystemets cachemapp. |
| Tokens och API-nycklar | Din credential helper. **Inställningar > SSH & inloggning > Glöm alla användarnamn och lösenord** tar bort dem. |

Se [Konfigurationsreferens](config-reference.md) för de exakta sökvägarna.

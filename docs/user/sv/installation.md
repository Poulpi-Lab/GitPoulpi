[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · **Svenska** · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installation

GitPoulpi har tre körbara filer:

| Körbar fil | Vad den är |
|---|---|
| `gitpoulpi-tui` | Terminalgränssnittet. |
| `gitpoulpi` | Kommandoradsgränssnittet (`--json` för skript). Utan underkommando öppnar den gränssnittet. |
| `gitpoulpi-mcp` | MCP-servern för AI-agenter. Se [MCP och agenter](mcp-and-agents.md). |

Bredvid dem innehåller arkivet `gitpoulpi-launch`, ett litet program som genvägarna kör för att visa ett redan öppet GitPoulpi-fönster (`gitpoulpi launch`). I Windows har det inget konsolfönster, så ett klick får aldrig något att blinka till. Det installeras, uppdateras och tas bort tillsammans med de andra.

## Vad du behöver

- Ett **`git` installerat i systemet**, i din `PATH`. GitPoulpi använder libgit2 för de flesta
  operationer och din `git` för resten.
  Version **2.34 eller nyare** krävs. På macOS fungerar `git` från Apples Command Line Tools, och likaså den från Homebrew.
- En **terminal** med musstöd. kitty, Ghostty, WezTerm och iTerm2 (och även Windows Terminal och foot, via sixel) ritar
  commit-grafen som bilder. Andra terminaler (Windows-konsolen, tmux och
  de flesta andra) visar en teckengraf. Se
  [Välja terminal](terminals.md).
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
`.sha256`-filen, packa upp det och lägg de tre körbara filerna i en mapp i din
`PATH`. `INSTALL.txt` i arkivet ger samma steg.

## Med ett installationsprogram

Varje release innehåller också ett installationsprogram per operativsystem. Båda gör samma val
(genvägar, PATH, ett kommandoalias som `gp`) genom att anropa det installerade
`gitpoulpi` självt, koden bakom Inställningar > Allmänt > Startare, så det som
installationsprogrammet ställer in och det som Inställningar visar stämmer alltid överens.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Det installerar för den aktuella användaren, utan administratörsfråga, i
`%LOCALAPPDATA%\Programs\GitPoulpi` (den första dialogen erbjuder en
installation för alla användare och val av installationsmapp). Kryssrutor: genväg i Startmenyn, genväg på
skrivbordet, lägg till i PATH. En sida frågar efter **kommandoaliaset**: `gp` som standard,
valfritt namn, tomt för inget. PowerShell använder redan `gp` för
`Get-ItemProperty`, så välj ett annat namn där eller använd `gitpoulpi`.

Installationsprogrammet kopierar de tre körbara filerna och markeringen `.gitpoulpi-dist`, så att
[Uppdatera nu](#uppdatera-på-plats) fungerar precis som för zip-arkivet. En installation
för en användare är skrivbar och kan därför uppdatera sig själv; en installation för alla användare i
Program Files kräver förhöjda rättigheter för att ersättas, så installera om den med den nya setup-filen i stället.
Avinstallera från Windows Inställningar > Appar: det kör `gitpoulpi uninstall` (alias,
PATH-poster, genvägar) och raderar filerna, även uppdateringens kvarlämnade `.old`.
Dina inställningar och repositories finns kvar.

### Linux och macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Skriptet väljer arkivet för ditt operativsystem och din processor, kontrollerar dess `.sha256`, installerar i
`~/.local/opt/gitpoulpi` och länkar `gitpoulpi` och `gitpoulpi-mcp` till
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

Skripten `build-opti` i repositoryts rot kör det här bygget för de tre körbara
filerna och skriver ut deras storlekar:

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
system). Arkivet innehåller de tre körbara filerna, `INSTALL.txt`,
`README.md`, `LICENSE` och `CHANGELOG.md`. En `.sha256`-fil skrivs bredvid.

## Uppdateringar

Titelraden visar den version som körs (`v1.0.0`) mellan programnamnet
och repositoryts sökväg. Klicka på den, eller kör **Hjälp: Versionsanteckningar** i kommandopaletten (`Ctrl+P`): en flik Versionsanteckningar öppnas med ändringsloggen, rullad
till den version som körs.

En gång per dygn frågar GitPoulpi GitHub i bakgrunden om det finns en nyare release
(`updates.mode`, [Inställningar > Synkronisering](settings.md#synkronisering)).
När det finns en står det `v0.2.0 tillgänglig` på chipet i accentfärgen och ett klick på det
öppnar fliken Versionsanteckningar med den releasens anteckningar överst. Fliken har
tre knappar, var och en med en tangent och ett palettkommando:

- **Ladda ned** (`d`, *Uppdateringar: Ladda ned*) öppnar releasesidan i din webbläsare.
- **Hoppa över den här versionen** (`s`, *Uppdateringar: Hoppa över den här versionen*) döljer meddelandet
  tills en nyare release dyker upp.
- **Sök nu** (`r`, *Uppdateringar: Sök efter uppdateringar nu*) frågar igen direkt.

`updates.mode = "off"` gör ingen förfrågan alls. Utan nätverksanslutning,
eller medan repositoryt är privat och ingen github.com-token är lagrad, misslyckas kontrollen
tyst (en rad i programloggen, ingenting i gränssnittet).
`GITPOULPI_UPDATE_API` pekar kontrollen mot en annan API-rot (används av testerna).

### Uppdatera på plats

Ett bygge som installerats från ett releasearkiv kan ersätta sig självt. Arkivet innehåller
en liten fil `.gitpoulpi-dist` bredvid de körbara filerna; ett bygge som körs är
”uppdateringsbart” bara när den filen ligger bredvid dess körbara fil (och namnger dess
plattform). Ett bygge gjort med `cargo`, ett anpassat bygge som kopierats någon annanstans eller ett
paket som tappat filen uppdaterar sig aldrig själv: dess flik Versionsanteckningar säger
det och erbjuder bara **Ladda ned**.

För ett uppdateringsbart bygge lägger fliken till:

- **Uppdatera nu** (`u`, *Uppdateringar: Uppdatera nu*) laddar ned arkivet och dess
  `.sha256` i bakgrunden (ett jobb i statusradens indikator, med sin
  procentsats, avbrytbart som vilket jobb som helst), kontrollerar hashen, packar upp arkivet
  och lägger varje ny fil bredvid den gamla som `<namn>.new`. Inget som körs
  rörs. I läget `auto` sker samma sak av sig självt så snart en kontroll hittar en
  nyare release.
- **Starta om för att uppdatera** (`R`, *Uppdateringar: Starta om för att uppdatera*; chipet säger
  detsamma när nedladdningen väl är förberedd) är alltid din åtgärd, aldrig automatisk. Den
  stoppar sessionens fristående daemon (en inbäddad försvinner med fönstret), byter
  filerna och startar den nya versionen med samma argument och
  session.
- **Gå tillbaka till föregående version** (`v`, *Uppdateringar: Gå tillbaka till föregående version*) byter
  tillbaka de tidigare filerna och startar om. Den tidigare versionens filer finns kvar som
  `<namn>.old` tills den nya versionen har körts i en minut, sedan raderas
  de, så knappen finns kvar under den första körningen. Den ingår inte i
  gits ångrajournal: den ändrar GitPoulpis egna filer, inte ett repository.

Hur filerna ersätts: i Linux och macOS döps varje ny fil om över den
gamla (atomiskt; ett program som körs behåller sin gamla kopia tills det avslutas). I
Windows kan en `.exe` som körs inte skrivas över, så den gamla döps om till
`<namn>.old` och den nya flyttas in; en `.old` som fortfarande körs raderas
vid en senare start. Bytet är allt eller inget: om ett annat program håller en av
de körbara filerna (en MCP-klient som kör `gitpoulpi-mcp`, till exempel) namnger GitPoulpi
filen, ändrar ingenting och lämnar nedladdningen förberedd för ett nytt försök.

Vad som vägras, var och en med ett meddelande: en release utan `.sha256`-fil, en hash
som inte stämmer, en arkivfil vars namn inte är det för den här plattformen
(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), ett arkiv vars markering namnger
en annan version eller plattform, och varje nedladdningslänk utanför dessa värdar: GitHubs
`github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` och `github-releases.githubusercontent.com`
(den första förfrågan och varje omdirigering; en omdirigering någon annanstans stoppar
nedladdningen). Ett privat repositorys arkiv kräver en webbläsarsession, så
nedladdningen misslyckas där tills repositoryt är publikt.

## Kontrollera att det fungerar

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` skriver ut versionen och den commit bygget gjordes från. Följ sedan
[Komma igång](getting-started.md).

## Avinstallera

`gitpoulpi uninstall` (det som båda installationsprogrammen kör) tar bort kommandoaliasen,
PATH-posterna och genvägarna som GitPoulpi har skapat, och sedan programfilerna i en
releaseinstallation; det vägrar köra på ett bygge utan markeringen `.gitpoulpi-dist`
(ett cargo-bygge), och `--keep-files` lämnar programfilerna. Det är inte
tillgängligt för MCP-agenter. Utan ett installationsprogram, ta bort de tre körbara filerna.
GitPoulpi sparar också följande:

| Vad | Var |
|---|---|
| Inställningar (`config.toml`) och sessioner | Operativsystemets konfigurationsmapp. `GITPOULPI_CONFIG_DIR` ändrar den. |
| Tillstånd per repository | `.gitpoulpi/` i varje repository. |
| Cacher (avatarer) | Operativsystemets cachemapp. |
| Tokens och API-nycklar | Din credential helper. **Inställningar > SSH & inloggning > Glöm alla användarnamn och lösenord** tar bort dem. |

Se [Konfigurationsreferens](config-reference.md) för de exakta sökvägarna.

[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · **Nederlands** · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="De octopus van GitPoulpi" width="128"></p>

**Een Git-client voor de terminal die aanvoelt als een desktopapp.** GitPoulpi
geeft je een volledige commitgrafiek waarin je kunt klikken, slepen en
rechtsklikken, een opdrachtenpalet voor het toetsenbord dat elke functie
bereikt, integratie met codehostingdiensten (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; in de cloud of zelf beheerd) en issuetrackers (Jira, Trello, Linear, YouTrack, Redmine of een map met Markdown-notities), en dezelfde handelingen als scriptbare CLI en
als MCP-server, zodat een AI-agent het precies zo kan bedienen als jij. GitPoulpi is
geïnspireerd door GitKraken en helemaal opnieuw in Rust geschreven: native binaries, zuinig met schijfruimte en geheugen, geen Electron, geen account, geen telemetrie. Het draait in je favoriete terminal ([welke?](../../docs/user/nl/terminals.md)), want sommigen van ons wonen daarin.

![GitPoulpi toont de commitgrafiek met branches, tags en een stash, links de zijbalk Refs en rechts de werkwijzigingen met het commitformulier](../../docs/user/images/hero.png)

- **Muis en toetsenbord, even volwaardig.** Klik, dubbelklik, rechtsklik en
  sleep refs naar commits; of druk op `Ctrl+P` en typ wat je wilt. Elk
  muisgebaar heeft een toetsenbordtegenhanger en elke opdracht staat in het
  palet.
- **Een echte commitgrafiek.** Branches, merges, tags, stashes en de werkkopie
  in één scrollende Log, getekend als pixels in terminals die dat ondersteunen
  en als tekens overal elders. Veel donkere en lichte kleurthema's,
  auteursavatars, zoeken.
- **Stage nauwkeurig.** Stage, unstage of verwerp een bestand, een map, een hunk
  of één enkele regel rechtstreeks vanuit de diff. Inline of naast elkaar, met
  een minimap.
- **Bewerk de geschiedenis veilig.** Herformuleer of wijzig elke commit
  (bericht, auteur, datums), doe een interactieve rebase met een
  controlescherm, herschrijf veel commits tegelijk met een voorbeeld en één
  keer ongedaan maken, cherry-pick meerdere commits, fast-forward, maak
  worktrees en patches.
- **Hosting en issuetrackers ingebouwd.** Blader door pull en merge requests (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket), check ze uit en maak ze, en review Gerrit-changes. Blader door issues, work items en kaarten (ook Jira, Trello, Linear, YouTrack, Redmine en lokale Markdown-notities) en maak ze aan, desgewenst op een Kanban-bord. Kloon vanuit je account en beheer SSH-sleutels.
- **CLI en MCP voor agents.** `gitpoulpi <commando> --json` dekt alles wat de UI
  doet, en `gitpoulpi-mcp` biedt het aan als MCP-tools. Eén achtergronddaemon per
  repository houdt elke frontend live en gesynchroniseerd.
- **Optionele AI, nooit verplicht.** Standaard uit en zonder account: gebruik je
  eigen API-sleutel of een lokale agent-CLI zoals `claude -p`. Genereer
  commitberichten, leg wijzigingen uit, schrijf beschrijvingen voor pull
  requests, splits je werkwijzigingen in meerdere commits en stel regels op
  voor het herschrijven van de geschiedenis.
- **Git LFS, submodules, worktrees, patches, blame en bestandsgeschiedenis,
  afbeeldingsdiffs, een ingebouwde editor.**

## Schermafbeeldingen

| | |
|---|---|
| ![De Log-grafiek met het kleurthema Material en rijkleuring ingeschakeld](../../docs/user/images/theme-material.png) **De commitgrafiek**, hier in het thema Material met rijkleuring. | ![De diff van een gewijzigd bestand met knoppen Hunk stagen en Hunk verwerpen bij elke hunk](../../docs/user/images/diff.png) **De diff**, met stage en verwerpen per hunk of per regel. |
| ![Het opdrachtenpalet gefilterd op het woord stash](../../docs/user/images/palette.png) **Het opdrachtenpalet** (`Ctrl+P`) bereikt elke opdracht. | ![De commitcomposer die vier commits voorstelt voor de werkwijzigingen, klaar om te controleren](../../docs/user/images/composer.png) **De commitcomposer** laat door AI voorgestelde commits controleren voordat er iets wordt gecommit. |
| ![Het rechtsklikmenu van een commit met acties voor cherry-pick, revert, reset, rebase, branch en tag](../../docs/user/images/context-menu.png) **Rechtsklik** op alles voor de bijbehorende acties. | ![De pagina AI van de Instellingen met profielen, instructies per actie en de aan/uit-schakelaar](../../docs/user/images/settings-ai.png) **De Instellingen**, met zoekfunctie, inclusief de optionele pagina AI. |

Meer afbeeldingen, met uitleg, staan in de [gebruikersdocumentatie](../../docs/user/nl/README.md).

## Installeren

Je hebt een `git` op je systeem nodig; `git-lfs` is optioneel.

### Linux en macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Windows-installatieprogramma

Download `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` van de
[nieuwste release](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest) en
voer het uit. Voor beheerders is er de MSI:

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

Zie [Draaien in Docker](../user/nl/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Releasearchief

Download het van de
[nieuwste release](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest),
pak het uit en volg `INSTALL.txt` erin.

### Vanuit de broncode

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Je eigen releasearchief, een kleinere aangepaste build en opmerkingen per
besturingssysteem:
[docs/user/nl/installation.md](../user/nl/installation.md) en
[docs/user/nl/custom-builds.md](../user/nl/custom-builds.md).

## Snel starten

```
cd path/to/your/repository
gitpoulpi
```

- `?` opent de help met elke toets en elk gebaar, `Ctrl+P` het opdrachtenpalet,
  `Ctrl+,` de Instellingen, `q` sluit af.
- Klik op een commit om de wijzigingen te zien; klik op de bovenste rij van de
  Log (de werkwijzigingen) om bestanden te stagen en te committen.
- In een terminal die het ondersteunt (kitty, Ghostty, WezTerm, iTerm2 (en ook Windows Terminal en foot, via sixel)) wordt de
  grafiek als pixels getekend. In andere (een gewone Windows-console,
  tmux) valt hij automatisch terug op tekens.

De stapsgewijze versie staat in [Aan de slag](../../docs/user/nl/getting-started.md),
en de [rondleiding door de interface](../../docs/user/nl/interface-tour.md) benoemt elk
deel van het scherm.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

AI is optioneel en standaard uit: schakel de eigen AI van GitPoulpi in via
*Instellingen > AI* ([AI-ondersteuning](../../docs/user/nl/ai.md)); laat een externe
agent GitPoulpi bedienen met [MCP en agents](../../docs/user/nl/mcp-and-agents.md)
(rechten, elke tool) en de agentskill
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Documentatie

De volledige gebruikersdocumentatie begint bij **[docs/user/nl/README.md](../../docs/user/nl/README.md)**:

| | |
|---|---|
| [Aan de slag](../../docs/user/nl/getting-started.md) | Installeren, een repository openen, een eerste commit maken |
| [Rondleiding door de interface](../../docs/user/nl/interface-tour.md) | Elk deel van het venster, benoemd |
| [Werken met de Log](../../docs/user/nl/log-and-history.md) | Grafiek, refs, branches, tags, stashes, rebase, cherry-pick |
| [Stagen en committen](../../docs/user/nl/staging-and-committing.md) | Status, diff, hunks en regels, de composer, de editor |
| [Instellingen](../../docs/user/nl/settings.md) | Elke pagina en optie van de Instellingen |
| [AI-ondersteuning](../../docs/user/nl/ai.md) | De ingebouwde AI instellen, elke AI-actie, privacy |
| [Hosting en issuetrackers](../../docs/user/nl/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: tokens, een repository koppelen, pull en merge requests |
| [CLI](../../docs/user/nl/cli-reference.md) en [MCP](../../docs/user/nl/mcp-and-agents.md) | Automatiseer het, of laat een agent het bedienen (rechten, elke tool) |
| [Sneltoetsen](../../docs/user/nl/keybindings.md) (en elke paletopdracht), [muis](../../docs/user/nl/mouse-and-gestures.md), [configuratie](../../docs/user/nl/config-reference.md), [LFS](../../docs/user/nl/lfs.md), [ongedaan maken](../../docs/user/nl/undo-redo.md), [probleemoplossing](../../docs/user/nl/troubleshooting.md) | Naslag en hulp |

Ontwikkelaars: [CLAUDE.md](../../CLAUDE.md) en `docs/architecture/overview.md`
beschrijven de crates en het ontwerp; [CHANGELOG.md](../../CHANGELOG.md) toont wat er
in elke release zit.

## Licentie

[MIT](../../LICENSE).

## Dankbetuiging en licenties van derden

GitPoulpi is gebouwd op open-sourcebibliotheken en gebruikt kleurwaarden uit gepubliceerde thema's. Hun licenties en dankbetuigingen, thema-auteurs inbegrepen, staan in [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt). Het bestand wordt met elke release meegeleverd; het venster Over GitPoulpi en `gitpoulpi licenses` tonen het ook.

## Handelsmerken

GitKraken is een merk van de respectieve eigenaar. GitPoulpi is een onafhankelijk project, niet gelieerd aan, goedgekeurd door of gesponsord door deze eigenaar; de naam wordt alleen genoemd om compatibiliteit en inspiratie te beschrijven.

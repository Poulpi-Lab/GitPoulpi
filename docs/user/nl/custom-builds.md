[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · **Nederlands** · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Aangepaste builds

Voor wie een kleinere binary wil: een interactief script vraagt welke talen, hostingintegraties, thema's, logokleuren en functies je wilt opnemen, en bouwt dan alleen die met het geoptimaliseerde profiel. De normale build en de officiële release blijven ongewijzigd (alles aan).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd of PowerShell (scripts/build-custom.ps1)
```

De uitvoer komt in `target/custom/`, nooit over de officiële `target/dist/` heen. Het script toont de grootte van elke binary en het verschil met `target/dist/` als die build bestaat. Het schrijft ook `BUILD-FEATURES.txt` naast de binaries.

## Wat je kiest

| Keuze | Als je het uitzet |
|---|---|
| Talen | Alleen Engels (altijd gebouwd) plus de talen die je aanvinkt. De taalkiezer in de Instellingen toont alleen gebouwde talen. |
| Thema's | GitPoulpi Dark, GitPoulpi Light en Terminal worden altijd gebouwd (`system` heeft Dark en Light nodig). Elk ander thema wordt per familie gekozen (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` toont ze). Niet-gekozen thema's worden niet meegecompileerd: ze ontbreken in de keuzelijst van de Instellingen en in `gitpoulpi theme list`, en een configuratie die er een noemt valt terug als bij een onbekend thema, met de gebruikelijke waarschuwing. |
| Logokleuren | De pixeloctopus van het venster Over komt in 20 kleurstellingen (`--list-logo-colors`). Kies degene die je meecompileert (minstens één), de standaard (Over toont die eerst; het is ook waarop een opgeslagen `ui.logo_color` terugvalt die niet is meegecompileerd; hij moet een van de gecompileerde zijn; officiële standaard: lavender) en de kleur van het pictogram van het Windows-uitvoerbestand (elke kleurstelling, standaard: de logostandaard). Met één enkele kleurstelling draait het logo nog steeds bij een klik maar blijft die kleur, en de kleurenlijst (`c`, rechtsklik) wordt niet aangeboden. |
| Integraties (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Zonder: alleen klonen via URL; geen issues, geen pull/merge requests. |
| AI | Geen AI-knoppen, -opdrachten of `ai` CLI/MCP-tools. |
| MCP | Geen `gitpoulpi-mcp`-binary en geen `gitpoulpi mcp`-opdracht. De `ui`-agentopdrachten blijven. |
| LFS, Worktree, Submodules | De bijbehorende beheerschermen en opdrachten zijn weg; er komt een getypeerd antwoord "not built into this binary" terug. |
| Avatar-download | Alleen initialenbadges, geen netwerkophaal. |
| Afbeeldingsvoorbeeld | Afbeeldingsbestanden tonen de gewone binaire melding; **Bewerken in** werkt nog. |
| Radar | Geen tabblad Radar, `Alt+R` of paletopdracht; de Radar opent nooit. Vereist een integratie (zonder is het toch werkloos). |
| Mermaid | Geen Kopiëren als Mermaid gitGraph (Log-menu, `Shift+Y`, palet) en geen `log --format mermaid` in de CLI/MCP. Ook geen knop **Diagram openen** onder een `mermaid`-blok van een [gesprek](hosting-integration.md#5c-gesprekken) (het blok blijft code) en geen paletrij *Gesprek: Diagram van item openen*. |
| Bord | Geen kanbanbordweergave (knop in de kop ISSUES, palet) en geen opdracht `hosting board`. Vereist een integratie. |
| Gesprek | Geen [gespreksweergave](hosting-integration.md#5c-gesprekken) van een issue of verzoek in de app: Enter op een issuerij, een bordkaart of een Radar-rij opent de browser zoals eerder, er is geen rij *Gesprek openen* of paletcommando *Gesprek: ...*, en geen **Schrijven met AI** in een gesprek (de schrijfvakken van de review, het issueformulier en het pull-requestformulier houden de hunne). De gesprekscommando's van de CLI en MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) en `ai hosting-reply` blijven. Vereist een integratie. |
| Review | Geen codereviewweergave in de app (rijmenu, Enter op een pull-requestrij, palet, `Review` van de Radar) en geen AI-reviewhulp. De CLI/MCP-opdrachten `hosting review` blijven (Gerrit en agents gebruiken ze). Vereist een integratie. |
| Werkbank voor geschiedenis (geavanceerd herschrijven van de geschiedenis: auteurs, datums, berichten, inhoud) | Geen werkbank voor geschiedenis herschrijven: geen `Herschrijven:`-paletopdrachten, geen werkbankcombinatie, -toetsen of helpsectie, geen rijen "Geschiedenis herschrijven" in de Log-menu's, geen knop "Geschiedenis dieper wijzigen..." in het venster Amend, geen "Regels maken met AI" of script-AI, geen scriptlimieten op de pagina Sync van de instellingen en geen AI-blokken voor het herschrijven van de geschiedenis in de AI-instellingen. De CLI/MCP-opdrachten `rewrite`, `ai rewrite-script`, `ai rewrite-rules` en `ui propose rewrite` zijn uit de opdrachtenboom verdwenen. `.mailmap`-identiteiten (`mailmap`, `log.use_mailmap`) horen er niet bij en blijven. |
| Globaal zoeken | Geen knop Globaal zoeken in de titelbalk, geen `Ctrl+Alt+F`, geen paletopdracht `Zoeken: Werkruimte...` en geen CLI/MCP-opdracht `gitpoulpi search`. De zoekbalk van de Log (`ui show search`) is een andere functie en blijft. |
| Werkruimtes | Het venster werkt met de ene standaardtabbladlijst ("Geen werkruimte"): geen werkruimtechip, geen tabblad Werkruimtes, geen `Alt+W` / `Alt+J`, geen `Werkruimte:`-paletopdrachten, geen rijen "Aan werkruimte toevoegen" bij tabbladen, geen CLI/MCP-opdrachten `gitpoulpi workspace`, geen vlag `--workspace` bij `commit` en `search`. Tabbladen en sessies werken als gewoonlijk. Een `workspaces.toml` die een andere build schreef, wordt niet gelezen of gewijzigd; het standaardprofiel en de AI-instructies van een werkruimte gelden niet. De Radar en Globaal zoeken beslaan de open tabbladen. |
| Herschrijfscripts (ingebouwde Lua) | Script-regels van een geschiedenisherschrijving worden geweigerd met een getypeerd antwoord "niet in deze binary ingebouwd", en er wordt geen Lua gelinkt. De werkbank voor geschiedenis zelf blijft (die heeft een eigen keuze). Zinloos, en met een waarschuwing, als de werkbank voor geschiedenis uit staat. |

Je configuratiebestanden worden gedeeld tussen builds: een instelling voor iets dat niet is gebouwd, blijft bewaard en wordt genegeerd.

## Interactief

Het Bash-script gebruikt checklists van `whiptail` of `dialog` (drie schermen, dan een samenvatting), of genummerde prompts als geen van beide bestaat. Het PowerShell-script gebruikt `Out-GridView` als dat beschikbaar is, anders prompts. Je vorige keuzes zijn vooraf aangevinkt.

## Niet-interactief

Elk argument slaat de vragen over en gaat rechtstreeks naar `cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # vorige keuzes, alleen de cargo-opdracht tonen
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Optie | Betekenis |
|---|---|
| `--langs fr,de` / `all` / `en` | Talen naast Engels. De codes zijn de namen van de localebestanden (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Providers: GitHub, GitLab (ook zelf beheerd), Azure DevOps (Services en Server), Bitbucket (Cloud en Data Center) (hosting), Gitea en Forgejo incl. Codeberg (`gitea`) (hosting), Jira, Trello, Linear, YouTrack (Cloud en Server), Redmine (issuetrackers), Gerrit Code Review (`gerrit`, alleen review), lokale Markdown-issues (`local-issues`, een map of Obsidian-vault). `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello` en `youtrack` zijn elk een eigen cargo-feature; groottes in [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Themafamilies om mee te compileren (standaard `all`). `--list-themes` toont de families. |
| `--logo-colors all` / `violet,teal` | Logokleurstellingen om mee te compileren (standaard `all`, in de volgorde van de tabel, die de draaivolgorde is). `--list-logo-colors` toont ze (`*` = officiële standaard). |
| `--logo-default teal` | De standaardkleurstelling; moet bij de gecompileerde horen (standaard: lavender, anders de eerste gecompileerde). |
| `--icon-color sun` | Kleur van het pictogram van het Windows-uitvoerbestand (standaard: de logostandaard). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Elk staat standaard aan. |
| `--[no-]history-rewrite` | De werkbank voor geschiedenis (geavanceerd herschrijven van de geschiedenis: auteurs, datums, berichten, inhoud). Standaard aan. |
| `--[no-]global-search` | Globaal zoeken (knop in de titelbalk, `Ctrl+Alt+F`, `gitpoulpi search`). Standaard aan. |
| `--[no-]workspaces` | Werkruimtes (benoemde groepen repositories). Standaard aan; uit blijft alleen de standaardtabbladlijst over. |
| `--[no-]rewrite-scripts` | Ingebouwde Lua voor de Script-regels van een herschrijving. Standaard aan; met een waarschuwing genegeerd als de werkbank voor geschiedenis uit staat. |
| `--save <name\|file>` | Slaat de bepaalde keuzes op als opgeslagen selectie (een naam of een bestandspad). |
| `--preset <name\|file>` | Begint met een opgeslagen selectie (een naam of een bestandspad); de andere opties overschrijven die, waar ze ook in de regel staan. |
| `--list-presets`, `--delete-preset <name>` | Toont de opgeslagen selecties en hun map / verwijdert er een op naam. |
| `--last` | Hergebruikt `target/custom-build/last-choices.env`; andere opties overschrijven die. |
| `--full` | Geen enkele selectie (alles aan). |
| `--profile custom\|dist\|release` | Standaard `custom` (de `dist`-instellingen, aparte uitvoermap). |
| `--dry-run` | Toon de exacte cargo-opdracht en stop. |
| `-- <cargo args>` | Doorgegeven aan cargo, bijvoorbeeld `--target x86_64-unknown-linux-musl`. |

PowerShell-namen: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

De themaselectie bereikt de build als omgevingsvariabele
`GITPOULPI_THEMES` (`all`, `none` of een kommalijst van families), gelezen door het buildscript van `gitpoulpi-config`, dat de andere paletten tijdens het compileren weglaat; stel hem zelf in voor een gewone `cargo build` als je dat liever hebt. `gitpoulpi
--version` toont een regel `themes:` met de set die is gebouwd.

De logokeuzes zijn de omgevingsvariabelen `GITPOULPI_LOGO_COLORS`
(`all` of een kommalijst van slugs), `GITPOULPI_LOGO_DEFAULT` en
`GITPOULPI_ICON_COLOR`, gelezen door de buildscripts van `gitpoulpi-tui` (dat `assets/logo/colourways.json` filtert tot de tabel die het venster Over insluit) en van de drie binarycrates (het pictogram). De bron van de kleuren blijft `assets/logo/colourways.json`.

### Pictogram van het Windows-uitvoerbestand

Als het doel Windows is, krijgen de uitvoerbare bestanden `gitpoulpi`, `gitpoulpi-tui` en `gitpoulpi-mcp` de octopus als pictogram, tijdens de build gegenereerd (geen Python of ImageMagick nodig) in de gekozen pictogramkleur, formaten 16 tot 128 px. Het wordt ingesloten met de crate `winresource` en heeft de resourcecompiler van het platform nodig (`rc.exe` uit de Windows SDK met MSVC, `windres` met GNU). Ontbreekt die, dan toont de build alleen een waarschuwing en houdt het uitvoerbare bestand het standaardpictogram. Met de GNU-toolchain draagt alleen `gitpoulpi` het pictogram (de resource zou anders twee keer worden gelinkt). Builds voor Linux en macOS negeren dit alles behalve de kleurentabel.

## Opgeslagen selecties

Om een nieuwe versie te bouwen zonder de hele selectie opnieuw te doen, sla je haar onder een naam op en begin je er later weer mee. Opties die je samen met `--preset` opgeeft, overschrijven de opgeslagen waarden nog steeds.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # bewaar deze keuzes als "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # begin er later mee
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # een bestandspad kan ook
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Opgeslagen selecties zijn gewone `SLEUTEL=waarde`-bestanden (het formaat van `last-choices.env`, leesbaar voor een shell). Ze staan in de map `custom-builds` van de configuratiemap van GitPoulpi, buiten `target/`, dus `cargo clean` verwijdert ze niet:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (standaard `~/.config/gitpoulpi/custom-builds/`)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` verplaatst die map samen met de configuratie. `--preset` en `--save` accepteren ook een bestandspad (een waarde met `/` of `\`, of die op `.env` eindigt), om een selectie in een dotfiles-repository te bewaren.

Een selectie die door een andere versie van GitPoulpi is geschreven, wordt toch geladen. Een onbekende sleutel, of een taal, integratie, thema of kleur die niet meer bestaat, wordt met een waarschuwing genegeerd. Een sleutel die in het bestand ontbreekt (een functie die later is toegevoegd) krijgt zijn standaardwaarde, en een later toegevoegde integratie wordt opgenomen; beide worden op de standaardfoutuitvoer gemeld.

Als er een opgeslagen selectie bestaat, biedt de interactieve modus eerst "Beginnen met" aan (vorige keuzes, een opgeslagen selectie of alles aan) en vraagt aan het eind of de huidige selectie onder een naam moet worden opgeslagen.

## Wat het bespaart

Gemeten op Linux op 2026-10-29 bij commit `65999ef5c`, vanaf de volledige build, het verschil daarmee tussen haakjes
(details in `docs/developer/binary-size.md`):

| Build (vanaf `full` tenzij anders vermeld) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Volledig (alles aan) | 64,6 | 50,5 | 31,7 |
| Alleen Engels | 52,0 (-12,6) | 40,2 (-10,3) | 31,7 (0,0) |
| MCP uit | 61,6 (-3,0) | 49,6 (-0,8) | niet gebouwd |
| AI uit | 62,6 (-2,0) | 48,6 (-1,8) | 30,5 (-1,3) |
| Alle integraties uit | 56,3 (-8,2) | 43,7 (-6,7) | 26,7 (-5,0) |
| AI + integraties + avatar-ophaal uit | 52,2 (-12,3) | 39,9 (-10,6) | 23,0 (-8,8) |
| Werkbank voor geschiedenis uit (`--no-history-rewrite`) | 61,6 (-3,0) | 48,9 (-1,5) | 28,5 (-3,2) |
| Herschrijfscripts uit | 63,1 (-1,5) | 49,0 (-1,5) | 31,1 (-0,7) |
| Globaal zoeken uit | 63,5 (-1,1) | 49,5 (-1,0) | 31,6 (-0,1) |
| Werkruimtes uit | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,2) |
| Radar uit | 63,4 (-1,2) | 49,3 (-1,2) | 31,7 (0,0) |
| Bord uit | 63,6 (-1,0) | 49,5 (-1,0) | 31,7 (0,0) |
| Review uit | 63,2 (-1,4) | 49,0 (-1,4) | 31,7 (0,0) |
| Mermaid uit | 63,6 (-1,0) | 49,5 (-0,9) | 31,6 (-0,1) |
| LFS uit | 63,5 (-1,1) | 49,5 (-0,9) | 31,5 (-0,3) |
| Worktree uit | 63,7 (-0,9) | 49,6 (-0,8) | 31,7 (0,0) |
| Submodules uit | 63,7 (-0,9) | 49,6 (-0,9) | 31,6 (-0,1) |
| Avatar-ophaal uit | 63,7 (-0,9) | 49,6 (-0,9) | 31,7 (0,0) |
| Afbeeldingsvoorbeeld uit | 63,3 (-1,2) | 49,2 (-1,3) | 31,7 (0,0) |
| Alles uit, alleen Engels (LFS behouden) | 35,2 (-29,4) | 28,9 (-21,5) | niet gebouwd |
| **Alles uit, alleen Engels** | **34,9 (-29,7)** | **28,8 (-21,6)** | niet gebouwd |

## Opmerkingen

- Een aangepaste build is traag zoals elke `dist`-build (fat LTO, een paar minuten).
- Hij draait zonder sccache, zoals `build-opti`.
- Taalcodes worden gecontroleerd tegen de meegeleverde locales; een onbekende code wordt geweigerd met de lijst van geldige.

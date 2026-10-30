[English](../settings.md) · [Čeština](../cs/settings.md) · [Deutsch](../de/settings.md) · [Español](../es/settings.md) · [Français](../fr/settings.md) · [Italiano](../it/settings.md) · [日本語](../ja/settings.md) · [한국어](../ko/settings.md) · **Nederlands** · [Polski](../pl/settings.md) · [Português (Brasil)](../pt-BR/settings.md) · [Português (Portugal)](../pt-PT/settings.md) · [Русский](../ru/settings.md) · [Svenska](../sv/settings.md) · [Türkçe](../tr/settings.md) · [Українська](../uk/settings.md) · [简体中文](../zh-CN/settings.md)

# Instellingen

Open de Instellingen met `Ctrl+,`, `Ctrl+N` en dan `,`, het tandwiel in de titelbalk
(links van de `×`), of `Ctrl+P` > *Instellingen*. Het scherm Instellingen vervangt het
venster zolang het open is. Sluit het met de `✕` rechtsboven, `Esc` of `q`. Het vraagt alleen eerst
om bevestiging als sluiten iets zou doen verliezen (zie [Instellingen sluiten](#instellingen-sluiten)).

Wijzigingen worden meteen opgeslagen in `config.toml`. De
[configuratiereferentie](config-reference.md) beschrijft elke sleutel, zijn standaardwaarde
en zijn omgevingsvariabele. Deze pagina beschrijft het scherm Instellingen.

![Het scherm Instellingen: links een zoekvak en de paginalijst, rechts de pagina Algemeen](../images/settings.png)

## Het scherm Instellingen gebruiken

| Wil je | Doe dan |
|---|---|
| Wisselen tussen de paginalijst en de opties | `Tab` / `Shift+Tab`, of klik. |
| Bewegen | `j` / `k` of de pijltoetsen. |
| Een optie wijzigen | `Enter` of `Space`, of klik op de waarde. Zie hieronder. |
| Sluiten | Klik op `✕`, of druk op `Esc` of `q`. Zolang een veld, een keuzelijst of een bevestiging open is, sluit `Esc` alleen dat. |

Wat `Enter` of `Space` doet hangt van de rij af:

- Een selectievakje schakelt om.
- Een keuzelijst opent. De pijltoetsen kiezen, `Enter` accepteert.
- Een tekst- of getalveld begint met bewerken. De pijltoetsen, `Home`, `End`, `Backspace` en
  `Delete` werken, en een klik plaatst de cursor. `Enter`, `Tab`, `Up`, `Down` of een klik
  elders slaat de waarde op. `Esc` annuleert. Een geweigerde waarde blijft in rood in het
  veld staan en de statusbalk geeft de reden.
- Een rij met knoppen (**Maken**, **Verwijderen**, **Testen**...) toont alleen de knoppen die nu van
  toepassing zijn. `Left` / `Right` kiezen er een, `Enter` drukt hem in, of klik erop. Een knop die
  iets verwijdert, vraagt eerst om bevestiging.

Op elke pagina beginnen de waarden in één kolom, zodat de labels links en de velden rechts op één lijn
staan. Een tekst of getal is een getinte band, een keuze is een band die eindigt op `▾`, een selectievakje is
`[x]`. Een lang label wordt afgebroken in plaats van afgesneden, en in een smal venster komt de waarde onder
het label te staan.

Een grijze rij zegt waarom ze inactief is. **Sterkte van rijtint** is bijvoorbeeld grijs
zolang **Achtergrond van grafiekbanen** op **Uit** staat.

### Lijsten

Pagina's die meerdere items van dezelfde soort bewerken (Profielen, AI, Workflow, Openen in, Afbeeldingen)
tonen die als een **lijst links en de velden van het geselecteerde item rechts**. In een smal venster staat de
lijst boven de velden. De lijst heeft overal dezelfde vorm:

- Het merkteken van een rij toont zijn status: `●` het actieve profiel, `★` het standaarditem, `⚠` een item
  dat niet af is (een profiel zonder e-mailadres), met de reden op de plek van de details.
- Onder de lijst: **+ Toevoegen**, **Dupliceren**, **Verwijderen**, `↑` en `↓`. Een knop die nu niet kan
  (`↑` op de eerste rij) is gedimd. **+ Toevoegen ▾** opent een keuzelijst waar de pagina voorinstellingen of
  soorten aanbiedt.
- **Verwijderen** vraagt eerst om bevestiging, behalve voor een item dat je net hebt toegevoegd en niet hebt
  bewerkt.
- Sleep een rij om hem te verplaatsen; een balk toont waar hij terechtkomt en `Esc` annuleert het slepen.
  Een dubbelklik gaat naar de velden van het item.

| Toets | Doet |
|---|---|
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Een item selecteren. |
| `Insert` | Een item toevoegen (opent de keuzelijst als die er is). |
| `Ctrl+D` | Het item dupliceren. |
| `Delete` | Het item verwijderen, na een bevestiging. |
| `Alt+Up` / `Alt+Down` | Het item omhoog of omlaag verplaatsen. |
| `Enter` of `Right` | Naar de velden van het item gaan. |
| `Left` | Terug naar de paginalijst. |
| `Alt+Left` / `Alt+Right` | Wisselen tussen het tabblad Algemeen en het lijsttabblad (overal op de pagina). |

Elk van deze pagina's heeft onder de titel twee **subtabbladen**, zodat een instelling van de pagina nooit voor een eigenschap van het geselecteerde item wordt aangezien: **Algemeen**, de eigen instellingen van de pagina (het actieve profiel, het standaarditem, de instructies die elke AI-actie krijgt, het workflowsjabloon ...), en een tabblad met de naam van de items (**Profielen**, **Profielen (verbindingen)**, **Items**, **Editors**, **Regels**) met de lijst en de velden van het geselecteerde item. Een pagina opent op **Algemeen**; de instellingen onthouden het tabblad waarop je elke pagina verliet tot je afsluit, net als de scrollpositie en de geselecteerde rij van elke pagina (en elk tabblad). Klik op een tabblad of druk op `Alt+Left` / `Alt+Right`. Het zoekvak, de rijen **Instellingen:** van het opdrachtenpalet en de koppelingen die "stel het in via de instellingen" zeggen, openen het tabblad waarop de instelling staat.
[Toetsenbord & muis](#toetsenbord--muis) heeft dezelfde twee tabbladen (**Algemeen**, **Sneltoetsen**).

De toetsen kunnen worden herbonden in **Toetsenbord & muis** (context *preferences*, zie
[Sneltoetsen](keybindings.md)).

### Editors met meerdere regels

De instellingen die meerdere regels bevatten (zoals de paden van de gedeeltelijke checkout, de beschrijving van de commitsjabloon, de basisbranches van conflictpreventie en de workflowhooks) openen met `Enter` of een klik in een kleine editor; `Ctrl+S` bewaart de tekst en `Esc` laat hem vallen. De muis werkt daar zoals in elk tekstvak:

- Een klik plaatst de cursor waar je klikte.
- Slepen selecteert tekst; het wiel scrolt.
- Een dubbelklik selecteert het woord onder de aanwijzer.
- `Shift` + klik breidt de selectie uit van de cursor tot de aanwijzer.

Het toetsenbord kan hetzelfde: `Shift` met de pijltoetsen selecteert, `Ctrl` met de pijltoetsen springt per woord, `Ctrl+A` selecteert alles, `Ctrl+C` en `Ctrl+X` kopiëren en knippen.

### Instellingen sluiten

De `✕`, `Esc` en `q` doen hetzelfde. De Instellingen passen elke wijziging meteen toe, dus meestal sluiten ze
gewoon. Ze vragen **Instellingen sluiten?** alleen als er iets verloren zou gaan:

- de pagina **Workflow** heeft niet-opgeslagen wijzigingen;
- het sjabloon voor commitberichten heeft niet-opgeslagen wijzigingen;
- een profiel heeft geen e-mailadres van de auteur (de commits die het maakt, kunnen niet aan jou worden
  toegeschreven).

Het venster biedt **Ga ernaartoe** (opent die pagina), **Verwerpen en sluiten** of **Toch sluiten**, en
**Annuleren**. De kleine editors binnen de Instellingen (een AI-instructietekst, een workflowhook, de paden
van de gedeeltelijke checkout) hebben een eigen `✕` en vragen **Wijzigingen verwerpen?** alleen als hun tekst
is bewerkt.

### Pop-ups over de Instellingen

De titelregel toont de octopus en de naam van de applicatie, net als die van het hoofdvenster. Klik op de octopus
om Over te openen.

Een paar pop-ups voor het hele venster openen over de Instellingen en geven ze terug zoals ze waren (de pagina,
de scrollpositie en niet-opgeslagen wijzigingen aan Workflow of sjablonen blijven behouden):

- het opdrachtenpalet (`Ctrl+P`),
- de themakiezer (`Ctrl+N y`, **App: Thema wijzigen...**), die elk thema in de Instellingen zelf laat zien;
  `Esc` zet het vorige thema terug,
- de kiezers voor taal en grafiekkleuren (`Ctrl+N` en dan dezelfde letters als in het hoofdvenster),
- het zoeken in de werkruimte (`Ctrl+Alt+F`),
- Over (`Ctrl+N a`, de octopus, of **Help: Over GitPoulpi**) en het helpoverlay (`?`).

Sluit elk met `Esc`, zijn eigen `✕` of een klik erbuiten. Opdrachten die werken op wat de Instellingen bedekken
(een tabblad, een branch, een diff...) staan in het palet maar zijn grijs met "sluit eerst de Instellingen", en
de `Ctrl+N`-combinaties ervoor tonen die regel op de onderrand van de Instellingen: sluit de Instellingen en voer
ze dan uit.

`Ctrl+N` toont een badge op elk bedieningselement van de pagina: het zoekvak, de paginalijst (één badge, op de huidige pagina), de subtabbladen, de lijst en de knoppen ervan, elk veld op het scherm en het sluitkruis. Druk op het teken van een badge en het werkt als een klik op dat element: een tekstveld gaat open om te bewerken, een selectievakje wordt aangevinkt, een keuzelijst gaat open, een knop wordt uitgevoerd. `Esc` annuleert. Een veld dat uit beeld is gescrold heeft geen badge. Terwijl je in een veld typt, of een keuzelijst, een editor of het zoeken openstaat, is `Ctrl+N` van dat onderdeel. Zie [Badges](keybindings.md#badges).

### Het zoekvak

Het vak bovenaan de paginalijst doorzoekt de naam en de beschrijving van elke optie op elke
pagina. Klik erop, of druk op `/` of `Ctrl+F`, en typ. Er klapt een lijst met treffers uit.
Elke treffer toont de optie en haar pagina. `Up` / `Down` kiezen, `Enter` gaat naar de optie,
`Esc` sluit de lijst.

![De Instellingen doorzoeken op "diff": de overeenkomende opties van de pagina's Log & grafiek, Diff & editor en Git](../images/settings-search.png)

## Instellingen via het opdrachtenpalet

De meeste instellingen kun je wijzigen zonder dit scherm te openen: typ `Instellingen:` in het opdrachtenpalet (`Ctrl+P`). De rijen worden gegenereerd uit de velden van de pagina's, dus een nieuwe instelling verschijnt op beide plaatsen.

- Een selectievakje is **Instellingen: ... omschakelen**. Het schakelt direct om en de rij toont `[x]` of `[ ]`.
- Een keuzelijst is **Instellingen: ...**. `Enter` beperkt het palet tot de waarden, de huidige is gemarkeerd met `huidig`; `Enter` op een waarde kiest die.
- Een getal of tekst opent een invoerveld met de huidige waarde. Een waarde die de pagina zou weigeren (geen getal, buiten het bereik) wordt op dezelfde manier geweigerd en het veld blijft open.
- Een instelling die haar pagina nodig heeft (een lijst, een keuzelijst met opties die pas tijdens het draaien bekend zijn) opent Instellingen daarop.

![Het opdrachtenpalet gefilterd op Instellingen: met één regel per instelling en rechts de huidige waarde](../images/palette-settings.png)

Een waarde wordt opgeslagen in `config.toml` en precies toegepast zoals de pagina dat doet. De Engelse woorden van de sleutel van de instelling (`remember tabs`) vinden de rij in elke taal.

## De pagina's

De paginalijst heeft twee groepen:

- **Instellingen**: globale instellingen.
- **Repositoryspecifieke instellingen**: instellingen voor de repository van het actieve
  tabblad. Ze verschijnen alleen zolang een repository open is.

### Algemeen

![De keuzelijst Taal van de pagina Algemeen: Auto en de zeventien talen, met een zoekveld](../images/settings-language.png)

| Optie | Wat ze doet |
|---|---|
| **Taal** | De taal van de interface: **Auto** (je systeemtaal, anders Engels), English, Français, Español, Português (Brasil), Português (Portugal), Italiano, Deutsch, 简体中文, 日本語, 한국어, Русский, Українська, Polski, Čeština, Nederlands, Svenska, Türkçe. Geldt meteen, zonder herstart. De maandnamen en datums van de Log volgen haar. De omgevingsvariabele `GITPOULPI_LANG` (bijvoorbeeld `GITPOULPI_LANG=fr`) overschrijft deze keuze. De uitvoer van CLI en MCP blijft in het Engels. |
| **Git-termen vertalen** | Standaard uit: commit, push, pull, stash, tag en de andere git-termen blijven in elke taal Engels. Aan: het eigen git-vocabulaire van de taal wordt gebruikt (Frans: *Tirer*, *Pousser*, *Remiser*, *Étiquette*). Geldt meteen. |
| **Engelse git-woorden** | Alleen voor Japans, Koreaans en Chinees: toont *stage*, *unstage*, *staged* als Engelse woorden in plaats van hun transcriptie. Geen effect zolang **Git-termen vertalen** aan staat. |
| **Tabbladen onthouden** | Heropent bij het starten de tabbladen van de vorige sessie. Uit opent een start binnen een repository alleen die repository, en een start daarbuiten begint op Start zonder tabblad voor een repository. Vereist een herstart. |
| **Radar inschakelen** | Standaard aan (`general.radar_enabled`). Uit verbergt het Radar-tabblad, de opdrachten van het palet ervan en `Alt+R`, en op de achtergrond wordt er niets voor opgehaald. Werkt direct. Zie [de Radar](radar.md#het-tabblad-sluiten-of-de-radar-uitzetten). Niet zichtbaar in een build zonder Radar. |
| **Openingstabblad binnen een repository** | Het tabblad waarop een start opent als de map waarin hij begint binnen een repository ligt (`general.startup_tab_in_repo`): **Huidige repository** (standaard, het tabblad van die repository, indien nodig geopend), **Laatste tabblad** (het tabblad dat actief was bij het laatste afsluiten), **Start**, **Radar**. Een repository die met `--repo` is opgegeven opent altijd zijn eigen tabblad. **Laatste tabblad** is Start als **Tabbladen onthouden** uit staat of er niets onthouden is; **Radar** is Start als Radar uit staat of niet in de build zit. Eenmalig gelezen bij het opstarten; vereist een herstart. |
| **Openingstabblad buiten een repository** | Het tabblad waarop een start opent als de map waarin hij begint geen repository is (bijvoorbeeld een gebruikersmap) (`general.startup_tab_outside_repo`): **Start** (standaard), **Laatste tabblad**, **Radar**. **Laatste tabblad** is Start als **Tabbladen onthouden** uit staat of er niets onthouden is; **Radar** is Start als Radar uit staat of niet in de build zit. Eenmalig gelezen bij het opstarten; vereist een herstart. |
| **Openingstabblad vanuit de starter** | Het tabblad waarop een start opent als GitPoulpi wordt gestart vanaf de snelkoppeling op het bureaublad of in het Startmenu die het zelf maakte (die geeft `--launcher` mee), welke map ook (`general.startup_tab_launcher`): **Laatste tabblad** (standaard), **Start**, **Radar**. Heeft voorrang op de twee instellingen hierboven. **Laatste tabblad** is Start als **Tabbladen onthouden** uit staat of er niets onthouden is; **Radar** is Start als Radar uit staat of niet in de build zit. Snelkoppelingen van een oudere versie krijgen de optie bij de volgende start. Eenmalig gelezen bij het opstarten; vereist een herstart. Op Windows wordt een kopie die je aan de taakbalk hebt vastgemaakt bij de volgende start ook herschreven, en een start zonder shell eromheen (dubbelklik, de vastgemaakte exe zelf) telt als een start vanuit de starter. |
| **Worktree-tabbladen groeperen** | Standaard aan (`general.group_worktree_tabs`). De tabbladen van de gekoppelde worktrees van een repository staan direct na het tabblad van de repository, en een weggesleept worktree-tabblad springt daar terug. Uit laat tabbladen waar je ze neerzet. Aanzetten groepeert de open tabbladen opnieuw. |
| **Standaard kloonmap** | Waar het dialoogvenster Klonen begint. Leeg: `%USERPROFILE%\source\repos` op Windows, `~/src` elders. |
| **Standaard bladermap** | Waar Bladeren opent. Leeg: de laatste map die je gebruikte, anders je thuismap. |
| **Opslaglocatie van repositorygegevens** | Waar GitPoulpi zijn gegevens per repository bewaart (`session.per_repo_state_dir`): **In .git** (standaard: er verschijnt niets in je werkmap, alle worktrees delen het), **In de werkmap** (een map `.gitpoulpi`; nodig om een `workflow.toml` voor het team te committen) of **In mijn gebruikersgegevensmap** (buiten de repository). Een map `.gitpoulpi` die een oudere versie achterliet, wordt de eerste keer dat ze nodig is naar de nieuwe plek verplaatst en verwijderd zodra ze leeg is; een bestand dat git volgt (een gecommitte `workflow.toml`) wordt nooit verplaatst of verwijderd en blijft leesbaar. |
| **Standaardnaam van branch** | De eerste branch van een repository die je met Init maakt. Leeg: git beslist. Hernoemt geen bestaande branch. |
| **Proactieve conflictdetectie inschakelen** | Standaard aan (`general.conflict_detection`). Waarschuwt in de Log als het mergen van de huidige branch in de branch waarvan hij is gestart zou conflicteren, voordat je het probeert. Een repository kan dit voor zichzelf uitschakelen op zijn pagina **Conflictpreventie**. Zie [Merge-conflicten oplossen](merge-conflicts.md#conflictpreventie). Geldt zodra Instellingen sluit. |
| **Logniveau**, **Logbestanden bewaren**, **Maximale grootte van een logbestand** | Het applicatielogboek achter het activiteitenlogboek: het meest uitgebreide niveau dat wordt vastgelegd (`log.level`), hoe lang bestanden bewaard blijven (een geheel getal en een eenheid, dagen, weken, maanden of jaren, op één regel: klik op het getal om het te typen, Omhoog en Omlaag verhogen of verlagen het, Tab gaat naar de eenheid en Enter opent de lijst ervan; hooguit 1 tot 10 jaar, standaard 1 week, `log.retention_days` en `log.retention_unit`) en de grootte waarbij het bestand van een dag wordt geroteerd (een getal en MiB of GiB, 1 MiB tot 1 GiB, `log.max_file_mb` en `log.max_file_unit`). |
| **Duur van bewerkingen tonen**, **Uitgebreid loggen** | Het tabblad Git-bewerkingen van het activiteitenlogboek: de duur van elke bewerking (`log.timings`, standaard aan) en ook het achtergrondwerk, vernieuwen en auto-fetch (`log.extended`, standaard uit). Zie [Wanneer er iets mislukt](troubleshooting.md#het-activiteitenlogboek). |
| **Ongedaan maken ingeschakeld**, **Diepte van de ongedaanmaakgeschiedenis**, **Wijzigingen buiten GitPoulpi vastleggen** | De groep **Geschiedenis van ongedaan maken**. Zet Ongedaan maken aan of uit en bepaalt hoeveel bewerkingen het bewaart (standaard 100, `0` betekent onbeperkt). Ongedaan maken uitzetten wist de ongedaanmaakgeschiedenis, na een bevestiging. De laatste legt ook vast wat een shell of een agent met branches, tags, stashes en HEAD doet. Zie [Ongedaan maken en opnieuw](undo-redo.md). |

#### Starter (bureaubladintegratie)

De groep **Starter**, de laatste op de pagina Algemeen, maakt een snelkoppeling die GitPoulpi
start vanuit de starter van het besturingssysteem, in de terminal die je kiest.

| Rij | Wat ze doet |
|---|---|
| **Terminal voor snelkoppelingen** | `ui.launch_terminal`: **Automatisch**, **Standaard van het systeem** of een van de terminals die op deze computer zijn gevonden (dezelfde detectie als bij **Openen in**). Automatisch: Windows Terminal op Windows (anders WezTerm, Alacritty, de gewone console), `xdg-terminal-exec` op Linux (Omarchy; anders de eerste gevonden terminal), Terminal op macOS, tenzij iTerm2, Ghostty, WezTerm of kitty is geïnstalleerd. |
| **Bij dubbelklik opnieuw openen in die terminal** | Alleen Windows (`ui.relaunch_in_terminal`). Dubbelklikken op de exe opent de standaardconsole, die afbeeldingen slecht tekent: GitPoulpi opent zich opnieuw in de gekozen terminal en het eerste venster sluit. Gebeurt niet als het vanuit een terminal is gestart. |
| **Geopend venster tonen bij klikken** | `ui.reuse_window` (standaard aan). Een klik op een GitPoulpi-snelkoppeling (`gitpoulpi launch`) toont het venster van een GitPoulpi dat al open staat, in plaats van een nieuw te starten. Er start alleen een nieuwe terminal als er geen open is, de instelling uit staat of het venster niet naar voren kan komen. Het komt naar voren op Windows, in WezTerm en op Hyprland; op andere Linux-bureaubladen start de klik nog steeds een nieuw venster. Niet gebruikt op macOS (het Dock doet dat al). Houd **Shift** ingedrukt bij het klikken (Windows) of gebruik de starteractie **New Window** (Linux) om bewust een extra venster te openen; `gitpoulpi launch --new-window` doet het vanuit een script. |
| **Opdrachtalias** | `ui.command_alias`: een keuzelijst met **Geen** en `gp`, of typ een naam naar keuze (letters, cijfers, `-`, `_`, `.`; geen spaties of padscheidingstekens). De knop **Maken** in de rij laat die naam GitPoulpi starten (hij wordt **Verwijderen** zodra het alias bestaat, en verwijdert elk alias dat GitPoulpi heeft gemaakt). Onder de rij verschijnt een waarschuwing als de naam op je PATH al iets anders uitvoert, en op Windows als PowerShell een ingebouwd alias met die naam heeft (`gp` is daar `Get-ItemProperty`, en dat gaat voor op elk bestand). Een shell-alias of -functie is van hieruit niet te zien: oh-my-zsh definieert `gp` zelf en dat gaat voor op het onze. |
| **Alias nieuw tabblad** | `ui.command_alias_add`: een tweede opdracht, bijvoorbeeld `gpa`, die de huidige map als nieuw tabblad opent in het al draaiende GitPoulpi (`gitpoulpi --add`) in plaats van een tweede te starten; draait er geen, dan start ze GitPoulpi zoals gewoonlijk, en bij meerdere vraagt ze bij welke je wilt aansluiten. Een keuzelijst met **Geen**, `gpa` en de aliasnaam gevolgd door `a`, of typ een naam naar keuze (dezelfde regels als hierboven). De ene knop **Maken** maakt beide opdrachten, **Verwijderen** verwijdert elke opdracht die GitPoulpi heeft gemaakt; elk heeft een eigen statusrij. Op Linux en macOS is het een script van twee regels, op Windows een `.cmd`. Zie de [CLI-referentie](cli-reference.md#een-map-toevoegen-aan-het-open-venster). |
| **GitPoulpi toevoegen aan PATH** | Zet de map waarvan GitPoulpi draait en de aliasmap op je PATH, voor nieuwe terminals. De knop van de rij heet **Toevoegen** of **Verwijderen**, en **Verwijderen** haalt precies weg wat is toegevoegd. Mappen die al op PATH staan, blijven ongemoeid (de rij zegt dan **staat al op PATH**). |
| **Linux: App-starter** | Schrijft `~/.local/share/applications/gitpoulpi.desktop` (volgt `XDG_DATA_HOME`) en het pictogram in `~/.local/share/icons/hicolor/256x256/apps`. Op Omarchy is het item hetzelfde als wat `omarchy-tui-install` maakt (`xdg-terminal-exec --app-id=TUI.tile -e <exe>`); voor andere terminals de bijbehorende opdracht (`kitty`, `wezterm start --`, `alacritty -e`, `ghostty -e`, `foot`, ...). |
| **Linux: Bureaublad** | Hetzelfde item in je bureaubladmap, als je er een hebt. |
| **Windows: Startmenu** | `GitPoulpi.lnk` in `%APPDATA%\Microsoft\Windows\Start Menu\Programs`. Een aan de taakbalk vastgemaakte kopie volgt wijzigingen van pictogram en kleur vanzelf; blijft het pictogram oud, maak de kopie dan los en weer vast (rechtermuisknop op het item in het Startmenu, dan **Aan taakbalk vastmaken**). |
| **Windows: Bureaublad** | `GitPoulpi.lnk` op je bureaublad. |
| **macOS: Map Programma's** | `~/Applications/GitPoulpi.app`, een kleine bundel die de terminal opent met GitPoulpi erin. |

Elke rij toont haar toestand: **niet gemaakt**, **gemaakt**, **verouderd, wordt bijgewerkt...**
(de exe is verplaatst of er is een andere terminal gekozen; GitPoulpi schrijft hem binnen een seconde zelf opnieuw, dus je ziet dit zelden) of
**er staat een bestand dat niet van GitPoulpi is** (er bestaat een bestand met die naam dat niet door GitPoulpi is gemaakt, dus
het wordt nooit aangeraakt). **Verwijderen** wist alleen bestanden die GitPoulpi heeft
gemaakt (ze dragen de markering `X-GitPoulpi=1`). Het pictogram is de octopus in je
logokleur (Over GitPoulpi > kleur); op Linux heeft elke kleur zijn eigen pictogramnaam, dus een starter die pictogrammen in de cache bewaart toont de nieuwe kleur. Is een snelkoppeling eenmaal gemaakt, dan blijft ze zelf actueel: wijzig je de kleur of verhuist de terminal of de exe, dan schrijft GitPoulpi de snelkoppelingen die het maakte opnieuw (nooit een die het niet maakte, en het maakt er nooit een aan), bij het starten en tijdens het draaien. Er is geen knop Bijwerken; alleen **Maken** (voor een snelkoppeling die nog niet bestaat) en **Verwijderen**.

**Groeperen in taakbalk en Dock.** GitPoulpi draait in een terminalvenster, dus de taakbalk toont het normaal onder het pictogram van de terminal, zonder “actief”-stip onder het vastgemaakte GitPoulpi-pictogram. Windows: de snelkoppelingen dragen de AppUserModelID `GitPoulpi.GitPoulpi`, en een GitPoulpi dat vanuit een ervan is gestart geeft zijn terminalvenster dezelfde ID en zijn pictogram (het vindt het venster door het even een unieke titel te geven en zet de titel daarna terug), zodat het venster onder het vastgemaakte GitPoulpi-pictogram staat. Een terminalvenster met meerdere tabbladen verplaatst als geheel: al zijn tabbladen groeperen onder het GitPoulpi-pictogram zolang GitPoulpi erin draait. Eerder gemaakte snelkoppelingen en vastgemaakte taakbalkitems worden door dezelfde synchronisatie herschreven; toont een vastgemaakt item nog het pictogram van de terminal, maak het dan los en maak het opnieuw vast. Linux: kitty, foot, Alacritty, WezTerm en Ghostty worden gestart met de vensterklasse `gitpoulpi` en het `.desktop`-bestand noemt die (`StartupWMClass`), zodat een dock het venster onder het pictogram van de starter groepeert; andere terminals houden hun eigen pictogram. macOS: niets te doen, het Dock groepeert per app.

![Instellingen > Algemeen, gescrold naar de groep Starter: de terminal voor snelkoppelingen en de rij van de app-starter](../images/settings-launcher.png)

##### Opdrachtalias en PATH, per systeem

| Systeem | Alias | PATH |
|---|---|---|
| Linux, macOS | Een symbolische koppeling `~/.local/bin/<naam>` naar `gitpoulpi-tui`. Een koppeling kan geen markering bevatten, dus ze geldt als van ons als haar doel een bestand met de naam `gitpoulpi-tui` is; een gewoon bestand, of een koppeling naar iets anders, is **niet van ons** en wordt nooit vervangen of verwijderd. | Een blok tussen `# >>> GitPoulpi PATH (X-GitPoulpi=1) >>>` en `# <<< GitPoulpi PATH <<<` in `~/.bashrc` en `~/.zshrc` (welke er ook bestaan; `~/.zshrc` op macOS, anders `~/.bashrc` als geen van beide bestaat), en `~/.config/fish/conf.d/gitpoulpi-path.fish` als fish is ingesteld. Verwijderen haalt het blok (of het fish-bestand) weg en niets anders. |
| Windows | `%LOCALAPPDATA%\Programs\GitPoulpi\bin\<naam>.cmd`, een shim van twee regels met `REM X-GitPoulpi=1`. | De gebruikers-PATH in `HKCU\Environment`, geschreven via PowerShell, gevolgd door een `WM_SETTINGCHANGE`-broadcast zodat nieuwe programma's hem zien. De waarde `GitPoulpiPathAdded` legt vast wat er is toegevoegd, en Verwijderen haalt alleen dat weg. |

Nieuwe terminals zien de PATH-wijziging, de al geopende niet. Geen van beide acties staat in het journaal: er is geen Ongedaan maken voor, **Verwijderen** is de weg terug.

Het palet heeft dezelfde acties (**Starter: Snelkoppeling maken**, **Starter:
Bureaublad-snelkoppeling maken**, **Starter: Snelkoppelingen verwijderen**, **Starter: Opdrachtalias maken**, **Starter: Opdrachtalias verwijderen**, **Starter: Toevoegen aan PATH**, **Starter: Verwijderen uit PATH**), en
`gitpoulpi desktop shortcut status|install|remove [--where launcher|desktop|startmenu]`
doet het vanuit een script.

De opdracht **App: Taal wijzigen...** van het palet (`Ctrl+N g`) opent dezelfde talenlijst. Daar toont
de gemarkeerde taal live een voorbeeld; `Enter` houdt haar en `Esc` herstelt de vorige.

### Synchronisatie

Hoe GitPoulpi een repository gelijk houdt met zijn remotes, en wat het ondertussen met je
lokale wijzigingen doet.

![De pagina Synchronisatie: het interval voor automatisch ophalen als getal en eenheid (minuten) op één regel, daaronder de overige opties](../images/settings-quantity.png)

| Optie | Wat ze doet |
|---|---|
| **Auto-fetch-interval** | Haalt elke remote op de achtergrond op met dit interval, een getal en een eenheid (minuten of uren) op één regel. Standaard 5 minuten, maximaal 60 minuten (1 uur), `0` zet het uit. `general.auto_fetch_interval_minutes` blijft in minuten; `general.auto_fetch_interval_unit` legt alleen vast hoe het wordt getoond. Vereist een herstart. |
| **Fetch bij het openen van een tabblad** | Voert Fetch All op de achtergrond uit als een repositorytabblad opent. Standaard aan. Een repository die in de laatste 60 seconden is opgehaald wordt niet opnieuw opgehaald. |
| **Auto-prune** | Fetches verwijderen ook remote-tracking branches die niet meer op de remote bestaan. Standaard uit. |
| **Tags automatisch opschonen** | Fetches verwijderen ook lokale tags die op de remote zijn verwijderd. Alleen tags die op de remote stonden worden aangeraakt: een tag die je nooit hebt gepusht blijft staan. Standaard uit, net als in git. Vereist git 2.41 of nieuwer. CLI: `gitpoulpi fetch --prune-tags`; configuratiesleutel `general.auto_prune_tags`. Zie [Branches, tags en de zijbalk Refs](log-and-history.md#branches-tags-en-de-zijbalk-refs). |
| **Submodules up-to-date houden** | Werkt na een checkout of een pull de submodules bij naar de commits die de repository vastlegt. Zie [Submodules](submodules.md). |
| **".orig"-bestanden na het mergen verwijderen** | Verwijdert nadat een merge is voltooid de back-upbestanden `*.orig`. Standaard uit. |
| **Auto-stash voor checkout, pull, merge of rebase** | Wat te doen als lokale wijzigingen een van deze bewerkingen blokkeren: **ask** (een dialoogvenster biedt aan de wijzigingen te stashen, de bewerking uit te voeren en de stash te herstellen), **always** (zonder vragen doen) of **never** (weigeren). |
| **Niet-gevolgde bestanden opnemen in auto-stash** | De automatische stash neemt ook niet-gevolgde bestanden mee. |

Ook op deze pagina, onder **Updates**:

| Optie | Wat ze doet |
|---|---|
| **Op updates controleren** | Een keuzelijst (`updates.mode`): **Melden** (standaard) controleert eenmaal per dag op de achtergrond en toont `vX.Y.Z beschikbaar` in de titelbalk; **Auto** downloadt de update ook op de achtergrond als GitPoulpi uit een releasearchief is geïnstalleerd (jij kiest nog steeds wanneer je herstart); **Uit** doet nooit een verzoek. Geldt direct. Zie [Updates](installation.md#updates). |
| **Updatekanaal** | Een keuzelijst (`updates.channel`): **Stabiel** (standaard) of **Pre-release** (ook release candidates). Geldt direct. |

Ook op deze pagina, onder **Geschiedenis herschrijven**, de tijdslimieten van externe scripts (zie [Geschiedenis herschrijven](history-rewrite.md)):

| Optie | Wat ze doet |
|---|---|
| **Tijdslimiet voor scriptantwoord** | Hoe lang een extern script krijgt om één commit te beantwoorden, een getal en een eenheid (milliseconden of seconden) op één regel. Standaard 5 seconden, 1 milliseconde tot 10 minuten (`rewrite.script_timeout_ms`, `rewrite.script_timeout_unit`). Bij een timeout wordt het hele proces gestopt en mislukt de herschrijving zonder dat er iets is verplaatst. |
| **Tijdslimiet per scriptronde** | Hoe lang één volledige doorloop van een extern script mag duren, een getal en een eenheid (seconden, minuten of uren). Standaard 1 uur, hooguit 24 uur (`rewrite.script_total_timeout_s`, `rewrite.script_total_timeout_unit`). |

### Profielen

![De pagina Profielen](../images/settings-profiles.png)

Profielen met commit-identiteit: de naam en het e-mailadres die voor commits vanuit GitPoulpi
worden gebruikt. Het tabblad **Algemeen** bevat de keuzelijst **Actief profiel**; het tabblad
**Profielen** is een [lijst](#lijsten) (toevoegen, dupliceren, verwijderen, herordenen) met de
velden van het geselecteerde profiel. Elk
profiel heeft een **Profiel-ID**, een **Profielnaam**, een **Naam van auteur** en een
**E-mail van auteur**. Een profiel zonder e-mailadres toont `⚠` in de lijst. Zonder actief profiel, of met een
profiel met een lege naam of e-mailadres, gelden de eigen `user.name` en `user.email` van de repository.

Met twee of meer profielen toont de werkbalk aan zijn rechteruiteinde een profielkeuzelijst.
Die stelt het profiel alleen voor de actieve repository in. Na je keuze biedt een
dialoogvenster aan de identiteit ook in de configuratie van die repository te schrijven. De
opdracht in het palet is **Repository: Profiel wijzigen...**.

Deze profielen zijn niet de hostingprofielen van de pagina Integraties, noch de AI-profielen
van de pagina AI.

### SSH & inloggegevens

![De pagina SSH & inloggegevens](../images/settings-ssh.png)

SSH gebruikt je systeem-`ssh`, zodat je ssh-agent en je ssh-configuratie blijven werken.
GitPoulpi leest nooit sleutelmateriaal.

| Optie | Wat ze doet |
|---|---|
| **Pad van privésleutel** | Indien ingesteld voeren fetch, pull, push en clone git uit met `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`, zodat ssh alleen die sleutel aanbiedt. Leeg: de agent en je ssh-configuratie gelden ongewijzigd. |
| **Pad van publieke sleutel** | Alleen gebruikt als het pad van de privésleutel leeg is: ssh kiest de bijpassende sleutel die ssh-agent bewaart (of een hardwaresleutel). |
| **Sleuteltype** | Het type dat **Sleutelpaar genereren** maakt: `ed25519`, `rsa` (4096 bits) of `ecdsa` (NIST P-256). |
| **HTTPS-inloggegevens** | Hoe fetch, pull, push en clone zich via HTTPS authenticeren: **credential_helper** (de eigen helper van git, de standaard), **prompt** (helpers negeren; git vraagt in een terminal) of **none** (geen inloggegevens, alleen publieke remotes). |
| **Alle gebruikersnamen en wachtwoorden vergeten** | Verwijdert elk token dat GitPoulpi in je credential helper heeft opgeslagen. Vraagt eerst om bevestiging. |
| **Sleutelpaar genereren** | Maakt een sleutelpaar van het **Sleuteltype** in `~/.ssh/gitpoulpi_<type>`, zonder wachtwoordzin, en legt de twee paden vast. Overschrijft nooit een bestand. |

De sleutelpaden gelden voor de volgende overdracht. Een voorvoegsel `~/` betekent je thuismap.

Een elders gemaakte instelling wint altijd van deze paden. De volgorde, hoogste eerst:

1. `GIT_SSH_COMMAND` of `GIT_SSH` in de omgeving waarin GitPoulpi draait.
2. `core.sshCommand` in de git-configuratie van de repository of je globale git-configuratie. Het item
   *SSH-sleutel...* van het tabbladmenu schrijft dit voor één repository.
3. De twee sleutelpaden van deze pagina.
4. De eigen standaardwaarden van ssh en de agent.

Een sleutel kan ook bij een hostingaccount horen; zie
[Hosting en issuetrackers](hosting-integration.md#ssh-sleutel-per-profiel).

### Integraties

![De pagina Integraties: links de providers en de knop Verbinden voor GitHub](../images/settings-integrations.png)

Verbindt je accounts. De groep **Hosting** heeft GitHub, GitHub Enterprise, GitLab, GitLab
Self-Managed, Azure DevOps, Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea, Forgejo en Gerrit. De groep
**Issuetrackers** heeft Jira Cloud, Jira Data Center, Trello, Linear, YouTrack, Redmine en Local Markdown. Elke provider heeft
**Verbinden**, een tokenveld, de scopes die het token nodig heeft en een link naar de
tokenpagina van de host. Deze pagina heeft een eigen gids:
[Hosting en issuetrackers](hosting-integration.md).

Twee opties staan boven de providers. Druk op `Up` bij de eerste provider om ze te bereiken.

| Optie | Wat ze doet |
|---|---|
| **Vernieuwing pull requests & issues** | Hoe vaak de zijbalklijsten en de werkbalkchip worden vernieuwd, een getal en een eenheid (minuten of uren). Standaard 5 minuten, hooguit 2 uur, `0` zet de timer uit (`general.hosting_refresh_minutes`, `general.hosting_refresh_unit`). |
| **Terminologie voor reviewverzoeken** | Het woord voor gehoste reviews: **Auto (per host)**, **Pull request** of **Merge request**. |
| **De pull request-chip verbergen als hij gesloten of gemerged is** | Standaard aan (`ui.hide_finished_pr_chip`; op GitLab heet het "merge request"). De chip in de werkbalk toont het laatste verzoek van de uitgecheckte branch; zodra dat verzoek gesloten of gemerged is, verdwijnt hij en toont de gedimde chip **+ PR** (een nieuw verzoek starten), alsof de branch er geen had. Uit: de chip toont altijd het laatste verzoek, in welke status het ook staat. Geldt meteen. |

### AI

![De pagina AI: AI inschakelen, globale instructies en de blokken per actie](../images/settings-ai.png)

Op het tabblad **Algemeen**: de schakelaar die AI aanzet, het standaardprofiel, de instructies die
elke actie krijgt, en een profiel en instructies voor elke actie. Het tabblad
**Profielen (verbindingen)** is een [lijst](#lijsten) van de AI-profielen met een inline tabel voor de
omgevingsvariabelen van een profiel. Het blok *Codereview* van het tabblad Algemeen bevat ook de
standaardwaarden van de [reviewcontext](ai.md#reviewcontext): **De agent in de repository laten draaien**
(*Profielinstelling*, *Ja*, *Nee*), **CLAUDE.md meegeven**, **AGENTS.md meegeven** en **Extra bestand** (een pad of een
link; leeg voegt er geen toe, met een knop **Bladeren...** eronder die een bestandskiezer opent; `Enter` op de knop is
het toetsenbordequivalent). Deze pagina heeft een eigen gids: [AI-ondersteuning](ai.md).

### Agents & MCP

Wat AI-agents in dit venster mogen doen, en of de MCP-server mag draaien. De twee
MCP-rijen verschijnen alleen in builds die de server bevatten; zie
[MCP en agents](mcp-and-agents.md).

| Optie | Wat ze doet |
|---|---|
| **MCP-server** | Aan (standaard): een agent kan de server starten (`gitpoulpi mcp serve`). Uit: hij weigert te starten. Opgeslagen als `mcp.enabled`. Een wijziging geldt de volgende keer dat een agent de server start; een server die al draait houdt zijn instellingen. |
| **Poort van MCP-server** | De poort waarop `--http` luistert als er geen adres is opgegeven, 1 tot 65535 (standaard 8808). Opgeslagen in de poort van `mcp.bind`; het hostdeel blijft behouden (`127.0.0.1` als het leeg is). Geldt de volgende keer dat een agent de server start. |
| **AI-agents de weergave laten besturen** | Laat agents je een commit, een bestand of een melding in dit venster tonen (`ui show`). Uit: geen enkele agent bereikt het venster. |
| **AI-agents wijzigingen laten voorstellen die ik bevestig** | Laat agents vooraf ingevulde dialoogvensters voor Reword, commit, branch of tag openen die jij bevestigt. Standaard uit. Zie [MCP en agents](mcp-and-agents.md#rechten). |
| **Een venster voor agents openen**, **Startopdracht (custom)** | Of een agent een GitPoulpi-venster mag openen als er geen open is, en met welke terminal of multiplexer. Standaard uit. |

### Weergave

![De pagina Weergave: thema, avatars, invoerkleur, kleurdiepte en grafiekkleuren (de datumformaten volgen hieronder)](../images/settings-ui.png)

![De keuzelijst Thema open op Weergave met Licht gemarkeerd: het hele scherm Instellingen verft al in dat thema](../images/settings-appearance-theme.png)

| Optie | Wat ze doet |
|---|---|
| **Thema** | **Systeem (volgt de terminal)**, een ingebouwd thema of een van je eigen themabestanden. Het gemarkeerde thema toont live een voorbeeld: `Enter` houdt het, `Esc` herstelt het vorige. De strook **Kleuren** onder de rij toont de kleuren die gelden. Zie [Thema's](themes.md). |
| **Avatars van auteurs** | **uit** (standaard, geen netwerk), **grafiek** (in de commit-stip) of **auteur** (in de kolom Auteur). Aan wordt een hash van elk e-mailadres van een auteur naar Gravatar en naar je hostingprovider gestuurd. |
| **Avatarbronnen**, **Avatar-terugval** | Waar afbeeldingen worden opgezocht en in welke volgorde. Voor auteurs zonder afbeelding: **identicon** (een gegenereerd patroon) of **initialen**. |
| **Invoerkleur**, **Aangepaste invoerkleur** | De achtergrond van tekstvelden: **Thema volgen** (standaard), een vaste donkere kleur of je eigen `#rrggbb`. |
| **Kleurdiepte** | **auto** (standaard), **truecolor**, **256** of **16**. Zie [Thema's](themes.md). |
| **Kleurthema van de grafiek** | **Thema volgen** (standaard: de eigen baankleuren van het thema) of een willekeurig baanpalet. Elk palet toont enkele van zijn kleuren. Geldt als de Instellingen sluiten. De opdracht **App: Kleurthema van de grafiek wijzigen...** van het palet (`Ctrl+N Shift+Y`) toont elk palet live vooraf. |
| **Achtergrond van grafiekbanen** | **Uit** (transparant) of **Rijtint**: een zwakke band in de baankleur achter de grafiekkolom, met een dunne balk aan de rand ervan. |
| **Sterkte van rijtint** | Hoe vervaagd de band van Rijtint is: **Sterk**, **Gemiddeld** (standaard) of **Subtiel**. De balk aan de rand behoudt zijn volle kleur. |
| **Positie van de werkbalk** | `ui.toolbar_position`: Waar de werkbalk staat: Boven, Onder (standaard), Links of Rechts. Links en Rechts maken er een kolom naast de panelen van; in een venster smaller dan 80 kolommen staat hij onderaan. Werkt direct. |
| **Uitlijning van de werkbalkknoppen** | `ui.toolbar_align`: Waar de knoppen langs de balk staan: Begin (standaard), Midden of Einde. Begin is links in een rij en boven in een kolom; Einde is rechts en onder. Werkt direct. |
| **Zet de pull request-knop vast aan het einde** | `ui.toolbar_pr_pinned`: Aan (standaard): de pull request-knop staat helemaal aan het einde van de werkbalk (rechts, in een kolom onderaan), wat de uitlijning ook is. Uit: hij stroomt mee met de andere knoppen en wordt met hen uitgelijnd. Werkt direct. |
| **Positie van de statusbalk** | `ui.statusbar_position`: Waar de statusbalk (branch, tellers, meldingen, toetstips) staat: Boven, boven de tabbladen van de repository, of Onder (standaard). Het tabblad Start houdt zijn eigen statusbalk onderaan. Werkt direct. |
| **Positie van de tabbalk** | `ui.tabs_position`: Waar de repositorytabbladen staan: Boven (standaard), Onder of Links, een lijst met één tabblad per regel aan de linkerrand van het venster. In een venster smaller dan 80 kolommen staan ze bovenaan. Naast een werkbalk als kolom blijven de tabbladen aan de buitenkant. Werkt direct. |
| **Landinstelling voor datum/tijd** | **system** (standaard): maand- en dagnamen in de taal van de interface. **english**: altijd Engelse namen. |
| **Datumformaat**, **Datum-/tijdformaat**, **Uitgebreid datumformaat** | De datumpatronen van de datumkolom van Bestandsgeschiedenis en Blame, van de kolom Datum van de Log en van de commit-details. Elk toont een live voorbeeld. Zie de [configuratiereferentie](config-reference.md#ui) voor de tokens. |

![De keuzelijst Kleurthema van de grafiek: Thema volgen, daarna elk baanpalet met enkele van zijn kleuren](../images/settings-log-graph-lanes.png)

### Log & grafiek

Hoe de commit-grafiek wordt getekend en hoeveel commits de Log laadt. De grafiekkleuren staan
op de pagina **Weergave**.

| Optie | Wat ze doet |
|---|---|
| **Grafiekweergave** | **auto** tekent een afbeelding waar de terminal dat ondersteunt, en tekens elders. **image** forceert de afbeelding, **text** forceert tekens. Vereist een herstart. |
| **Afbeeldingsprotocol** | Hoe afbeeldingen de terminal bereiken: **auto**, **kitty**, **iterm2**, **sixel** of **none**. Vereist een herstart. |
| **Stashes in de grafiek tonen** | Een rij voor elke stash in de Log. Uit: stashes verschijnen alleen in de zijbalk. |
| **Spookbranch tonen bij de aangewezen commit** | Standaard aan (`ui.graph_ghost_ref`). In de kolom Ref toont de commit onder de aanwijzer die geen eigen ref heeft een vervaagde pil met de naam van de branch waartoe die behoort (anders de remote branch), afgeleid uit de geschiedenis, waar de kolom ruimte heeft. Een branch die na een merge is verwijderd, wordt toch getoond, doorgestreept, onder de naam uit het onderwerp van de merge-commit (`Merge branch 'x'`, `Merge pull request #N from owner/x`, ...); squash-merges, rebases en fast-forwards laten zo'n spoor niet na. Tags worden nooit als spook getekend. Wijs je de pil aan, dan zie je het soort. Klikken doet niets bijzonders; het palet-commando `Git: Branch maken op geselecteerde commit...` maakt een branch. |
| **Verbinding ref-naar-commit** | Een dunne lijn van het label van een ref naar zijn commit-stip. |
| **Kolombreedte aanpassen** | `ui.column_resize`, meteen toegepast op de Log, de Radar en de commitkolom van de diff-, bestands- en blame-weergaven. **line** (de standaard): elke kolomgrens is een tussenruimte van één cel met een klein streepje in de kop, een dunne accentlijn wordt over de hele tabel getekend zolang de aanwijzer op de grens staat, en slepen overal langs die lijn schaalt de kolom. **header**: geen streepje, het eerste teken van de kopcel na de grens licht op en is de enige plek om vanaf te slepen. De toetsen om te schalen (`Shift+C` in de Log, `C` in de Radar, `Tab` in de commitkolom) doen in beide hetzelfde. |
| **WIP-aantallen in het bericht tonen als de kolom Wijzigingen verborgen is** | Toont de aantallen wijzigingen (`✎ 3  + 2  − 1`) in het bericht van de WIP-rij. Is de kolom Wijzigingen zichtbaar, dan staan de aantallen daar. |
| **Boom van branchmappen** | Groepeert branches en tags op `/` in mappen in de zijbalk. `y` in het paneel Refs schakelt dit om. |
| **Secties van het linkerpaneel** | Voor wie de getoonde en verborgen secties van het linkerpaneel gelden: **per repository** (de standaard; elke repository houdt een eigen set, de globale is het startpunt) of **globaal** (één set voor alle repositories). Heen en weer wisselen behoudt beide sets. Zie [muis en gebaren](mouse-and-gestures.md). (`ui.sidebar_sections_scope`) |
| **Klik op een ref om die in de grafiek te tonen** | Eén klik op een ref in het Refs-paneel scrolt de Log naar waar de ref naartoe wijst en laat de rij even oplichten (de commit-selectie en het rechterpaneel blijven zoals ze zijn). Uit: een klik selecteert alleen de rij in de zijbalk. **In grafiek tonen** werkt in beide gevallen. |
| **Composervoorstellen** | Of de AI-commit-composer opent met de bestanden van elk voorstel **uitgeklapt** of **ingeklapt**. |
| **Eerste commits in de grafiek** | Hoeveel commits de Log laadt als een repository opent. Standaard 5000, minimaal 500. Hogere waarden vertragen grote repositories. |
| **Commits geleidelijk laden in de grafiek** | Aan: er worden meer commits op de achtergrond geladen als je naar de oudste geladen commit scrolt. Uit: de Log stopt bij de eerste commits. |
| **Commits per laadbeurt** | Hoeveel commits één achtergrondlading ophaalt. |
| **Maximum aantal commits in de grafiek** | Geleidelijk laden stopt bij dit aantal. `0` betekent geen limiet. Elke commit gebruikt ongeveer 300 bytes geheugen. |
| **Totaal aantal commits tellen met** | Hoe de statusbalk het totaal in `geladen / totaal` te weten komt (`log.commit_count`, een keuzelijst, geldt direct): **System git** voert één proces `git rev-list --count` uit (standaard, normaal het snelst); **libgit2** telt binnen GitPoulpi en start geen proces, voor computers waar antivirussoftware elke opdracht scant en elke git-aanroep traag maakt; **Uit** telt nooit en toont alleen de geladen commits. |
| **Auteurs tonen via .mailmap** | De Log, blame en de bestandsgeschiedenis tonen elke auteur zoals de `.mailmap` van de repository hem noemt (`log.use_mailmap`, een schakelaar, standaard aan, geldt direct). De opgeslagen commits worden nooit gewijzigd. Uit toont de namen zoals gecommit. Zie `history-rewrite.md`, *Het tabblad Identiteiten*. |

### Diff & editor

![De pagina Diff & editor](../images/settings-editor.png)

| Optie | Wat ze doet |
|---|---|
| **Tabbreedte** | Kolommen per tabstop, 1 tot 8 (standaard 4). |
| **Regeleinde** | Wat de editor bij het opslaan schrijft: **Auto (die van het bestand)**, **LF** of **CRLF**. |
| **Syntaxiskleuring** | Taalkleuren in de editor. |
| **Regelnummers** | De kolom met regelnummers van de editor en de diff. |
| **De selectie volgen in de editor** | Zolang de editor open is, opent het selecteren van een ander bestand in het rechterpaneel dat ook in de editor, nadat is gevraagd of niet-opgeslagen wijzigingen moeten worden opgeslagen. Een bestand dat niet bewerkt kan worden, opent in de gebruikelijke weergave. Alleen voor de werkwijzigingen: een bestand dat uit een commit is gekozen verlaat de editor voor de diff van die commit. |
| **Regelafbreking** | Breekt lange diffregels af. |
| **Standaardlay-out**, **Alleen hunks**, **Alleen-witruimtewijzigingen tonen**, **Minimap** | De beginstaat van de diffweergave. De weergavebalk van de diff wijzigt en bewaart ze. |
| **Blame-annotaties**, **Breedte van blame-annotaties** | Aan welke kant van de Blame-weergave de annotatiekolom staat (avatar, auteur, leeftijd, samenvatting): **right** (standaard) of **left**, en de breedte in cellen (24 tot 90). Als je in de Blame-weergave de rand van de kolom sleept, of de toetsen `>` / `<` gebruikt, wordt de breedte hier opgeslagen (`ui.blame_side`, `ui.blame_width`). Zie [Blame](log-and-history.md#blame). |
| **Merge-editor: naast elkaar vanaf (kolommen)** | Vanaf deze breedte toont de merge-editor A en B naast elkaar. Standaard 100, minimaal 40, `0` betekent altijd. Daaronder zijn A en B tabbladen. Zie [Merge-conflicten oplossen](merge-conflicts.md). |
| **Externe editor** | De opdracht voor het item *Openen in editor* van een bestand. `{file}` is het pad. Leeg: VS Code indien geïnstalleerd, anders de systeemstandaard. |
| **Externe diff-tool** | De opdracht voor *Openen in externe diff-tool*. `{local}` en `{remote}` zijn de twee versies. Leeg: `git difftool`. |
| **Externe merge-tool** | De opdracht voor *Openen in externe merge-tool* op een conflictkaart. `{base}`, `{local}`, `{remote}` en `{merged}` zijn de bestanden. Leeg: `git merge-tool`. |
| **Bestandsbeheer** | De opdracht voor *Tonen in map*. `{path}` is het bestand, `{dir}` is de map ervan. |

### Openen in

![De pagina Openen in, tabblad Items: de lijst met items, het item dat wordt bewerkt met zijn label en opdracht, en de knop + Toevoegen](../images/settings-open-in.png)

Bewerkt de lijst van de knop **Openen in** in de titelbalk. Het tabblad **Algemeen** bevat de
knoppen **Afsluiten & cd hier**, het **Standaarditem** en **Gedetecteerde standaardwaarden herstellen**.
Het tabblad **Items** (elk een naam en een opdracht) is een [lijst](#lijsten): **+ Toevoegen ▾** biedt de voorinstellingen en
**Eigen**, en het standaarditem is gemarkeerd met `★`. De velden van het geselecteerde item bevatten een
knop **Testen**. Zie [Openen in](open-in.md).

De eerste rij, **De map van het actieve tabblad volgen**, heeft twee waarden:

- **buttons** (standaard): de tabbalk toont **Afsluiten & cd hier** als het actieve
  tabblad niet de map is waarin GitPoulpi is gestart.
- **auto**: geen knop. Elke afsluiting wijzigt de map van je shell naar het actieve tabblad,
  en een nieuw tabblad openen voert het standaarditem van Openen in uit.

Zie [De map van het actieve tabblad volgen](following-the-tab-folder.md).

### Afbeeldingen

Hoe afbeeldingsbestanden worden getoond, en welk programma **Bewerken in** gebruikt. Zie
[Afbeeldingen](image-view.md). Het tabblad **Algemeen** bevat de onderstaande weergaveopties, het
**Standaarditem** en **Gedetecteerde standaardwaarden herstellen**; het tabblad **Editors** is de lijst
met afbeeldingseditors. Een build zonder afbeeldingsvoorbeeld heeft geen weergaveopties.

| Optie | Wat ze doet |
|---|---|
| **Afbeeldingen voorvertonen** | Toont een afbeeldingsbestand als afbeelding in de Diff-weergave en de Bestandsweergave. Uit: de gewone binaire melding. Standaard aan. |
| **Zonder afbeeldingsprotocol** | Op een terminal zonder inline-afbeeldingsprotocol: **Halfblok-kleuren** (de standaard) of **Alleen infokaart**. |
| **Max. voorbeeldgrootte** | Grotere afbeeldingen worden niet gedecodeerd; in plaats daarvan verschijnt de infokaart. Een getal en MiB of GiB, standaard 32 MiB, hooguit 1 GiB (`ui.image_view_max_mib`, `ui.image_view_max_unit`). |
| **Pixelig zoomen vanaf** | `1x`, `2x` (standaard) of `never`: vanaf deze zoom toont de afbeelding vierkante pixels in plaats van een afgevlakte afbeelding. |
| **Zoom en scrollen voor/na synchroniseren** | De twee panelen van een afbeeldingsdiff delen één zoom en één scrollpositie. Standaard aan. |

Het tabblad **Editors** is een [lijst](#lijsten) zoals die van [Openen in](open-in.md):
per editor een **Naam**, een **Opdracht**, de variabele-chips `${file}` `${dir}`
`${name}` `${repo}` en **Testen met een voorbeeldafbeelding**; **+ Toevoegen ▾** biedt voorinstellingen en
**Eigen**, en **Gedetecteerde standaardwaarden herstellen** zet de lijst terug. Tot je haar bewerkt bevat de lijst de editors die op deze machine zijn
geïnstalleerd en de standaardapp van het systeem. Elke wijziging geldt meteen.

### Toetsenbord & muis

![De pagina Toetsenbord & muis: elke actie per context met zijn toetsen, een zoekvak en filters](../images/settings-keyboard.png)

**Instellingen > Toetsenbord & muis** heeft twee tabbladen (klik, of `Alt+Left` / `Alt+Right`).
**Algemeen** bevat het toetsenbordprotocol en de muisinstellingen; **Sneltoetsen**
toont elke sneltoets per context, met een zoekvak (`/`) en filters. Op een rij:

| Toets | Actie |
|---|---|
| `Enter` of dubbelklik | Legt een nieuwe toets vast. Conflicten en terminalwaarschuwingen verschijnen terwijl je drukt. |
| `a` | Voegt een alternatieve toets toe. |
| `u` | Maakt de binding van de actie los. |
| `r` | Zet de rij terug naar haar standaard. |
| `R` | Zet elke sneltoets terug, na een bevestiging. |
| `c` of een klik op de `!` van de rij | Toont de actie met alle acties waarmee haar toets botst; `Esc` toont weer alles. Een toets die botst met die van een andere actie is rood, een waarschuwing over alleen de toets (de terminal kan hem niet sturen) oranje. |

Gebruik dit om botsingen met tmux, AutoHotkey, herdr of je terminal op te lossen. Dezelfde
instellingen staan in de tabel `[keybindings]` van `config.toml`; zie de
[configuratiereferentie](config-reference.md#keybindings) en [Sneltoetsen](keybindings.md).

![De pagina Toetsenbord & muis na c op Quit: de toets q is rood bij Quit en bij Stage or Unstage the Selected Row, de twee acties die botsen](../images/settings-keyboard-conflicts.png)

Drie opties staan op het tabblad **Algemeen**.

| Optie | Wat ze doet |
|---|---|
| **Kitty-toetsenbordprotocol** | **auto** vraagt de terminal om het kitty-toetsenbordprotocol, zodat `Ctrl+,`, `Ctrl+Enter` en `Ctrl+Shift+N` aankomen. **off** vraagt nooit; gebruik het als een terminal zich misdraagt. Vereist een herstart. |
| **Muis ingeschakeld** | Uit laat de muis en het selecteren van tekst aan je terminal. Vereist een herstart. |
| **Regels per muiswielstap** | Hoeveel rijen één wielstap in de Log verplaatst. Vereist een herstart. |

### Repositoryspecifiek: Git & codering

![De pagina Git & codering voor deze repository](../images/settings-git.png)

| Optie | Wat ze doet |
|---|---|
| **Git-backend** | `git2` (libgit2, de standaard) of `system` (je geïnstalleerde `git`). Globaal. Geldt voor daemons die na de wijziging worden gestart. |
| **Pull-knop** | Wat de knop Pull doet: merge (fast-forward indien mogelijk), alleen fast-forward, of rebase. Globaal. De items `▾` wijzigen het niet. |
| **Push-knop** | **direct** pusht meteen; **ask** toont eerst het remotemenu. Globaal. |
| **Standaardcodering** | De codering waarmee tekst wordt getoond die **geen geldige UTF-8** is: bestandsinhoud, blame, diffs en commit-berichten. De pagina heeft een globale rij en een rij voor deze repository; de rij van de repository wint. Geldige UTF-8 wordt nooit opnieuw gedecodeerd en staging codeert nooit iets opnieuw. |
| **core.autocrlf**, **core.eol** | De eigen git-instellingen van deze repository. De rij schrijft `core.autocrlf` of `core.eol` in de git-configuratie van de repository. **(overgenomen)** verwijdert de waarde van de repository, zodat je globale git-configuratie geldt. `Ctrl+Z` herstelt de vorige waarde. Het CLI-equivalent is `gitpoulpi repo-config`. |
| **Git LFS** | Toont Git LFS in de interface. Zie [Git LFS](lfs.md). |
| **LFS-filters installeren vóór pull en push** | Voert vóór een pull of push `git lfs install --local` uit als de repository LFS gebruikt en de filters ontbreken. |
| **git-lfs-programma** | Het gedetecteerde programma `git-lfs` (alleen lezen). |
| **LFS initialiseren voor deze repository** | Voert nu `git lfs install --local` uit. |

### Repositoryspecifiek: Gedeeltelijke checkout

Dezelfde bediening als het dialoogvenster Gedeeltelijke checkout (`Repository: Gedeeltelijke checkout...`), als pagina van de
instellingen van deze repository (`gitpoulpi ui show settings sparse-checkout`).

| Optie | Wat ze doet |
|---|---|
| **Status** | Of deze werkmap een gedeeltelijke checkout is. |
| **Mappen om te behouden** | Eén map per regel, relatief aan de hoofdmap van de repository. `Enter` (of een klik) opent een editor met meerdere regels: `Ctrl+S` bewaart de tekst, `Esc` laat hem vallen. Zolang de gedeeltelijke checkout aan staat, toont het veld de huidige mappen. De cone-modus van git kent alleen mappen (geen patronen, geen losse bestanden), de bestanden in de hoofdmap worden altijd uitgecheckt en een lege lijst wordt geweigerd. |
| **Gedeeltelijke checkout starten** | Checkt alleen de genoemde mappen uit. Zodra de gedeeltelijke checkout aan staat, heet dezelfde knop **Maplijst toepassen** en past ze de bewerkte lijst toe: een map die uit de lijst is gehaald, wordt uit je werkmap verwijderd. Grijs zolang de lijst leeg is. |
| **Gedeeltelijke checkout stoppen** | Terug naar een volledige checkout: elk bestand komt terug (na een bevestiging). Grijs zolang de gedeeltelijke checkout uit staat. |
| **Gedeeltelijke checkout vernieuwen** | Past de huidige lijst opnieuw toe (bijvoorbeeld nadat een script of een ander programma bestanden daarbuiten heeft uitgecheckt). Ze stuurt de huidige mappen, niet de nog niet verstuurde tekst van het veld. Grijs zolang de gedeeltelijke checkout uit staat. |

De knoppen draaien als gewone achtergrondacties (de pagina toont "gedeeltelijke checkout wordt toegepast..." terwijl er een loopt). Net
als het dialoogvenster staan ze niet op de stapel van Ongedaan maken: er gaat niets verloren en **Uitschakelen** brengt elk bestand terug.

### Repositoryspecifiek: LFS

De LFS-status van deze repository (`gitpoulpi ui show settings lfs`; niet in een build zonder LFS).

- **Niet geïnitialiseerd:** "Git LFS is niet ingesteld in `<repository>`" en de knop **LFS hier instellen**,
  die `git lfs install --local` uitvoert (de filters en hooks van deze repository, nooit je globale configuratie; vereist
  `git-lfs`). Als de `.gitattributes` van de repository bestanden al via LFS leidt maar de hooks ontbreken, zegt de
  pagina dat.
- **Geïnitialiseerd:** "Git LFS is ingesteld in `<repository>`", en de patronen die in `.gitattributes` worden gevolgd
  (`filter=lfs`).

LFS geldt als **geïnitialiseerd** wanneer deze repository eigen LFS-hooks heeft (de hook `pre-push` of `post-checkout` die
`git lfs install --local` schrijft). Filters alleen tellen niet mee: `git lfs install` op gebruikersniveau zet ze voor elke
repository, waardoor elke repository geïnitialiseerd zou lijken.

De knop **`LFS ▾`** in de werkbalk volgt dezelfde status: hij verschijnt alleen in een repository waar LFS is geïnitialiseerd of
waar `.gitattributes` het al gebruikt, en blijft verborgen in een gewone repository (en tot de eerste LFS-lezing van een
repository binnen is, dus hij kan een ogenblik na het openen van de repository verschijnen). Initialiseren vanaf deze pagina laat hem
verschijnen. De `LFS:`-opdrachten in het palet blijven overal beschikbaar, **LFS: Initialiseren voor deze repository** inbegrepen.

### Repositoryspecifiek: Commit

Hoe een commit in deze repository zich gedraagt (`gitpoulpi ui show settings commit`). Beide schakelaars staan standaard **uit** en worden alleen voor deze repository bewaard (`[preferences]` in
[`repo-state.toml`](config-reference.md#instellingen-per-repository-gitpoulpirepo-statetoml)).

| Instelling | Wat ze doet |
|---|---|
| **Push bij elke commit** | Pusht na een geslaagde commit de uitgecheckte branch naar zijn upstream, als de gewone push (een gevolgde overdracht, met de tekst "committen, daarna pushen..." terwijl ze loopt). Een branch zonder upstream wordt gecommit maar niet gepusht, en de melding zegt dat: de eerste push van een nieuwe branch blijft aan de knop **Push**, die de upstream instelt. Een geweigerde push opent de gewone kaart voor een geweigerde push; de commit blijft. |
| **Git-hooks standaard omzeilen bij elke commit** | Het selectievakje **Hooks overslaan** van het commitformulier (naast **Vorige commit amenden**; `Shift+H` in het Statuspaneel, of **Commit: Hooks overslaan omschakelen** in het palet) staat dan standaard aan, dus de hooks `pre-commit` en `commit-msg` draaien niet (`git commit --no-verify`). Het blijft een keuze per commit: na een commit keert het terug naar deze standaard. |

Zonder de standaard draaien de commithooks van een repository bij elke commit, welke backend GitPoulpi ook gebruikt (een repository
met commithooks wordt via de systeem-`git` gecommit, omdat libgit2 er geen draait). Een hook die mislukt, toont zijn
uitvoer in zijn item van het activiteitenlogboek (de statusbalk zegt dat de commit mislukte) en er wordt niets gecommit.

### Repositoryspecifiek: Merge-opties

Eén schakelaar, **Squash** (standaard uit). Aan: bij het mergen van een lokale branch (het menu van de branch, slepen en neerzetten, `Merge`
in het palet) komen de wijzigingen van de branch in de index zonder dat er een merge-commit wordt gemaakt (`git merge --squash`): het committen is aan
jou. Het commitformulier opent met het squashbericht van git (`SQUASH_MSG`, zonder commentaarregels), eenmaal per
squash. Daarna loopt er niets (geen `MERGE_HEAD`), dus **Afbreken** is niet van toepassing: om terug te gaan, verwerp je de
gestagede wijzigingen. Een squash die op conflicten stopt, laat ze in het Statuspaneel staan zoals elke merge met conflicten.
De gestagede wijzigingen zijn geen ongedaanmaak-item (er verplaatste geen ref, dus er is niets te herstellen); de CLI-opdracht
`gitpoulpi merge <branch> --squash` doet hetzelfde, wat de instelling ook is.

### Repositoryspecifiek: Berichtsjabloon

Een bericht waarmee het commitformulier begint (`gitpoulpi ui show settings commit-template`).

- **Een leeg commit-paneel vooraf invullen met het sjabloon**: een *leeg* commitformulier wordt vooraf ingevuld met het sjabloon, eenmaal per commit
  (het met de hand leegmaken wordt gerespecteerd tot de volgende commit).
- **Commentaarregels weglaten bij het maken van een commit**: bij het maken van de commit worden regels die met `#` beginnen (of met het
  `core.commentChar` van de repository) uit het bericht gehaald, zoals `git commit --cleanup=strip` zou doen. GitPoulpi commit met
  `-m`, dat git niet opschoont, dus het doet dit zelf. Met de schakelaar uit blijft een `#`-regel in de commit staan.
- **Samenvatting** (het label telt de 72 kolommen die nog over zijn, en hoeveel er te veel zijn) en een **Beschrijving** met meerdere regels
  (`Enter` of een klik opent de editor, `Ctrl+S` bewaart).
- **Sjabloon opslaan** schrijft het sjabloon naar `commit-template.txt` in de gegevensmap van de repository (zie
  `session.per_repo_state_dir`) en stelt het `commit.template` van deze repository in op dat bestand, zodat `git commit` op de
  opdrachtregel en andere tools het ook gebruiken. Als `commit.template` al naar een ander bestand wijst, toont de pagina het pad van dat
  bestand, bewerkt ze *dat bestand* en laat ze de instelling met rust.

### Repositoryspecifiek: Conflictpreventie

`gitpoulpi ui show settings conflict-prevention`. De pagina van de proactieve conflictdetectie van deze repository
(zie [Merge-conflicten oplossen](merge-conflicts.md#conflictpreventie)):

- **Proactieve conflictdetectie inschakelen met doelbranch**: standaard aan. Uitvinken schakelt de detectie alleen voor deze
  repository uit; de globale schakelaar is **Proactieve conflictdetectie inschakelen** op de pagina Algemeen.
  Beide moeten aan staan.
- **Basisbranches** (standaard `**`): de branches waarvan de huidige branch kan zijn gestart, één patroon per regel
  (`Enter` of een klik opent de editor, `Ctrl+S` bewaart). De patroontabel eronder is de naslag:

  | Patroon | Komt overeen met |
  |---|---|
  | `origin/main` | alleen die branch |
  | `origin/**` | elke branch van de remote `origin` |
  | `origin/feature/**` | de branches van `origin` die met `feature/` beginnen |
  | `!origin/abc` | sluit die branch uit |

  Een lokale branch is zijn kale naam (`main`), een remote-tracking branch heeft zijn remote ervoor (`origin/main`). `*` en `?`
  blijven binnen één padsegment, `**` loopt eroverheen. Lege regels en `#`-opmerkingen worden genegeerd; een lijst met alleen
  uitsluitingen betekent "elke branch behalve deze". Het wordt bewaard in `repo-state.toml` en opnieuw gelezen zodra Instellingen sluit.

### Repositoryspecifiek: AI

![De AI-pagina van de repository: repositoryinstructies, met Aan globale toevoegen of Globale vervangen, en een profiel en instructies voor elke actie](../images/settings-repo-ai.png)

Het AI-profiel en de instructies van deze repository voor elke actie, en de
*Repository-instructies* die op elke actie in deze repository van toepassing zijn. Lege velden
gebruiken de globale AI-instellingen. Zie [AI-ondersteuning](ai.md#instructies). Het blok *Codereview* herhaalt de
vier rijen van de reviewcontext: elke keuze begint op *Globaal gebruiken (waarde)* en toont de globale waarde, en een leeg
veld **Extra bestand** toont de globale grijs (en heeft dezelfde knop **Bladeren...**; de kiezer begint in de
wortel van de repository en een bestand binnen de repository wordt er relatief aan opgeslagen); de schakelaar **Geen extra bestand** zet het
globale extra bestand uit voor deze repository, terwijl een leeg veld het blijft volgen; **Terugzetten naar globaal** (getoond zodra de repository eigen waarden heeft)
zet ze allemaal terug. Ze worden bewaard in de `repo-state.toml` van deze repository.

### Repositoryspecifiek: Workflow

`gitpoulpi ui show settings workflow`. De branching-workflow van deze repository (sjabloon, detectie, basisbranch) heeft een eigen gids: [Branching-workflows](branching-workflows.md#instellingen--workflow). Het tabblad **Algemeen**
bevat wat geen regel is (sjabloon, detectie, hooks, issueflow, waar het wordt opgeslagen); het tabblad **Regels** is een
[lijst](#lijsten) in groepen (**Basisbranches**, **Branchtypen**, **Afrondregels**); **+ Toevoegen ▾**
kiest de soort en items verplaatsen binnen hun groep. Anders dan de andere pagina's houdt het een concept bij (de balk staat op beide tabbladen): een
balk onderaan de pagina toont **● Niet opgeslagen** (of **Geen wijzigingen**) met **Opslaan in het gedeelde
bestand**, **Alleen voor mij** en **Wijzigingen verwerpen**, en bij het sluiten van de Instellingen met een
niet-opgeslagen concept wordt eerst om bevestiging gevraagd.

## Bestanden, omgevingsvariabelen en de CLI

- Het instellingenbestand is `config.toml` in de configuratiemap van je besturingssysteem.
  `gitpoulpi config path` toont zijn locatie.
- De status per repository staat in `.gitpoulpi/repo-state.toml` in de repository.
- `gitpoulpi config get <key>` en `gitpoulpi config set <key> <value>` lezen en schrijven
  elke instelling.
- Omgevingsvariabelen en opdrachtregelvlaggen overschrijven het bestand.

De [configuratiereferentie](config-reference.md) geeft de details.

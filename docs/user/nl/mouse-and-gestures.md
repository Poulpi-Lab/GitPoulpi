[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · **Nederlands** · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Muis en gebaren

Alles wat je met de muis kunt doen heeft ook een toetsenbordequivalent. Zie
[Sneltoetsen](keybindings.md). De namen van de gebieden staan in de
[interfacerondleiding](interface-tour.md).

## Algemene regels

- **Klik** op een rij om haar te selecteren. Dit werkt in elk paneel, elke lijst, elk menu
  en elk dialoogvenster.
- **Dubbelklik** op een rij om haar te openen of uit te voeren. Een dubbelklik op een
  lokale branch, een tag of een refpil checkt hem uit. Een dubbelklik op een
  repositoryrij opent de repository.
- **Rechtsklik** op een rij voor haar menu. `.`, `Shift+F10` of de Menu-toets opent
  hetzelfde menu voor de geselecteerde rij.
- **Het wiel** scrolt het gebied onder de aanwijzer. In een lijst verplaatst het alleen
  het zicht. De selectie blijft waar ze is, ook als ze uit beeld scrolt. Alleen een klik
  of een toets verplaatst de selectie.
- **Shift+wiel** scrolt zijwaarts, in de diff, de Bestandsweergave en de editor als
  regelafbreking uit staat.
- **Aanwijzen** van een knop, een rij of een schuifbalkgreep laat die oplichten. Veel
  terminals kunnen de muisaanwijzer niet veranderen, dus dit oplichten is vaak het enige
  teken dat iets klikbaar is. Laat de aanwijzer op een knop of een Log-cel rusten voor een
  tooltip.
- **Klik buiten** een tekstveld om het te verlaten. Wat je typte blijft bewaard. Het
  reffilter past toe wat je typte, zoals `Enter` doet. De zoekbalk van de Log sluit,
  zoals `Esc` doet.

## Vensters en dialogen sluiten

Elk venster met een kader en elke dialoog heeft rechtsboven in de rand een `✕`, en overal gelden dezelfde regels:

- **Klik op `✕`** om te sluiten. `Esc` doet hetzelfde en sluit eerst het binnenste ding (een open keuzelijst, een tekstveld, een bevestiging) en dan pas het venster.
- **Klik buiten** een dialoog, een hulp- of infopaneel of een pop-up om die te sluiten. Een klik buiten sluit nooit een volledige weergave (Instellingen, de Diff-weergave, de Bestandsweergave, de editors, Review, Board): die hebben geen buitenkant.
- Een klik **binnen** een paneel sluit het nooit.
- Een formulier met tekst die je hebt getypt (een nieuw issue, een pull of merge request) negeert een klik buiten, zodat een verdwaalde klik niets kwijtraakt. `✕` en `Esc` vragen eerst **Wijzigingen verwerpen?**; met **Verder bewerken** ga je terug. Een formulier waarin niets is getypt sluit meteen, en een dialoog met één regel ook.
- Het opdrachtpalet, de kiezers (tab, thema, taal), de kleine review-, board-, merge- en composerdialogen en de bevestigingsvraag hebben ook een `✕`. Contextmenu's en keuzelijsten hebben er geen: ze sluiten met `Esc` of een klik buiten.

## Lijsten in de Instellingen

De lijsten in de Instellingen (Profielen, AI-profielen, Workflow, Openen in, Afbeeldingen) werken zo met de muis. **Klik op een subtabblad** (Algemeen of de lijst) om ernaartoe te wisselen; `Alt+Left` / `Alt+Right` doet hetzelfde.

- **Klik** op een rij om die te selecteren, **dubbelklik** om naar de velden ervan te gaan, **wiel** om te scrollen zonder te selecteren.
- **Druk op een rij en sleep** omhoog of omlaag om te herordenen: een balk toont waar de rij terechtkomt, loslaten zet hem neer, `Esc` annuleert. Het toetsenbordequivalent is `Alt+Omhoog` / `Alt+Omlaag`.
- **Klik op `+ Toevoegen`, `Dupliceren`, `Verwijderen`, `↑`, `↓`** onder de lijst. Toetsen: `Insert`, `Ctrl+D`, `Delete`, `Alt+Omhoog` / `Alt+Omlaag`; zie [Instellingen](settings.md#lijsten).

## Lijsten in pop-ups

Deze regels gelden voor elke lijst in een pop-up met een filterveld: het opdrachtenpalet, de
kiezers ervan (thema, taal, grafiekkleuren), de tabbladkiezer, doorzoekbare keuzelijsten,
de lijst met issue-verwijzingen en de help.

- **Klik** op een rij om haar te selecteren.
- **Dubbelklik** op een rij om haar uit te voeren of te kiezen, zoals `Enter` doet.
- **Het wiel** scrolt de lijst. Het verplaatst de selectie niet.
- Is de lijst langer dan de pop-up, dan verschijnt aan de rechterrand een **schuifbalk**.
  Sleep de greep of klik op het spoor.

Een korte keuzelijst zonder filterveld kiest de rij bij de eerste klik. Een contextmenu
voert de rij uit bij de eerste klik. De toetsen voor deze lijsten staan in
[Sneltoetsen](keybindings.md#filterbare-popuplijsten).

## Schuifbalken

Elk gebied dat kan scrollen toont een dunne schuifbalk op zijn rand als de inhoud niet
past: de Log, de Refs-secties, de bestandslijsten, de diff, de Bestandsweergave, de editor
en de lijsten in pop-ups.

- **Sleep de greep** om te scrollen.
- **Klik op het spoor** om de greep onder de aanwijzer te brengen: het zicht springt
  er naar verhouding heen, en zolang je de knop ingedrukt houdt, sleep je verder.
- **Houd de knop ingedrukt op een eindpijl** om te blijven scrollen. Het begint na een
  korte vertraging. Ga van de pijl af om te pauzeren.
- Een schuifbalk verplaatst alleen het zicht. Hij verplaatst nooit de selectie.

In de diff en de editor toont de verticale schuifbalk ook het hele bestand in kleine
markeringen: groen voor toegevoegde regels, rood voor verwijderde regels, oranje voor
gewijzigde regels, `◆` voor de cursorregel en `▌` voor elke andere plek waar de
geselecteerde tekst voorkomt. De merge-editor markeert elk conflict op zijn schuifbalken.
Klik op een markering om erheen te gaan.

## Fouten en het activiteitenlogboek

- **Klik op de rode fout** in de statusbalk om het activiteitenlogboek op het bijbehorende item te openen; **klik op Kopiëren / de voorgestelde knop** (Opnieuw, Eerst pull, Stash en opnieuw, Integraties openen) op het item.
- **Klik op Activiteit** aan het rechteruiteinde van de statusbalk om het activiteitenlogboek te openen. Klik op een tabblad om te wisselen, op een niveauvakje (of **Alles** / **Niets**) om die regels te tonen of te verbergen, op een regel van het tabblad Git-bewerkingen om de details te openen, op **Log wissen** op het tabblad Applicatie (er wordt eerst om bevestiging gevraagd). Scroll lange regels zijwaarts met `Shift` + wiel of de horizontale schuifbalk. Zie [Het activiteitenlogboek](troubleshooting.md#het-activiteitenlogboek).
- **Klik op `[x]`** bij de takenindicator rechts in de statusbalk om de lopende taak te annuleren, **klik op het label** om het takenpaneel te openen. De knop **Activiteit** rechtsonder op het tabblad Start (of `Ctrl+N Shift+G`) opent het activiteitenlogboek op Alle repositories (het vak Bereik, een keuzelijst onder de tabbladen, kiest een repository of een werkruimte).

## Formaat wijzigen

- **Sleep een scheidingslijn** tussen de zijbalk Refs, de Log en het rechterpaneel. Sleep
  hem voorbij de minimumbreedte om dat paneel in te klappen. Het toetsenbord verplaatst ze een stap per keer: `Ctrl+N Left` / `Ctrl+N Right` voor de scheiding Refs | Log en `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` voor de scheiding Log | rechterpaneel (de scheiding gaat de kant van de pijl op). Het palet heeft ze als `Navigatie: Refs-paneel verbreden`, `Navigatie: Refs-paneel versmallen`, `Navigatie: Statuspaneel verbreden` en `Navigatie: Statuspaneel versmallen`, en `Navigatie: Refs-paneel in-/uitklappen` en `Navigatie: Statuspaneel in-/uitklappen`.
- **Sleep de onderrand** van een Refs-sectie om haar hoogte te wijzigen.
- **Sleep de onderrand** van het commit-berichtblok in het rechterpaneel.
- **Sleep de rand tussen twee Log-koppen** (standaard de tussenruimte, vanaf de kop of vanaf overal in de tabel; Instellingen > Log & grafiek > Kolombreedte aanpassen) om een kolom van grootte te veranderen (toetsenbord: `Shift+C` om een kolom te kiezen, daarna `]` / `[`).
- **Dubbelklik op de rand tussen twee Log-koppen** om de kolom links ervan aan zijn inhoud aan te passen (het toetsenbord: `Shift+C` om een kolom te kiezen, dan `=`; `+` past elke kolom aan). De laatste kolom heeft een greep op zijn eigen laatste cel: sleep om die kolom van grootte te veranderen, waarbij de kolom Bericht het verschil opvangt.
- **Scroll de Log zijwaarts** (de kolomindeling `scroll`; Instellingen > Log & grafiek > Kolomindeling, of het selectievakje `Zijwaarts scrollen` in de keuzelijst): het horizontale wiel of `Shift`+wiel boven de tabel, de balk op de onderrand (sleep de schuif, druk op het kale spoor, klik op de pijlen aan de uiteinden), of `Shift+Links` / `Shift+Rechts`. In de standaardindeling `fit` vullen de kolommen het paneel en valt er niets te scrollen.
- **Sleep een Log-kop** om de kolom te verplaatsen: een spook van zijn naam volgt de aanwijzer, de kop die hij verliet wordt gedimd en een accentbalk markeert de plek waar hij zou landen. De kolommen verplaatsen als je loslaat; `Esc` annuleert en de aanwijzer toont de open hand zolang je hem draagt (toetsenbord: `Shift+C` om een kolom te kiezen, daarna `<` / `>`).

GitPoulpi onthoudt elke grootte. Zie
[Formaten worden onthouden](interface-tour.md#formaten-worden-onthouden).

## Titelbalk, tabbalk en werkbalk

- **Klik op 🐙** in de titelbalk om **Over** te openen. In Over volgen de pupillen van de
  octopus de aanwijzer; klik op het logo om het naar de volgende kleur te laten draaien, klik met rechts
  voor de lijst met kleuren, klik op een link om hem te openen.
- **Klik op Openen in ▾** in de titelbalk voor de lijst met programma's. Zie
  [Openen in](open-in.md).
- **Klik op de versie** (`v1.0.0`, of `vX.Y.Z beschikbaar`) in de titelbalk om het tabblad Releasenotes te openen.
- **Klik op het vergrootglas en `Globaal zoeken`** in de titelbalk, direct links van het tandwiel, voor het
  [globale zoeken](search.md).
- **Klik op het tandwiel** in de titelbalk, direct links van de ×, voor de Instellingen.
- **Klik op ×** aan het rechteruiteinde van de titelbalk om af te sluiten.
- **Klik op een tabblad** om ernaartoe te wisselen. **Klik op zijn `x`** (op het actieve
  tabblad en op het tabblad onder de aanwijzer) om het te sluiten. **Sleep een tabblad** opzij
  om de volgorde te wijzigen. **Klik met rechts op een tabblad** voor het tabbladmenu.
- **Middenklik op een tabblad** om het te sluiten, zoals zijn `x`.
- **Klik op de `+`** na het laatste tabblad voor een menu: Nieuw tabblad (`Alt+T`) en, zolang hun tabbladen gesloten zijn, Radar en Werkruimtes (`Alt+J`).
- **Klik op het huispictogram** voor het tabblad Start.
- **Klik op het tabblad Radar of Werkruimtes** om het te openen; zijn `x`, een middenklik of `Ctrl+W` sluit het. **Klik op de werkruimtechip** (`Alt+W`) om van werkruimte te wisselen.
- **Klik op Afsluiten & cd hier** (rechts van de tabbladen, als je GitPoulpi via de shellwrapper `gp` hebt gestart) om af te sluiten en je shell in de map van het actieve tabblad te laten. Zie [De map van het actieve tabblad volgen](following-the-tab-folder.md).
- **Klik op een werkbalkknop** om hem uit te voeren. **Klik op `▾`** naast Pull, Push of
  LFS voor zijn lijst. **Klik op `⋯`** voor de knoppen die niet pasten.
- **Klik op de profielkeuzelijst** aan het rechteruiteinde van de werkbalk om de
  commit-identiteit voor deze repository te kiezen.

## Tabblad Start

- **Klik** op een rij om haar te selecteren. **Dubbelklik** erop om de repository te openen.
- **Klik op de ster** om een favoriet toe te voegen of te verwijderen.
- **Klik op Openen, Wisselen, Sluiten of Verwijderen** op een rij, of op **`⋮`** voor het
  menu van de rij. **Klik met rechts** op een rij voor hetzelfde menu.
- **Klik op een sectietitel** om de sectie in te klappen.
- **Klik bovenaan op Bladeren, Klonen, Init, Alles inklappen of Alles uitklappen**, en op
  **Alle tabbladen sluiten** of **Alles verwijderen** op een sectietitel.

In de mapkiezer **Bladeren**: klik op een plaats of een map om ernaartoe te gaan, dubbelklik
op een map om hem te openen, klik op het padveld om een pad te typen. Vink **Naastgelegen mappen tonen** aan (`Alt+S`, wordt onthouden) en de linkerboom toont ook de andere mappen van elk geopend niveau: de mappen op het geopende pad tonen `▾`, de andere `▸`, en een klik op een map gaat ernaartoe. In **Klonen** en
**Init**: klik op een veld om erin te typen, en klik op een knop of een selectievakje om
het te gebruiken.

## Zijbalk Refs

- **Klik** op een rij om haar te selecteren. **Dubbelklik** op een lokale branch of een tag
  om hem uit te checken. **Dubbelklik** op een worktree of een submodule om hem in een
  tabblad te openen.
- **Wijs** een rij **aan** om het pictogram links ervan te tonen. **Klik** erop om de ref in de
  Log te verbergen, of weer te tonen. Staat **Solo** aan, dan soloot dezelfde plek de ref.
  Op een maprij werkt het pictogram op elke ref in de map.
- **Klik op Solo stoppen** of **Alles tonen** in de titel van de zijbalk om elke solo of
  elke verborgen ref te wissen.
- **Klik met rechts** op een rij voor haar menu: uitchecken, merge, fast-forward, rebase,
  een worktree maken, hernoemen, verwijderen, verbergen, solo en meer. De eigen rij van een
  remote heeft **Fetch**, **Bewerken**, **Verwijderen** en andere remote-acties. Een stash
  heeft **Pop**, **Toepassen** en **Drop**. Een grijze rij zegt waarom ze niet kan worden
  uitgevoerd.
- **Klik op `+`** op de titel REMOTE om een remote toe te voegen.
- **Klik op een maprij** om haar in of uit te klappen.

### Pull requests en issues

| Gebaar | Actie |
|---|---|
| Klik op een rij | Selecteer haar. |
| Dubbelklik op een rij | Open haar in de browser. |
| Rechtsklik op een rij | Haar menu: openen, de URL of het nummer kopiëren, de bronbranch uitchecken, een branch maken vanuit een issue. |
| Klik op `+` op de sectietitel | Open het formulier **Pull request maken** of **Nieuw issue**. |
| Klik op de filterrij (`Open ▾`) | Kies welke requests of issues worden getoond. |
| Klik op een groepstitel | Klap de groep in of uit. |
| Klik op de rij van provider of repository (ISSUES) | Kies de bron van de issues. **Geen (ontkoppel deze repository)** verwijdert haar. |
| Klik op de queryrij (ISSUES) | Typ een JQL- of WIQL-query (Jira, Azure DevOps), of kies Trello-lijsten. |
| Klik op de werkbalkchip (`GH #12`, `GL !53`, ...) | Open de pull request in de browser. Klik er met rechts op voor **Openen in browser**, **URL kopiëren**, **Vernieuwen** en **Profiel wisselen**. |
| Klik op de gedimde chip `+ PR` | Start een pull request voor de gepushte branch. |

Zie [Hosting en issuetrackers](hosting-integration.md).

## De Log

- **Klik** op een commit om hem te selecteren. Zijn details verschijnen in het rechterpaneel.
- **Ctrl+klik** voegt een commit aan de selectie toe of haalt hem eruit. **Shift+klik**
  selecteert een bereik (**Alt+klik** in terminals die Shift+klik voor hun eigen
  tekstselectie houden, zoals kitty en Windows Terminal). Klik met rechts op een geselecteerde
  commit om ze allemaal te cherry-picken. De selectie kan de WIP-rij en stashes bevatten; twee of
  meer rijen maken van het rechterpaneel een vergelijking, zie
  [Commits vergelijken](log-and-history.md#commits-vergelijken).
- **Klik op een kaart** van het vergelijkingspaneel om de Log-cursor naar die rij te verplaatsen;
  **klik op Omwisselen** om de uiteinden Van en Tot te verwisselen. Een gewone klik op een rij
  verlaat de vergelijking.
- **Klik op de WIP-rij** voor de lokale wijzigingen. **Klik op de tekst `// WIP`** om daar de
  samenvatting van de commit te typen.
- **Wijs een cel aan** voor de volledige tekst: het bericht, de auteur en het e-mailadres,
  de volledige datum, de volledige hash.
- **Wijs een badge `+N` aan** voor de andere refs op de commit. Ga in de pop-up om daar op
  een ref te klikken. Dubbelklik op een ref in de pop-up om hem uit te checken.
- **Dubbelklik op een refpil** om hem uit te checken.
- **Klik met rechts op een commit** voor het commit-menu. Het heeft één submenu voor elke
  ref op de commit.
- **Klik met rechts op een refpil** voor één menu met de acties van de ref en de acties van
  de commit.
- **Klik met rechts op de kop** om kolommen te tonen of te verbergen: een checklist (`[x]` / `[ ]`) met een ✕ in de titel; klik op een rij of gebruik de pijltoetsen en `Spatie` / `Enter`, en sluit hem met de ✕, `Esc` of een klik ernaast (het palet: `Weergave: Log: Kolommen kiezen om te tonen`). Onder de selectievakjes: `<kolom> aan inhoud aanpassen` (de kolom waarop met rechts is geklikt), `Alle kolommen aan inhoud aanpassen` en `Kolombreedtes herstellen`.
- **Klik op `↑`, `↓` of `✕`** in de zoekbalk om naar de vorige of volgende treffer te gaan,
  of de balk te sluiten.
- **Klik op `#12`, `ABC-12` of `AB#12`** in een commit-bericht, in het rechterpaneel, om dat
  issue of die pull request in de browser te openen.

### Secties van het linkerpaneel

**Mappen** (branches en tags met namen als `a/b/c`, de bestanden van het rechterpaneel in
boomweergave) klappen alle tegelijk in en uit: **klik met rechts op de kop of op de lege ruimte
van een sectie** die mappen heeft voor **Alles uitklappen** / **Alles inklappen** (en
**Secties tonen/verbergen...**, dat naar de afvinklijst hieronder leidt), en **klik met rechts op
een maprij** voor **Alle submappen uitklappen** / **Alle submappen inklappen** (de map en alles
eronder). Toetsen, in het paneel Refs, de lijsten Unstaged en Staged en de gewijzigde bestanden
van een commit evenzeer: `=` klapt alles uit, `-` klapt alles in, `*` (of `Shift+Right`) klapt de map
onder de cursor en alles eronder uit, `_` (of `Shift+Left`) klapt haar in. Palet: *Refs: Alle
mappen uitklappen*, *Refs: Alle mappen inklappen*, *Refs: De geselecteerde map en zijn submappen
uitklappen*, *Refs: De geselecteerde map en zijn submappen inklappen* en de vier tweelingen
*Bestanden:* voor de lijst met bestanden die de focus heeft. Zolang een naamfilter actief is (het
filter van Refs, *Bestanden filteren* bij een commit), staat elke map met een treffer open, wat je
ook had ingeklapt, en je eigen in- en uitklappen wordt niet gewijzigd of opgeslagen: wis het filter
en het komt terug. Een map die je tijdens het filter in- of uitklapt, werkt op wat getoond wordt en
wordt met het filter vergeten.

**Klik met rechts op een sectiekop** (een sectie zonder mappen, of het lege deel van het linkerpaneel) voor een
afvinklijst van de secties: Lokaal, Remote, Worktrees, Stashes, Tags, Pull requests,
Issues en Submodules. Een klik vinkt een sectie uit of aan; het menu blijft open. Een
verborgen sectie wordt niet getekend en het toetsenbord slaat haar over. De laatste
zichtbare sectie kan niet worden uitgevinkt. De keuze geldt per repository en blijft na een herstart bewaard; een repository zonder eigen keuze volgt de standaard (de globale `ui.sidebar_sections_hidden`). *Voor alle repositories gebruiken* maakt de huidige keuze de standaard en *Standaard herstellen* gooit de eigen keuze van deze repository weg (palet: *Linkerpaneel: Deze secties voor alle repositories gebruiken*). Palet: *Linkerpaneel: Secties tonen
of verbergen...*, één opdracht *tonen of verbergen* per sectie en *Secties terugzetten naar
standaard*.

### Refs slepen

Druk op een branch in de zijbalk Refs, of op een refpil in de Log, en sleep hem op een
andere branch. De statusbalk zegt wat je draagt. Een doel dat de drop kan ontvangen licht op.
Laat los voor een menu. `Esc` annuleert de sleepactie.

| Slepen | Menu |
|---|---|
| Lokale branch X op lokale branch Y | **Merge X in Y**, **Rebase X op Y**, **Interactief rebasen X op Y...**, **Fast-forward Y naar X** |
| Lokale branch op een remote branch | **Push X naar** die remote |
| Remote branch op lokale branch Y | **Merge**, **Rebase Y op** die branch, **Fast-forward Y naar** die branch, en **Pull** als hij de upstream van Y is |

Wijzigt de actie een branch die niet is uitgecheckt, dan checkt GitPoulpi hem eerst uit.
Weigert git de checkout, dan verandert er niets. Tags kunnen niet worden gesleept en kunnen
geen drop ontvangen.

Toetsenbord: "markeren en dan droppen" doet wat slepen doet, voor elk paar refs. Druk in het
Refs-paneel op `m` op een branch (palet: `Refs: Markeren als dropbron`). De ref toont een `◆` en is
onderstreept, ook op zijn labels in de Log, en de statusbalk zegt waar je heen moet. Selecteer een
andere ref, in welke sectie ook, en druk op `Shift+M` (palet: `Refs: Gemarkeerde ref hier droppen...`):
het menu dat slepen opent verschijnt, en zijn rijen werken precies zoals na een sleepbeweging. De
markering is verbruikt zodra het menu open is; `Esc`, of `m` op de gemarkeerde ref, wist haar eerder.
De opdracht `Refs: Geselecteerde ref droppen op uitgecheckte branch...` van het palet opent het menu
voor de geselecteerde ref en de huidige branch.

## Het rechterpaneel

### Lokale wijzigingen

- **Klik** op een bestand om zijn diff te openen. **Klik nogmaals op hetzelfde bestand** om
  terug te gaan naar de Log. Staat het bestand open in de editor met niet-opgeslagen
  wijzigingen, dan vraagt GitPoulpi eerst.
- **Wijs een rij aan** om haar knop **Bestand stagen** of **Bestand unstagen** te tonen. Een
  maprij heeft **Map stagen** of **Map unstagen**.
- **Klik op Alle wijzigingen stagen** of **Alle wijzigingen unstagen** op een sectietitel.
- **Klik op een sectietitel** (`Unstaged bestanden`, `Staged bestanden`) om haar in te
  klappen.
- **Klik op Pad of Boom** om een platte lijst of mappen te kiezen. **Klik op Alles
  inklappen** of **Alles uitklappen** om elke map in of uit te klappen.
- **Klik op `[✖]`** links van de titel om alle wijzigingen te verwerpen (vraagt eerst).
  **Klik op `[✦]`** rechts om de wijzigingen met AI uit te leggen.
- **Klik met rechts op een bestand** voor zijn menu: stagen, verwerpen, negeren, LFS, het
  bestand stashen, geschiedenis, blame, openen in een extern programma, het pad kopiëren,
  een patch maken, bewerken, verwijderen. Een conflicterend bestand begint met de rijen
  **Oplossen met ...**.
- **Klik met rechts op een maprij** voor het mapmenu: stagen, verwerpen, negeren, stashen,
  een patch maken, de map openen, er een bestand in maken.
- **Klik met rechts op het lege deel** van Unstaged bestanden voor **Bestand maken**.
- **Negeren**, **Als binair behandelen** en **Als tekst behandelen** (bestands- en mapmenu) openen eerst een lijst van de `.gitignore`- of `.gitattributes`-bestanden waarin de regel kan. Klik op een rij en druk op `Enter` (of gebruik de pijltjestoetsen); `Esc` of een klik ernaast annuleert zonder iets te schrijven.
- **Ctrl+klik** op een bestands- of maprij voegt die aan de selectie toe of verwijdert ze, **Shift+klik** (**Alt+klik**) neemt een bereik; hetzelfde in de gewijzigde bestanden van een commit en in Alle bestanden tonen. Een gewone klik begint opnieuw. **Rechtsklik op een geselecteerde rij**: het menu van alle geselecteerde bestanden.
- **Rechtsklik op het lege deel** van een lijst biedt ook **Repositorymap openen**; Unstaged voegt **Alle wijzigingen stagen** en **Alle wijzigingen verwerpen...** toe, Staged voegt **Alle wijzigingen unstagen** toe.
- **Converteren** (menu's van bestand, map en selectie) herschrijft de codering, de regeleinden of de inspringing van de bestanden op schijf; één `Ctrl+Z` maakt de reeks ongedaan.

### Het commit-formulier

- **Klik** op de Samenvatting of de Beschrijving om erin te typen.
- **Klik op Vorige commit amenden** om amend aan of uit te zetten.
- **Klik op Wijzigingen in N bestanden committen** om te committen. Is de knop grijs, dan zegt een klik
  waarom (bijvoorbeeld *Stage wijzigingen om te committen*).
- **Klik op `✦`** rechts van de Samenvatting om het bericht met AI te schrijven.
- **Klik op `#▾`** aan het eind van de Samenvatting voor de lijst met issues. Klik op een
  issue om zijn verwijzing in te voegen.
- **Klik op Commits samenstellen met AI** om de commit-composer te openen.

### Commit-details

- **Klik** op een gewijzigd bestand om zijn diff te openen. **Klik er nogmaals op** om terug
  te gaan naar de Log.
- **Sleep** de onderrand van het bericht om het van grootte te veranderen. Het wiel boven het
  bericht scrolt een lange beschrijving.
- **Rechtsklik** op de samenvatting, de beschrijving of het auteursblok voor *Samenvatting
  wijzigen...*, *Beschrijving wijzigen...* of *Auteur wijzigen...*; **dubbelklik** op een
  ervan (of op de auteursdatum) om het volledige venster Commit wijzigen te openen met de
  cursor in dat veld. De toetsenbordequivalenten zijn `Ctrl+G Shift+E` (alle velden) en
  `Ctrl+G Shift+A` (de auteur); samenvatting en beschrijving staan in het palet.
- **Rechtsklik** op een gewijzigd bestand, een maprij, het lege deel van de lijst of een selectie voor het menu (openen, geschiedenis, blame, openen in een programma, paden kopiëren, negeren, als binair of tekst behandelen, converteren, herstellen uit de commit, verwijderen). **Ctrl+klik** en **Shift+klik** selecteren meerdere bestanden.

Zie [Stagen en committen](staging-and-committing.md).

## De diff en de Bestandsweergave

- **Klik op een knop** in de balk boven de diff: Inhoud, Diff, Blame, de kolomkeuze
  `Bestand │ Alles`, Bewerken, `↑` `↓`, Inline, Split, Hunks, WS, Afbr., Kaart, Codering. Een
  knop die aan staat is gemarkeerd. Een klik op `Bestand` of `Alles` toont die lijst in de
  commit-kolom; een klik op het segment dat al gekozen is verbergt de kolom (zie
  [De commit-kolom](interface-tour.md#de-commit-kolom)).
- **Klik op `✕`** in de titel om het bestand te sluiten.
- **Wijs een hunk aan** om **Hunk stagen** of **Hunk unstagen**, en **Hunk verwerpen** te
  tonen.
- **Wijs een gewijzigde regel aan** om haar eigen knoppen **+ Stage**, **− Unstage** en
  **✖ Verwerpen** aan het rechteruiteinde van de rij te tonen.
- **Klik met rechts op een regel** voor het hunkmenu: de hunk of de geselecteerde regels
  stagen, unstagen of verwerpen, bestandsgeschiedenis, blame, kopiëren.
- **Klik op de minimap** om het zicht naar die regel te verplaatsen. Houd de knop ingedrukt
  en beweeg om te scrollen.

Elke verwerping vraagt eerst bevestiging.

### Blame en bestandsgeschiedenis

- **Blame.** Laat de aanwijzer op een annotatie rusten voor de hunkkaart; een klik selecteert een hunk, een rechtsklik opent zijn menu. Sleep de rand van de annotatiekolom om haar te herschalen (wordt opgeslagen). Klik of sleep op de minimap om te scrollen.
- **Bestandsgeschiedenis.** Een klik selecteert een commit; `Shift+klik` selecteert een bereik om te vergelijken; een dubbelklik toont de commit in de Log; een rechtsklik opent het rijmenu; sleep de rand tussen lijst en diff om te herschalen; de ✕ op de lijst verbergt haar (klik op de `›`-strook om haar terug te halen); klik op `Inhoud` / `Diff` / `Blame` in de balk om te wisselen wat de rechterkant toont, en op de keuze `Bestand │ Alles` om de lijst van de kolom te kiezen. Het wiel scrolt de helft onder de aanwijzer. Elk heeft een toetsenbordequivalent ([Blame](log-and-history.md#blame), [Bestandsgeschiedenis](log-and-history.md#bestandsgeschiedenis)).

### De commit-kolom (lijst Alles)

- **Wijs een rij aan** voor de tooltip van de Log (het hele bericht, de naam en het e-mailadres van de auteur, de volledige datum, naargelang het deel onder de aanwijzer).
- **Klik met rechts op een rij** voor haar menu: tonen in de Log, de diff van het bestand in die commit, het bestand bij die commit, de blame ervan, en dan het menu van de Log voor de commit. De menutoets doet hetzelfde met het toetsenbord.
- **Sleep een grens in de kopregel** (`ui.column_resize`: standaard de scheidingscel van één cel tussen twee delen, gemarkeerd met een gedempte `│` in de kop en als accentlijn over de kolom getekend zolang de aanwijzer erop staat, waarbij slepen vanaf elk punt ervan schaalt; met `header` de eerste letter van de volgende naam, die oplicht zolang de aanwijzer erop staat) om het deel links ervan te schalen: banen, bericht, auteur, datum, refs, wijzigingen of SHA. De cel direct na het laatste deel is de grens van het laatste deel.
- **Sleep een naam van de kopregel** over een andere om het deel daarheen te verplaatsen: hetzelfde spook, dezelfde gedimde bron en accentbalk als bij de kop van de Log, en het deel verplaatst als je loslaat (`Esc` annuleert; `<` / `>` op het toetsenbord, na `Tab`, doen hetzelfde, één plaats per keer).
- **Rechtsklik op de kopregel** om te kiezen welke delen getoond worden (Ref, Grafiek, Auteur, Datum, Wijzigingen, SHA; Bericht is altijd zichtbaar); de checklist heeft een ✕ in de titel. Het palet heeft `Kolommen kiezen` en een omschakelaar per deel.
- **Shift+wiel** of een horizontaal wiel scrolt de rijen zijwaarts als ze breder zijn dan de kolom.
- **Sleep de rechterrand** van de kolom om haar te herschalen. **Klik op de strook `›`** (of op de ✕ op de kolom, of **klik op het gekozen segment** van de keuze `Bestand │ Alles` in de balk) om haar terug te halen of te verbergen.

### Tekst selecteren en kopiëren

Je kunt de tekst van de diff, de Bestandsweergave, Blame en Bestandsgeschiedenis selecteren en
kopiëren.

- **Druk en sleep** om tekst te selecteren. De selectie blijft op dezelfde tekst staan
  als je scrolt.
- **Dubbelklik** om een woord te selecteren.
- **Een gewone klik** selecteert geen tekst. Ze verplaatst de cursor of selecteert de rij,
  en wist de tekstselectie.
- **`y` of `Ctrl+C`** kopieert de selectie. Zonder selectie kopiëren ze de huidige regel.
- **Klik met rechts** in de diff voor **Selectie kopiëren**, **Regel kopiëren** en **Hunk
  kopiëren**.

GitPoulpi kopieert alleen de tekst van het bestand. Regelnummers, de tekens `+` en `-`,
hunkkoppen en de Blame-kolommen worden niet gekopieerd. In Split blijft een selectie aan de
kant waar je met slepen begon.

In de diff selecteert slepen ook de regels die het kruist, zoals `v` doet. `Enter` stagt die
regels dan. `Esc` wist eerst de tekstselectie, dan de regelselectie en sluit dan de diff.

Tekst selecteren kleurt ook elke andere plek in waar die tekst voorkomt, in de diff en de
editor.

## Tekstvelden

Dit geldt voor elk tekstveld: filters, de zoekbalk, het palet, de Samenvatting en de
Beschrijving, velden in dialoogvensters en waarden in de Instellingen.

- **Klik** om de cursor daar te plaatsen.
- **Druk en sleep** om tekst te selecteren.
- **Dubbelklik** om een woord te selecteren.
- Typ om de selectie te vervangen. `Backspace` of `Delete` verwijdert haar.

De toetsen voor tekstvelden staan in [Sneltoetsen](keybindings.md).

## De editor

- **Klik** om de cursor daar te plaatsen. **Shift+klik** breidt de selectie uit.
- **Sleep** om te selecteren. Sleep voorbij de boven- of onderrand om te scrollen.
- **Dubbelklik** om een woord te selecteren.
- **Het wiel** scrolt zonder de cursor te verplaatsen.
- **Klik op Opslaan, Ongedaan, Opnieuw of Zoeken** in de balk. In zoekmodus gaan **`↑`** en
  **`↓`** naar de vorige en volgende treffer.
- **Klik op `✕`** om te sluiten. GitPoulpi vraagt eerst bij niet-opgeslagen wijzigingen.

## Afbeeldingen

Zie [Afbeeldingen](image-view.md) voor de muis in de afbeeldingsweergave.

## Dialogen verplaatsen en schalen

Dialogen kunnen worden verplaatst en de grotere vensters geschaald:

- **Sleep de titelbalk** om een dialoog te verplaatsen (het sluitkruis houdt zijn klik). Het opdrachtpalet en de kiezers hebben geen aparte titelbalk: sleep de bovenrand.
- **Sleep de linker-, rechter- of onderrand**, of een onderhoek (het `◢`-handvat), om een venster te schalen. Het dialoog blijft op het scherm en wordt nooit kleiner dan zijn minimum (60 x 16 voor de Geschiedenis van ongedaan maken, kleiner voor de andere). Een geschaald venster gebruikt de ruimte: tekstvelden en lijsten groeien, tekst loopt door op de nieuwe breedte.
- De sleepbeweging gaat door als de aanwijzer het dialoog verlaat, en loslaten erbuiten sluit het nooit.
- **Sleep de scheidingen** binnen de weergave Reflog / Verloren werk van de Geschiedenis van ongedaan maken; dubbelklik op een scheiding om hem te herstellen.
- De toetsenbordtegenstukken zijn `Ctrl+N` en dan `m` (pijlen verplaatsen, `Shift`+pijlen schalen, `0` herstelt, `Enter` of `Esc` verlaat de modus) en `Alt`+pijlen voor de scheidingen; het palet heeft `Dialoog: Verplaatsen of schalen` en `Dialoog: Positie en grootte herstellen`. Ze werken op het bovenste dialoog. `Ctrl+N` werkt op elk vlak hetzelfde: het toont een badge op de bedieningselementen van het venster, dialoog of de popup vooraan (velden, lijsten, tabbladen, knoppen en het sluitkruis), de toets erna drukt dat element in, `m` start de schikmodus en `Esc` annuleert; de statusregel noemt `<badge>`, `m=schikken` en `Esc`. Dat geldt ook voor het opdrachtpalet, de kiezers, de werkruimtezoeker en de help-overlay, waar `Ctrl+N` niet meer omlaag in de lijst gaat. `Alt+G` (`palette.arrange`) is nog een manier, geldig boven elk dialoog en elke popup, om het voorste te schikken zonder het prefix.
- **Een instelling** (`ui.dialog_geometry`, Instellingen > Weergave > Indeling) geldt voor elk dialoog: elk onthoudt zijn eigen plaats en grootte voor deze sessie, op schijf voor de volgende start, of nooit.

| Dialoog | Verplaatsen | Schalen |
|---|---|---|
| Geschiedenis van ongedaan maken / Reflog / Verloren werk, Activiteitenlog, takenpaneel, mapkiezer | ja | ja |
| Opdrachtpalet, tab-/thema-/taalkiezers, werkruimtezoeker | ja | ja (de lijst groeit met de hoogte, de rijen met de breedte) |
| Clone (de repositorylijst), Pull request en Nieuw issue (de beschrijving), "Wat wordt verzonden" (het paneel), Mermaid (het voorbeeld), review-opties (de velden, alleen breedte) | ja | ja |
| Lange-tekstbewerkers in Instellingen (sjablonen, hooks, AI-prompts), het AI-vak van het werkruimteformulier | ja | ja (het tekstveld) |
| Geschiedenis herschrijven: de twee AI-hulpdialogen | ja | ja (de beschrijving, het verzonden paneel) |
| Info, updatepaneel, beveiligingscontrole, voorspelde conflicten, "Over de AI-limiet", tag verwijderen, gevoelige bestanden | ja | nee (de grootte volgt de tekst) |
| Init, Worktree, Sparse checkout, Flow-dialoog, issueflow-dialogen, commitberichtformulier (reword), werkruimteformulier, label-/mijlpaalkiezers | ja | nee |
| Bevestigingen (elke "Weet je het zeker?") | ja | nee |
| Review- en Board-dialogen: hun kleine dialogen (opmerking, afronden, kiezen, kolommen) | ja | nee |
| Popups van de toetsenbordpagina (resetbevestiging, toetsopname), de AI-antwoordpopup | ja | nee |
| Instellingen: de resultaten en bevestiging van de repositoryauthenticatie | ja | nee |
| Geschiedenis herschrijven: purge-, identiteits- en vraagdialogen | ja | nee |
| Verplaats-bestand-popup van de commitcomposer, merge-meldingen | ja | nee |

Niet verplaatsbaar: de schermvullende pagina's (Instellingen, Help, de Review- en Board-pagina's, die het venster al vullen), contextmenu's en keuzelijsten (ze hangen aan wat ze opende), tooltips en de kleine popups binnen een pagina (het remotes-formulier, de editor voor het splitsen van een commit).

De aanwijzer toont een verplaatsvorm boven de titelbalk en een schaalvorm boven de randen en hoeken (terminals met `OSC 22`).

## Vorm van de muisaanwijzer

Sommige terminals laten GitPoulpi de muisaanwijzer veranderen: een formaataanwijzer boven
scheidingslijnen, boven een kolomgrens van de Log, de Radar en de commitkolom (ook tijdens het
slepen ervan), een hand boven klikbare items. Dit werkt in kitty, foot, Ghostty, WezTerm en
recente xterm (de reeks `OSC 22`). Hij wordt alleen verstuurd als de vorm verandert, nooit in
tmux, screen, zellij of herdr, en de standaardaanwijzer wordt hersteld als GitPoulpi afsluit.
In andere terminals verandert de aanwijzer niet. De scheidingslijn of de knop onder de aanwijzer
licht in elke terminal op.

Hetzelfde soort component geeft overal in de app dezelfde aanwijzer.

- **De wijzende hand** staat op bedieningselementen: knoppen, selectievakjes en schakelaars,
  keuzelijsten (in Instellingen, de panelen met issues en pull requests, de Radar, dialogen),
  tabs en hun kruisjes, de `+`-knoppen, links en verwijzingen (`#12`, `!45`), groeps- en
  mapkoppen met hun uitklappijl, ref-pillen, schuifbalken, en de knoppen Activiteit, taken en
  versie van de statusbalk. Een selectievakje of knop in een rij houdt de hand.
- **De pijl** staat op rijen. Een rij licht als geheel op als de aanwijzer erop staat (hetzelfde
  licht in elke lijst) en is zelf geen bedieningselement: de rijen van de panelen Refs,
  Worktrees, Stashes en Submodules, de Log en de commitkolom (alleen een ref-pil krijgt de
  hand), de regels van een diff, de gewijzigde bestanden, de items van pull requests, issues en
  de Radar, en de rijen van de lijsten in dialogen en panelen (klonen, mapkiezer, Workspaces,
  de kaarten van het bord, het paneel Taken, kiezers voor toegewezenen, labels en milestones,
  rebase-stappen, reviewbestanden en -commits, keuzes in menu's en keuzelijsten).
- **De tekstcursor (I-beam)** staat op invoervakken, over het hele vak: de zoekvakken (globaal
  zoeken, Instellingen, het commandopalet, Home), de filters (Refs, bestanden, Radar,
  Activiteitenlog, de zoekbalk van het issuespaneel), de tekst-, getal- en gebiedsrijen van
  Instellingen (ook tijdens het bewerken), de velden van dialogen, de reviewopmerkingsvakken,
  de commit-composer, de tekst van de bestandseditor en het bewerkbare resultaat van de
  mergeeditor (de alleen-lezen panelen houden de pijl). In de commitdetails zijn het bericht
  en de beschrijving bewerkbaar, dus de I-beam staat erop en het hele blok licht licht op.
- **In de commitdetails** zijn de inforegels bedieningselementen: klik op `commit: <id>` om de
  korte id te kopiëren, dubbelklik op de auteur- of datumregel om te amenden (de hand en een
  licht merkteken wijzen erop). De statistiekregel informeert alleen en houdt de pijl.

De formaataanwijzers blijven boven scheidingslijnen en kolomgrenzen, en de open hand verschijnt
alleen terwijl je een ref draagt.

## Kopiëren naar het klembord

**Pad kopiëren** en enkele andere kopieeracties sturen de tekst naar de terminal, die haar op
het klembord zet (OSC 52). Ondersteunt je terminal dit niet, dan wordt er niets gekopieerd. De
kopieeracties voor geselecteerde tekst gebruiken direct het systeemklembord.

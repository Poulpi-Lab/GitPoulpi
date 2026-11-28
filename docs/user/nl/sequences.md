Een **reeks** is een benoemde lijst opdrachten die na elkaar worden uitgevoerd:
bijvoorbeeld *Alle wijzigingen stagen*, *AI: Commitbericht genereren*, *Commit*,
*Push* als één **Backup**-toets. Elke stap is een opdracht van het
[commandopalet](keybindings.md); een reeks is een eigen paletrij
(`Reeks: <naam>`), kan aan een toets worden gebonden en wordt bewerkt in
**Instellingen > Reeksen** of in `config.toml`.

## De pagina Reeksen

**Instellingen > Reeksen** is een [lijstpagina](settings.md#lijsten): links de reeksen,
rechts de rijen van de geselecteerde reeks. Elke wijziging wordt meteen toegepast
en geschreven, en het palet en de sneltoetsen volgen zonder herstart.

```
 Reeksen
+- Reeksen ----------------+  Reeks 1/2 bewerken
| Deze installatie         |  Naam          Backup
| > Backup         Alt+B   |  Id            backup
|   Worktree en agent      |  Beschrijving  Alles stagen, ...
| Deze repository          |  Bereik        Deze installatie
|   Release                |  Sneltoets     [ Alt+B ] [ Wissen ]
+--------------------------+  Stappen       Stage all > AI: ... > Commit > Push
| [+ Toevoegen v] [Dup]    |               [ Stappen bewerken (4) ] [ Nu uitvoeren ]
| [Omhoog] [Omlaag]        |
+--------------------------+
```

* **+ Toevoegen** biedt **Lege reeks** en de drie voorinstellingen: **Uit
  voorinstelling > Backup**, **Samenstellen en push** en **Worktree en agent**.
  Een voorinstelling is een startpunt: er is niets actief tot je hem toevoegt, en
  wat je toevoegt is een kopie die je kunt bewerken. Voorinstellingen krijgen
  bij het toevoegen een naam in de taal van de interface.
* **Dupliceren**, **Verwijderen** (vraagt eerst), **Omhoog** / **Omlaag** of
  slepen wijzigen de volgorde van de lijst (`Insert`, `Ctrl+D`, `Delete`,
  `Alt+Up`, `Alt+Down`). De volgorde van de lijst is de volgorde in
  `config.toml`.
* Een rij met een probleem is rood gemarkeerd en zegt waarom (geen stappen, een
  stap die deze versie niet kent, een pauze `Nooit` waarbij de nodige optie
  ontbreekt). Een reeks met problemen **kan worden opgeslagen**, onbekende
  toekomstige stappen blijven zoals ze zijn, maar zijn paletrij blijft grijs
  tot het is opgelost.

### De rijen van een reeks

| Rij | Wat het doet |
|---|---|
| **Naam** | Wat het palet toont: `Reeks: <naam>`. |
| **Id** | De korte naam in `config.toml`, op de opdrachtregel en als sleutel van de sneltoets (`a-z`, `0-9`, `_`, `-`). Hernoemen verplaatst de sneltoets mee, in dezelfde schrijfactie. |
| **Beschrijving** | Optioneel, voor jezelf. |
| **Bereik** | Zie hieronder. |
| **Sneltoets** | Een knop die de toets toont. `Enter` of een klik opent dezelfde toetsvastlegging als **Sneltoetsen**: conflicten en terminalwaarschuwingen verschijnen terwijl je drukt, en een toets die een andere actie gebruikt wordt op dezelfde manier geweigerd. **Wissen** verwijdert hem. De toets wordt opgeslagen als `[keybindings.sequence] <id>`. |
| **Stappen** | De stappen op volgorde. **Stappen bewerken** opent ze. |
| **Nu uitvoeren** | Voert de reeks uit in het huidige venster. |

### Bereik

Elke reeks heeft een bereik, dat met hem wordt opgeslagen:

| Bereik | Waar het wordt opgeslagen | Wie hem ziet |
|---|---|---|
| **Deze installatie** (standaard) | `[[sequences]]` in `config.toml`, met deze installatie als eigenaar | Alleen deze installatie van GitPoulpi. |
| **Alle installaties op deze computer** | `[[sequences]]` in `config.toml`, zonder eigenaar | Elke installatie die het bestand deelt. |
| **Alleen deze repository** | `.gitpoulpi/sequences.toml` in de huidige repository | Iedereen die de repository heeft. Hij telt pas mee zodra je hem vertrouwt (zie hieronder). |

Het bereik wijzigen verplaatst de definitie: **Alleen deze repository** kiezen
schrijft hem naar het bestand van de repository en verwijdert hem uit
`config.toml`, en omgekeerd. Voor een repositorybereik moet een repository
geopend zijn.

### Repositoryreeksen en vertrouwen

Het bestand `.gitpoulpi/sequences.toml` kan worden gecommit, dus het kan van
anderen komen, en een reeks voert opdrachten uit op jouw computer. De reeksen
van een repository worden daarom **pas aangeboden nadat je ze vertrouwt**, zoals
de opdrachten van een gecommit workflowbestand.

Heeft een repository zo'n bestand dat nog niemand vertrouwde, dan verschijnt er
een melding boven het Log: *Deze repository definieert 2 reeksen* met
**Bekijken**, **Vertrouwen** en **Negeren**. Ze blokkeert nooit iets.

- **Bekijken** toont elke reeks met elke stap, zijn pauze en zijn opties,
  alleen-lezen. **Vertrouwen** daar vertrouwt wat getoond is; **Sluiten** (of
  `Esc`) vertrouwt niets. Is de lijst langer dan het venster, dan wordt
  Vertrouwen daar niet aangeboden (vergroot het venster): je vertrouwt nooit wat
  je niet kon lezen.
- **Vertrouwen** (de knop, of de paletrij *Reeks: De reeksen van deze repository
  vertrouwen*) vertrouwt het bestand precies zoals het nu is.
- **Negeren** verbergt de melding tot het bestand verandert.
- Via het toetsenbord: het palet heeft *Reeks: De reeksen van deze repository
  bekijken / vertrouwen / negeren*.

Wat wordt vastgelegd is een digest van de id's, de opdrachten, de pauzes en de
opties, bewaard in de lokale status van de repository (nooit in de repository
zelf). Verandert daar iets van, bijvoorbeeld na een pull, dan komt de melding
terug als *De reeks van deze repository is gewijzigd* en verdwijnen de reeksen uit
het palet en van de toetsen tot je ze opnieuw vertrouwt. Een reeks hernoemen of
zijn beschrijving bewerken verandert niets. Een reeks die je zelf in de
Instellingen schrijft blijft vertrouwd als het bestand al vertrouwd (of leeg) was
voordat je het bewerkte.

Eenmaal vertrouwd verschijnen repositoryreeksen als `Reeks: <naam>` in het palet
zoals je eigen reeksen en kunnen ze in de Instellingen een sneltoets krijgen. Een
reeks met dezelfde id als een van jou is een clash: de jouwe wint en de andere
wordt als duplicaat gemeld.

Op de opdrachtregel: `gitpoulpi sequence list` toont de reeksen van het bestand
met `trusted` en een `repo_trust`-status (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (en `untrust`, `--repo PATH` voor een andere
repository) is bedoeld voor een persoon, na het lezen van `sequence show`.
`sequence run` weigert een niet-vertrouwde repositoryreeks. AI-agents kunnen niet
vertrouwen: er is geen MCP-tool voor.

## Stappen

**Stappen bewerken** vervangt de lijst door de stappen van de reeks (de titel
zegt **Stappen van <naam>**) en de rijen door die van de geselecteerde stap.
**Terug** (het kruisje boven de rijen) of `Esc` brengt je terug naar de reeksen.

* **+ Toevoegen** opent **Een stap toevoegen**: elke opdracht die een reeks mag
  uitvoeren, onder zijn paletnaam (de AI-opdrachten met het sterretje),
  gefilterd terwijl je typt met de matching van het palet. Een stap die deze
  versie niet kan uitvoeren is grijs met de reden. `Enter` of een klik voegt hem
  achteraan toe; `Esc` of het kruisje sluit de popup.
* **Dupliceren**, **Verwijderen**, **Omhoog** / **Omlaag** en slepen werken zoals
  bij de reeksen.
* Een destructieve opdracht (een branch verwijderen, wijzigingen verwerpen...)
  wordt nooit aangeboden: een reeks kan er geen uitvoeren.

### Pauze

Elke stap heeft een **Pauze**:

| Pauze | Betekenis |
|---|---|
| **Standaard** | De stap stopt alleen als hij een antwoord nodig heeft, in zijn gebruikelijke dialoogvenster (een branchnaam, een commitbericht dat nog leeg is...). |
| **Altijd** | Stop vóór deze stap en vraag om door te gaan. |
| **Nooit** | Nooit stoppen. De opties van de stap moeten hem alles geven wat hij nodig heeft, dus een stap die om een naam vraagt heeft zijn optie **Naam** ingevuld nodig. |

Een stap die altijd vraagt voordat hij draait (een force push) biedt **Nooit**
niet aan: de rij zegt waarom. **Nooit** op de stap met het AI-bericht is
toegestaan als bewuste keuze.

### Opties

De rijen onder **Pauze** zijn de opties van de stap: een tekst **Naam** of
**Bericht**, een **Branch**, een selectievakje zoals **Openen in een nieuw
tabblad**, of een keuzelijst gevuld vanuit je configuratie (een **Remote**, een
**AI-profiel**, een **Item van Openen in**). Teksten accepteren de variabelen
`${branch}`, `${date}`, `${time}` en `${n}` (een getal dat de naam uniek maakt).
Een tekst wissen verwijdert de optie.

## Maken en bewerken met AI

Met de [AI aan](ai.md#reeksconcept) (Instellingen > AI) verschijnen twee items:
**Toevoegen > Maken met AI...** (ook de paletrij `Reeks: Maken met AI...`, die
de Instellingen op deze pagina opent met het dialoogvenster) en **Bewerken met
AI...** naast **Nu uitvoeren** voor de geselecteerde reeks. Geen van beide bestaat
zolang de AI uit staat.

```
+- Een reeks maken met AI ---------------------------------- x -+
| Beschrijf de reeks die je wilt                                |
| +-----------------------------------------------------------+ |
| | alles stagen, het commitbericht met de AI schrijven,      | |
| | commit en daarna push                                     | |
| +-----------------------------------------------------------+ |
| Enter voegt een regel toe. Ctrl+Enter verstuurt. Esc sluit.   |
| [ ] Context meesturen (remotenamen, AI-profielen, labels ...) |
|    Uit: alleen je beschrijving wordt verstuurd.               |
| De AI doet alleen een voorstel. Er wordt niets opgeslagen ... |
| [ Versturen ] [ Annuleren ]                                   |
+---------------------------------------------------------------+
```

Je schrijft wat je wilt in je eigen woorden; **Versturen** (of `Ctrl+Enter`,
`Ctrl+S`) vraagt het model, via hetzelfde profiel, dezelfde takenindicator en
dezelfde `Esc` als elke andere AI-actie (en dezelfde vraag "Wat wordt
verstuurd"). Het antwoord is een **voorstel** dat je leest voordat er iets
gebeurt:

* de naam, id, beschrijving en de stappen met hun pauze en opties;
* bij **Bewerken met AI** een diff tegen de huidige reeks: `+` een stap
  toegevoegd, `-` een stap verwijderd, `~` een stap waarvan de pauze of opties
  veranderden, `~ Naam: "a" -> "b"` voor een veld;
* **Weggelaten**: een stap die het model voorstelde maar die niet is toegestaan
  (onbekend, destructief, een reeks in een reeks, een onmogelijke pauze), met de
  reden; de geldige stappen blijven behouden;
* de **opmerkingen** van het model, en een **voorgestelde sneltoets** die alleen
  wordt getoond, met of de toets al in gebruik is. Hij wordt nooit gebonden:
  gebruik daarna de knop **Sneltoets**.

```
+- Voorgestelde wijziging ---------------------------------- x -+
| + toegevoegd   - verwijderd   ~ gewijzigd                     |
| ~ Naam: "Daily" -> "Daily 2"                                  |
| Stappen (3)                                                   |
| + 1. Git: Fetch All Remotes                                   |
|   2. Git: Stage All Changes                                   |
| - Git: Commit...                                              |
| ~ 3. Git: Push: standaardwaarden -> Pauze: Altijd             |
| Weggelaten                                                    |
| stap 4 `reset_hard`: destructieve stappen zijn niet toegestaan|
| [ Accepteren ] [ Bewerken vóór opslaan ] [ Afwijzen ] [ Terug ]|
+---------------------------------------------------------------+
```

* **Accepteren** schrijft het met één opslag (een nieuwe reeks, of de vervanging
  van de bewerkte, die zijn id, bereik en toets behoudt) en selecteert hem in de
  lijst.
* **Bewerken vóór opslaan** doet hetzelfde en opent zijn stappen zodat je de
  rijen kunt bijstellen (deze pagina slaat elke wijziging meteen op, dus de
  reeks is al opgeslagen).
* **Afwijzen** schrijft niets. **Terug** keert terug naar de beschrijving om
  opnieuw te versturen.
* **Accepteren** is niet beschikbaar als er niets te accepteren valt: geen
  bruikbare stap, een probleem dat de pagina zou markeren, of een bewerking die
  niets verandert.

Er wordt niets geschreven vóór **Accepteren**, en een reeks wordt hier nooit
uitgevoerd. Het dialoogvenster sluiten (het kruisje, `Esc`, een klik erbuiten)
vraagt "Wijzigingen verwerpen?" zolang er een beschrijving getypt is of een
voorstel open staat; de Instellingen sluiten vraagt het ook.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<id van deze installatie>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Sleutels die deze versie niet kent, en stappen die ze niet kent, blijven
behouden wanneer de pagina opslaat.

## Een reeks uitvoeren

Een reeks is een benoemde lijst stappen, elk een opdracht van het palet (alles stagen, het commitbericht met de AI schrijven, commit, push...). Voer hem uit vanuit het palet (`Reeks: <naam>`), met de toets die je hem gaf in Instellingen > Toetsenbord en muis (`[keybindings.sequence]` in `tui.toml`), of vanaf de opdrachtregel (`gitpoulpi sequence run`).

Tijdens het uitvoeren toont een strook boven het Log `<naam> k/n`, de stap en waarop hij wacht. De uitvoering staat ook in het paneel Taken als `Reeks <naam> k/n: <stap>`. Er draait één reeks tegelijk; een tweede starten zegt welke er draait.

**Waar hij stopt om te vragen.** Elke stap heeft een instelling `pause` (`default`, `always`, `never`):

| Stap | Standaard | `pause = "never"` |
|---|---|---|
| AI-commitbericht | het bericht komt in het berichtveld; de strook zegt "Controleer het bericht en kies Doorgaan" | gaat meteen door |
| Commit | een leeg bericht stopt altijd: het veld krijgt het toetsenbord, en je commit (of Doorgaan) gaat verder | niet beschikbaar als er geen bericht is om te committen |
| AI: commits samenstellen | de review opent; Toepassen gaat verder, sluiten annuleert | past het voorstel toe zoals het is |
| Worktree maken | het dialoogvenster opent met de naam ingevuld | maakt hem aan vanuit `options.name` zonder dialoogvenster |
| Push | bij `git.push_behavior = ask` opent de keuzelijst "Push naar..." | pusht naar `options.remote` |
| Force push met lease, bestanden stagen die op geheimen lijken | vraagt altijd | niet toegestaan |

`pause = "always"` op een stap toont "Volgende: <stap>" met **Doorgaan** en **Annuleren** voordat hij draait. Wordt de push geweigerd omdat de branch is uiteengelopen, dan opent het gebruikelijke dialoogvenster en zegt de strook "Push geweigerd: kies in het dialoogvenster"; eerst pullen en dan pushen (of een force push met lease) zet de uitvoering voort, het dialoogvenster sluiten annuleert haar.

**Doorgaan** is de knop van de strook, `Reeks: Doorgaan` in het palet, of de toets gebonden aan `sequence.continue`. **Annuleren** is het sluitkruisje van de strook, `Reeks: Annuleren` (`sequence.cancel`), het paneel Taken, of `Esc` als niets anders hem wil (een open dialoogvenster sluit eerst, wat ook annuleert). Annuleren behoudt de stappen die al liepen en zegt dat: "<naam> geannuleerd na stap k/n (<stap>); eerdere stappen blijven staan".

**Als een stap mislukt** stopt de uitvoering daar: de statusregel noemt de reeks, de stap en de fout, de strook wordt rood met **Details** (het item in het activiteitenlogboek, dat de opties van de stap vermeldt) en een sluitkruisje, en de eerdere stappen blijven gedaan.

**Worktrees.** Een stap Worktree maken met `open_tab = true` verplaatst de uitvoering naar het tabblad van de nieuwe worktree: de volgende stappen (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) werken daar. Staat er een ander tabblad in beeld, dan zegt de strook "wacht op tabblad <naam>" met **Ga erheen**. Dat tabblad sluiten annuleert de uitvoering. Een uitvoering overleeft het afsluiten van GitPoulpi niet.

### Een agent die een uitvoering voorstelt

Met *AI-agents wijzigingen laten voorstellen die ik bevestig* aan (`[mcp]
allow_ui_propose`) kan een agent je venster vragen een reeks uit te voeren
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Er verschijnt een
banner: *✦ Voorgesteld door <agent>: reeks <naam> uitvoeren* met **Uitvoeren** en
**Afwijzen**. Er draait niets tot je op Uitvoeren drukt (klik erop, of `Ctrl+N` en
dan `Enter`); Afwijzen, `Esc` (als niets anders hem wil) of de paletrij *Agent:
Openstaande weergave verwerpen* laat het vallen. De uitvoering verloopt dan precies
alsof je haar zelf had gestart, en pauzeert waar de reeks dat zegt. Het voorstel
wordt geweigerd als de reeks niet bestaat, ongeldig is, of een repositoryreeks is
die je niet vertrouwt.

## Een uitvoering ongedaan maken

Elke stap die de repository wijzigt (stage, commit, tag, branch, stash, worktree...) laat een **eigen gewoon Ongedaan-item** achter, gemarkeerd met de uitvoering, dus `Ctrl+Z` maakt nog steeds één stap tegelijk ongedaan en de [Geschiedenis van ongedaan maken](undo-redo.md#de-geschiedenis-van-ongedaan-maken) toont `⟳ <naam>` erbij. Om de hele uitvoering terug te draaien is er **Hele reeks ongedaan maken**: de knop van de Geschiedenis van ongedaan maken (`s`), de paletrij `Bewerken: Hele reeks ongedaan maken`, of `gitpoulpi undo --sequence`. Het maakt de items van de uitvoering ongedaan, nieuwste eerst, elk met zijn eigen controles en vragen (een commit die al gepusht was vraagt eerst), stopt bij de eerste weigering met de naam van de stap, en laat de oudere in het journaal. Het AI-bericht en een push laten geen item achter: een push wordt nooit ongedaan gemaakt. Alleen het journaal van de huidige repository wordt aangeraakt; stappen in een worktree worden ongedaan gemaakt vanuit die repository. Opnieuw doen blijft per item. Details: [Reeksuitvoeringen](undo-redo.md#reeksuitvoeringen).

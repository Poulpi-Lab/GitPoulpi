[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · **Nederlands** · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# De Radar

De Radar is een eigen tabblad, direct na Start: één lijst van wat op je wacht in **meerdere repositories**: de pull (of merge) requests die
je schreef, die waarin om jouw review wordt gevraagd, de issues die aan jou zijn toegewezen of door jou zijn
gemaakt, en de repositories waarin je werk in uitvoering liet staan. Het is het venster Start met de
Radar-weergave erbovenop, zodat niets eronder een klik of toets kan afpakken.

Hij heeft een codehost-profiel nodig (zie [Hosting en issuetrackers](hosting-integration.md); de lijsten van één repository staan in [Pull requests en issues](pull-requests-and-issues.md), en de discussie van een rij opent als [gesprek](conversations.md)) en
repositories die eraan gekoppeld zijn, of een profiel met zijn accountbrede lijsten ingeschakeld (zie "Voorbij het bereik: accountbrede rijen" hieronder).
Een repository die niet gekoppeld is wordt nooit bevraagd: ze toont
een regel **Koppelen...** (zie "Repositories met iets te melden" hieronder).

![De Radar boven twee repositories: een vastgepinde aanvraag, dan Wacht op mijn review, Mijn pull requests, Aan mij toegewezen, issues en werk in uitvoering](../images/radar.png)

**Reviewen...** op de rij van een pull- of merge request (of `e`) opent de repository met de review van de aanvraag in de app
([Een pull request reviewen](review.md)). Bij een aanvraag van een repository die je klonde maar nooit aan de host koppelde, wordt de kloon eerst
aan het project van de aanvraag gekoppeld (tenzij hij al een eigen pull-requestkoppeling heeft), zodat de review in haar tabblad begint.

### De kolom Acties

Rechts van een rij met een aanvraag of issue doen kleine knoppen wat een toets op de geselecteerde rij doet (een klik selecteert de rij en
voert de actie uit; laat de aanwijzer op een knop rusten voor zijn naam en toets):

```
Breed:   #58  Round invoice totals  feature/login → main   alice   2d   Review  [globe]  [talk]  ⊕  ×  ⇣  ⋮
Smal:    #58  Round invoice totals  feature/login → main                 [rev]  [globe]  [talk]  ⋮
Issue:   #12  Crash on empty list                         bob     5h   Werk starten  [globe]  [talk]  ×  ⋮
```

| Knop | Toets | Palet |
|---|---|---|
| **Review** (een aanvraag die op jouw review wacht; vraagt hoe als hij niet gekloond is) | `e` | `Radar: Verzoek reviewen...` |
| **Werk starten** (een gekloond issue) | `n` | `Radar: Werk aan issue starten...` |
| de globe: de pagina op de host openen | `o` | `Radar: Openen in browser` |
| het regelsglyph: het gesprek openen | `i` | `Radar: Issuegesprek openen` |
| `»` **Markeren als klaar** (een conceptaanvraag) | `y` | `Radar: Markeren als klaar` |
| `⊕` **Merge** (een aanvraag) | `m` | `Radar: Merge...` |
| `×` **Sluiten** (een aanvraag of een issue) | `x` | `Radar: Sluiten...` |
| `⇣` **Klonen** (een rij waarvan de repository niet gekloond is) | `g` | `Radar: Repository klonen...` |

Een rij zonder actie laat haar cel leeg, zodat elke knop op elke rij in dezelfde kolom blijft. Als het venster smal is, worden de woorden Review en
Werk starten glyphs voordat een andere kolom verdwijnt; de knoppen Klaar, Merge, Sluiten en Klonen verlaten de kolom voordat de titel dat doet, maar blijven in
het rijmenu (`⋮`, rechtsklik, of `Shift+F10`), waar een actie die niet kan draaien grijs is met haar reden. De kolom is automatisch (hij is geen
van de kolommen waarvan je de breedte kunt aanpassen). Een dubbelklik of `Enter` op een aanvraag die niet gekloond is vraagt hoe hij te reviewen, zoals **Review**.

### De branches van een aanvraag

Een aanvraagrij toont `bron → doel` achter haar titel (`alice:feature/login → main` als de bronbranch in een fork staat).
Als er weinig ruimte is, wordt eerst het doel afgekapt (een beletselteken), houdt de titel het grootste deel van de ruimte, en worden in een smal venster de
branches niet getoond; de kolom Acties wordt nooit weggeduwd. Laat de aanwijzer op een afgekapte tekst rusten voor het geheel. Een host die de branches niet
noemde toont daar niets. Een issuerij noemt haar repository in dezelfde chip als een aanvraagrij (de mapnaam van de kloon, of `owner/name` als hij niet gekloond is).

### Handelen op de host

**Merge**, **Sluiten** en **Markeren als klaar** werken op GitHub, GitLab, Gitea en Forgejo (andere hosts maken ze grijs met "Niet ondersteund door ...";
issues kunnen op dezelfde hosts worden gesloten).

- **Merge** vraagt eerst: het vak noemt de aanvraag, haar project en branches, en biedt de methode (een merge-commit, squash,
  en rebase waar de host dat heeft; GitLab merget of squasht). Het wordt met zijn reden geweigerd voor een concept, een aanvraag die
  conflicteert met haar doel, of mislukkende checks; een weigering door de host (geen toestemming, een regel van de branch) staat op de
  statusbalk.
- **Sluiten** vraagt eerst en sluit zonder te mergen.
- **Markeren als klaar** haalt een concept uit de conceptstatus (GitLab en Gitea: de markering `Draft:` / `WIP:` verdwijnt uit de titel). Het vraagt
  niets: het is op de host omkeerbaar.
- **Klonen** opent het gewone dialoogvenster Klonen op het project van de rij (de URL is ingevuld, de mapkiezer is de gebruikelijke). Zodra
  de kloon bestaat plaatst de Radar de rij erop de volgende keer dat hij de remotes leest (heropen hem of Vernieuwen), en Klonen zegt dan
  waar de kloon staat.

Merge en Sluiten wijzigen de host en **kunnen niet vanuit GitPoulpi ongedaan worden gemaakt** (het vak zegt het); heropen een gesloten aanvraag op de host.
Na een geslaagde aanroep worden de lijsten opnieuw opgevraagd, dus een gemergede of gesloten rij verdwijnt en een rij die klaar is verliest haar `draft`-chip.

## Openen

- **`Alt+R`** (herbindbaar: `global.radar`), vanaf overal.
- Het palet: `Radar: Openen`.
- Het tabblad **Radar**, direct na Start in de tabbalk.
- De rij **Radar** van het menu **`+`**, zolang het tabblad gesloten is.

Vanuit een repositorytabblad brengt dit je naar het venster Start met de Radar open; `Esc` of **Terug**
keert terug naar de lijst met repositories, en de `⌂` doet hetzelfde.

## Het tabblad sluiten of de Radar uitzetten

Het tabblad sluit zoals een repositorytabblad: met zijn **x**, een klik met het middelste muisknopje, `Ctrl+W` terwijl het getoond wordt, of de opdracht van het palet `Radar: Tabblad sluiten`. Start is het enige tabblad dat niet kan sluiten. Een gesloten tabblad blijft voor die sessie gesloten, ook na een herstart, tot je de Radar weer opent (`Alt+R`, `Radar: Openen` of het menu `+`); dan staat het weer direct na Start. Sluiten wijzigt geen instelling.

Gebruik je hem nooit, zet hem dan uit onder **Instellingen > Algemeen > Radar inschakelen** (`general.radar_enabled`, zie de [configuratiereferentie](config-reference.md)). Uit verdwijnen het tabblad, de opdrachten van het palet en `Alt+R`, en haalt GitPoulpi er niets voor op. Het werkt direct, zonder herstart.

## Wat hij omvat: het bereik

De keuzelijst **Bereik** bepaalt welke repositories worden bevraagd:

- een **werkruimte**, bij naam (haar repositories, ook die zonder open tabblad);
- **Open tabbladen**: de repositories die als tabblad van deze sessie openstaan.
- **Standaardmap om te klonen** en **Standaardmap om te bladeren**: elke repository die in de map staat die je in
  de [Instellingen](config-reference.md) hebt ingesteld (`repos.clone_root`, `repos.browse_root`), met of zonder tabblad.

Hij opent op het laatste bereik dat je hier koos (bewaard in `radar.toml`), zolang het nog bestaat; vóór
je eerste keuze, of als die werkruimte is verwijderd, op de actieve werkruimte van de sessie, anders op de open
tabbladen. De keuze geldt alleen voor deze weergave: ze verandert de actieve werkruimte van het venster niet. Als twee
repositories dezelfde mapnaam hebben, komt de bovenliggende map in de chip zodat je ze kunt onderscheiden.

### De twee mapbereiken

Een mapbereik zoekt repositories **twee niveaus diep**: de mappen die er direct in staan, en de mappen in
die mappen die zelf geen repository zijn (`~/src/work/app`). Verborgen mappen worden overgeslagen, een
repository wordt nooit binnengegaan, en er worden hoogstens 200 repositories getoond. De zoektocht loopt op
de achtergrond, nooit op de draad die tekent, en het resultaat blijft staan tot je ververst: de knop
Vernieuwen, `r`, `F5` (de verversingstoets van het venster, opnieuw toewijsbaar) of `Radar: Vernieuwen` doorzoekt de map opnieuw (een repository die sindsdien is
gekloond, verschijnt dan).

Hosting blijft per repository opt-in: een repository in de map die **niet is gekoppeld** aan een host wordt
nooit bevraagd. Ze levert alleen haar rij met werk in uitvoering, heeft onder Repositories een regel `niet
gekoppeld` met **Koppelen...** (die opent het gedeelte met pull requests van die repository, waar je de
provider en het project kiest), en de statusbalk telt ze (`3 niet gekoppeld (alleen werk in uitvoering)`).

Een map die niet is ingesteld, is grijs in de lijst (`Standaardmap om te bladeren (stel in via Instellingen)`);
kiezen opent haar rij in de Instellingen (Instellingen > Algemeen) en laat het bereik zoals het was. De
kloonmap telt als ingesteld via haar standaard per besturingssysteem (de map waarop het dialoogvenster Klonen
begint), zolang die map bestaat; de bladermap heeft geen standaard. De lijst met bereiken wordt telkens als
hij opent uit de instellingen gelezen, dus een wijziging daar geldt meteen.

## De lijst

| Groep | Wat erin zit |
|---|---|
| Vastgepind | Alles wat je vastpinde, van elke soort, in de volgorde waarin je het vastpinde |
| Wacht op mijn review | Aanvragen waarin om jouw review wordt gevraagd (en die je niet zelf schreef) |
| Mijn aanvragen | De aanvragen die je schreef; de chip zegt `concept` bij een concept |
| Aan mij toegewezen | Aanvragen die aan jou zijn toegewezen |
| Waarin ik word genoemd | Open aanvragen en issues waarin je wordt genoemd en die nog niet in een groep hierboven staan (een issue dat niet aan jou is toegewezen en niet door jou is gemaakt staat hier alleen als het je noemt) |
| Overige aanvragen | De overige open aanvragen |
| Aan mij toegewezen issues | Open issues die aan jou zijn toegewezen |
| Door mij gemaakte issues | Open issues die je maakte |
| Werk in uitvoering | Repositories van het bereik met niet-gecommitte wijzigingen of niet-gepushte commits |
| Uitgesteld | Items die je een tijd verborg; ingeklapt tot je ze opent |
| Repositories | Repositories met iets te melden (offline, token te vernieuwen, niet gekoppeld) |

De woorden "pull request" en "merge request" volgen de gebruikelijke terminologie-instelling en de host.

Elke rij toont de soort item (`⇄` aanvraag, `●` issue, `✎` werk in uitvoering), de repository-chip (in de
kleur van de werkruimte), het nummer, de titel, de auteur, de leeftijd en, waar ruimte is, de labels. Een rij
met werk in uitvoering toont de branch en wat erin wacht: `✎` gewijzigde bestanden, `+` nieuwe bestanden, `↑`
te pushen commits, `↓` te pullen commits.

De aantallen staan in elke kop. Binnen een groep komt het meest recent bijgewerkte item eerst.

De statusbalk onder het filter toont een spinner als de hosts er langer dan een moment over doen om te
antwoorden. De gecachte lijsten (dezelfde cache als die van de zijbalk) staan er meteen en worden vervangen
naarmate de antwoorden binnenkomen.

### Kolommen

De repository-chip, de `#` (nummer en vinkjes), de auteur en de leeftijd zijn **instelbare kolommen**. Op automatisch is de repositorykolom zo breed als de langste naam op het scherm (tot 40 cellen, en hooguit een derde van het venster), zodat `owner/name` en de `group/sub/project` van een accountlijst op een gewone terminal niet worden afgekapt. De titel krijgt de rest. Een naam die niet past eindigt op een weglatingsteken (`group/sub...`), en:

- **Houd de aanwijzer stil** op een afgekapte repository, titel, auteur of labellijst en een tooltip toont de hele tekst. Selecteer de rij en de statusbalk onder het filter noemt ook het volledige project en de volledige titel, zodat het toetsenbord hetzelfde ziet.
- **Sleep het scheidingsteken** (de cel direct na de repositorynaam, na de `#`, of vlak voor de auteur of de leeftijd) om de kolom te vergroten of te verkleinen; **dubbelklik** erop om de kolom weer te laten passen. De lijst heeft een **kopregel** (repository, `#`, titel, auteur, leeftijd) en de scheidingstekens volgen `ui.column_resize` net als in de Log: met `line` (de standaard) staat een gedempte `│` in het scheidingsteken op de kopregel, en zolang de aanwijzer op het scheidingsteken staat (op de kop of op een rij van de lijst), de kolom met de toetsen is gekozen of je sleept, wordt een dunne accentlijn over de hele lijst getekend; slepen vanaf elk punt ervan schaalt de kolom. Zodra je loslaat, verdwijnt de lijn tenzij de aanwijzer nog op het scheidingsteken staat (dan verdwijnt ze met de aanwijzer): een muissleep laat nooit een kolom gekozen. Met `header` is er geen streepje of lijn: de eerste cel na het scheidingsteken, op de kopregel, licht op en is de enige plek om vanaf te slepen. De aanwijzer wordt een formaataanwijzer boven een scheidingsteken in terminals die dat ondersteunen. De statuschip heeft een vaste breedte, en de leeftijd ook (de breedste getoonde leeftijd, tot je haar schaalt; een leeftijd die door een smalle kolom is afgekapt toont de volledige datum in een tooltip), zodat de auteur op elke rij in dezelfde cel begint.
- Of gebruik de toetsen: **`C`** kiest een kolom (nogmaals, of `Tab`, voor de volgende), **`]`** verbreedt hem met twee cellen, **`[`** versmalt hem, **`0`** zet alle kolommen terug op automatisch, `Esc` verlaat. Het palet heeft hetzelfde: `Radar: Kolom kiezen`, `Radar: Kolom verbreden`, `Radar: Kolom versmallen`, `Radar: Kolombreedtes herstellen`.

De breedtes worden bewaard in `radar.toml` (`[columns]`, `repo`, `number`, `author` en `age`, in cellen) en blijven na een herstart behouden. Het zijn dezelfde formaatgebaren als die van de kolommen van de Log.

### Checks en conflicten

Een aanvraagrij draagt twee kleine markeringen na haar nummer als de host iets zei:

| Markering | Betekenis |
|---|---|
| `✓` | de checks van de head-commit zijn geslaagd |
| `✗` | een check is mislukt |
| `●` | checks staan in de wachtrij of lopen |
| `!` (ernaast) | de aanvraag conflicteert met haar doelbranch |

Laat de aanwijzer op de markeringen rusten en een tooltip zegt het in woorden ("Checks mislukt", "Conflicteert met main");
selecteer de rij en de statusbalk onder het filter zegt hetzelfde, zodat ook het toetsenbord het ziet. Een aanvraag
zonder markering is er een waarover de host niets zei (geen CI ingesteld, of niet gevraagd).

- **GitLab** stuurt de pipelinestatus en de conflictvlag mee met de lijst van merge requests: elke rij heeft
  haar markeringen.
- **GitHub** niet: de lijst heeft geen van beide. GitPoulpi vraagt ze per aanvraag (de aanvraag zelf voor
  haar samenvoegstatus, de gecombineerde status van de head-commit en zijn check runs: drie aanroepen), voor hoogstens
  **15 open aanvragen per repository**, de jouwe eerst. Als GitHub een van die aanroepen weigert (een rate limit),
  blijven de rest ongemarkeerd tot de volgende vernieuwing. De antwoorden worden met de hostingcache bewaard.

- **Gitea en Forgejo** worden gelezen zoals GitHub: de lijst draagt de conflictvlag (`mergeable`), de commitstatus
  wordt per aanvraag gevraagd (hoogstens 15 per repository, de jouwe eerst) en de issues zijn dezelfde groepen.
  Vermeldingen gebruiken de zoekfunctie van de server (`mentioned_by`).

### Vermeldingen

**Waarin ik word genoemd** en het filter **Toon: Waarin ik word genoemd** gebruiken wat de host over vermeldingen weet. GitHub
wordt gevraagd via zijn zoekfunctie (`mentions:`), GitLab via je takenlijst: alleen vermeldingen die daar nog **openstaan**
tellen, dus een taak die je als gedaan markeerde verlaat de groep.

Gesloten en gemergede aanvragen en gesloten issues staan niet in de lijst.

## Weergaven en filters

Drie keuzelijsten en een tekstveld beperken de lijst. Elke keuze is een keuzelijst: klik erop, of gebruik zijn
toets.

- **Weergave** (`v`): Alles, aanvragen, Issues, WIP (werk in uitvoering), Uitgesteld.
- **Toon** (`f`): Iedereen, Van mij (aanvragen die je schreef, issues die je maakte), Aan mij toegewezen,
  Review gevraagd, Waarin ik word genoemd.
- **Bereik** (`w`): zie boven.
- **Filtertekst** (`/` of `Ctrl+F`): fuzzy match op repository, nummer, titel, auteur, branch en labels.
  `Enter` geeft de toetsen terug aan de lijst en houdt de tekst, `Esc` wist hem.

`Ctrl+N` toont een badge op het filter, elke keuzelijst, de knoppen Vernieuwen en Terug en elke sectiekop op het scherm. Druk op het teken van een badge: het filter krijgt het toetsenbord, een keuzelijst gaat open, een knop wordt uitgevoerd en een kop krijgt de selectie (`Enter` of `Space` klapt hem dan in). `Esc` annuleert. Zie [Badges](keybindings.md#badges).

## Acties

Selecteer een rij en gebruik haar menu (rechtsklik, de `⋮` aan het eind van de rij, `.` of `Menu`), een
toets, of het palet (`Radar: ...`). Een muisactie heeft altijd een toets en een palettweeling.

| Actie | Toets | Palet |
|---|---|---|
| Repository openen (haar tabblad opent, of wordt geactiveerd; een aanvraag die niet gekloond is vraagt hoe hij te reviewen) | `Enter` / dubbelklik | `Radar: Repository openen` |
| Reviewen... (een aanvraag; bij een niet-gekloonde vraagt het hoe) | `e` | `Radar: Verzoek reviewen...` |
| Een aanvraag reviewen via de API van de host (geen kloon) | `a` in de vraag | `Radar: Reviewen via de API` |
| De repository klonen, dan de aanvraag reviewen | `c` in de vraag | `Radar: Klonen en reviewen...` |
| Openen in browser | `o` | `Radar: Openen in browser` |
| Branch uitchecken (een aanvraag) | `c` | `Radar: Branch uitchecken` |
| Worktree maken... (een aanvraag) | `t` | `Radar: Worktree maken...` |
| Tonen in de repository (haar lijst) | `s` | `Radar: Tonen in de repository` |
| Werk aan issue starten... (een issue) | `n` | `Radar: Werk aan issue starten...` |
| Bovenaan vastpinnen, of Losmaken | `p` / klik op de `★` | `Radar: Vastpinnen of losmaken` |
| Toegewezenen... / reviewers... / labels... bewerken, Mijlpaal instellen... (GitHub, GitLab) | rijmenu | `Radar: Toegewezenen bewerken...`, `Radar: Reviewers bewerken...`, `Radar: Labels bewerken...`, `Radar: Mijlpaal instellen...` |
| Uitstellen... | `z` | `Radar: Uitstellen...` |
| Niet meer uitstellen | `u` | |
| De aanvragen van niet-gekloonde repositories tonen of verbergen | `a` | `Radar: Niet-gekloonde verzoeken tonen` |
| Concepten van de accountbrede rijen tonen of verbergen | `d` | `Radar: Concepten tonen` |
| De accountbrede reviewverzoeken van een host inschakelen | `T` / `Enter` op zijn hintrij / **Inschakelen** | `Radar: Accountbrede reviewverzoeken inschakelen...` |
| Vernieuwen | `r`, `F5` | `Radar: Vernieuwen` |
| Het rijmenu | `.` | `Radar: Rijmenu...` |
| Terug naar Start | `Esc` | `Radar: Terug naar Start` |

Acties die het eigen venster van de repository nodig hebben (uitchecken, worktree, tonen, werk starten,
koppelen, de metadatakiezers) wisselen eerst naar haar tabblad en draaien daar zodra het geladen is:

- **Uitchecken** gebruikt het eigen uitchecken van de zijbalk: de branch als hij lokaal bekend is, anders de
  head van de aanvraag opgehaald in een lokale branch (bijvoorbeeld die van een fork).
- **Worktree maken...** opent het dialoogvenster Worktree maken op de branch van de aanvraag.
- **Tonen in de repository** selecteert het item in de lijst PULL REQUESTS of ISSUES.
- **Werk aan een issue starten** opent de prompt Nieuwe branch met een naam gemaakt van het issue
  (`12-fix-the-thing`).

### Vastpinnen en uitstellen

Een **pin** houdt een item bovenaan. **Uitstellen** verbergt het voor **1 uur**, **4 uur**, **Tot morgen**
(08:00 de volgende ochtend, je lokale tijd, ook als het nu nog geen 08:00 is) of **Tot het verandert** (de host meldt een update op het item, of, voor een rij met werk in
uitvoering, zijn tellers bewegen). Beide staan in `radar.toml` in je configuratiemap, gesleuteld op host,
repository, soort en nummer, en worden gedeeld door elke sessie op de machine.

### Repositories met iets te melden

Een repository met een probleem krijgt een eigen regel aan het eind van de lijst, en haar gecachte items
blijven op het scherm:

- **offline, 5m oud**: de host was niet bereikbaar;
- **Opnieuw verbinden**: het token ontbreekt of is geweigerd; de knop opent Instellingen > Integraties;
- **gepauzeerd in deze ronde**: de host zei dat het rustiger aan moest (of weigerde het token), dus GitPoulpi
  vroeg hem in deze ronde niet opnieuw voor de andere repositories;
- **niet gekoppeld, GitHub voorgesteld**: de remote van de repository komt overeen met een profiel, maar
  hosting is per repository opt-in, dus er wordt niets gevraagd tot je koppelt. **Koppelen...** brengt je naar
  de sectie PULL REQUESTS van de repository, waar je de provider en het project kiest.

Er worden hoogstens vier repositories tegelijk bevraagd, en nooit op de thread die het scherm tekent.

## Voorbij het bereik: accountbrede rijen

Het bereik omvat de repositories die je hebt gekloond. Een aanvraag die op je review wacht in een repository
die je nooit hebt gekloond, is daar onzichtbaar. **Accountbrede rijen** vragen de host zelf, één keer per profiel, en
voegen zich bij de lijst, wat het bereik ook is: ze gaan over jou, niet over een map.

Ze staan **standaard uit** (de zoekopdracht van een host kost API-quotum). Zolang een verbonden codehost geen van
zijn lijsten aan heeft, toont de bovenkant van de lijst er een regel voor:

`GitHub: toon de verzoeken die op jouw review wachten, ook in repositories die je niet hebt gekloond  [Inschakelen]`

Klik op **Inschakelen**, druk op `Enter` op die regel, druk op `T`, of voer `Radar: Accountbrede reviewverzoeken
inschakelen...` uit: het schrijft `integrations.radar.review_requests = true` voor dat profiel in `config.toml` en
vraagt de host meteen. De andere lijsten (elk openstaand verzoek van je namespaces, aan jou toegewezen issues,
concepten, namespaces) stel je in met `gitpoulpi hosting integrations radar` of in de instellingen van het profiel; zie
de [configuratienaslag](config-reference.md) en de [CLI-naslag](cli-reference.md). GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center en Gerrit antwoorden; Bitbucket Cloud heeft geen accountbreed
eindpunt, dus zijn aanvragen verschijnen alleen voor de gekoppelde repositories van het bereik.

De **Namespaces** van het profiel beperken al zijn lijsten tot bepaalde repositories (Instellingen > Integraties > Radar: een tekstgebied met één item per regel, en een selectievakje **Reguliere expressies**). Een regel komt overeen met het volledige pad van een repository als hij eraan gelijk is of een padvoorvoegsel is dat bij een `/` eindigt: `orbit` houdt `orbit/products/x` en laat `orbitx/y` en `training/y` vallen. Met het selectievakje aan is elke regel een reguliere expressie die op het volledige pad wordt uitgevoerd zoals je hem schrijft (`^orbit/(products|infra)/`); een regel die niet geldig is, wordt onder het veld getoond en niet opgeslagen. Leeg betekent geen limiet.

Hoe de rijen worden geplaatst:

- Een rij die het bereik al toont (dezelfde host, hetzelfde project, dezelfde soort en hetzelfde nummer) verschijnt één keer: **de
  rij van het bereik wint**.
- Een rij waarvan je het project ergens hebt gekloond (recente repositories, werkruimtes, de mappen die de
  mapbereiken vonden; vergeleken op de remote-URL) draagt die kloon: **Enter**, uitchecken, worktree en de
  rest openen hem als elke rij.
- Elke andere rij is **niet gekloond**: de repositorychip is gedimd en toont het project (`acme/app`), en
  de statusbalk zegt `Niet gekloond` als ze geselecteerd is. **Openen in browser** werkt, **Reviewen** vraagt hoe
  je haar wilt reviewen (hieronder), en uitchecken, worktree, tonen in de repository en werk starten
  zeggen `kloon eerst de repository`.

### Een verzoek reviewen

**Reviewen** (`e`) op een aanvraag van de eigen repositories van het bereik opent het tabblad van die repository met de review
erin. Bij een aanvraag van een accountbrede rij hangt de weg af van of een kloon bekend is en van de
instelling **Verzoek van een niet-gekloonde repository** van het profiel (Instellingen > Integraties, `review_mode`
in [de configuratienaslag](config-reference.md)):

| Instelling | Een kloon is bekend | Geen kloon bekend |
|---|---|---|
| Vragen (standaard) | vraagt: **De lokale kloon openen** (`l`) of **Reviewen via de API** (`a`) | vraagt: **Klonen en dan reviewen** (`c`) of **Reviewen via de API** (`a`) |
| Klonen | opent de kloon | opent het dialoogvenster Klonen |
| Reviewen via de API | reviewt via de API | reviewt via de API |

De vraag is een klein dialoogvenster: de letters kiezen, **Enter** kiest de gemarkeerde rij, `Esc`, **Annuleren**,
het `✕` en een klik erbuiten sluiten het, en **Onthouden voor GitHub** (`r`) schrijft de instelling van het profiel zodat de
vraag niet opnieuw wordt gesteld (de rij in Instellingen verandert het terug). `Ctrl+N` geeft zijn bedieningen badges.

- **Reviewen via de API** opent de review in het Home-venster, over de Radar, met de code van
  de host (zie [Reviewen zonder kloon](review.md#reviewen-zonder-kloon)). `Esc` of het `✕` geeft
  de Radar terug; de rijen *Review: ...* van het palet werken erop.
- **Klonen en dan reviewen** opent het dialoogvenster Klonen op het verbonden account waar de rij vandaan kwam (dezelfde lijst repositories,
  gegroepeerd per eigenaar), met de repository van het verzoek geselecteerd en de standaard kloonmap ingevuld; wijzig
  een van beide, en kloon, met dezelfde aanmelding als elke kloon van dat account. Staat de repository niet in de lijst van het
  account (je bent er geen lid van), dan blijft de lijst open met de zoekopdracht op haar naam en een melding zegt dat; bij een host die geen
  repositories kan opsommen opent het dialoogvenster op **Klonen met URL** met het adres ingevuld. Is het klonen klaar, dan wordt de nieuwe repository aan de host gekoppeld
  (haar bron voor PULL REQUESTS, en haar bron voor ISSUES als de host issues heeft en er geen is ingesteld), opent haar tabblad en
  is de review van die aanvraag erin open. Het dialoogvenster annuleren, of een kloon die mislukt, koppelt niets.
- Aanvragen van Azure DevOps en Bitbucket hebben geen review in de app: **Reviewen** opent de pagina van de aanvraag in de
  browser en zegt dat in de statusbalk.

De paletrijen *Radar: Reviewen via de API* en *Radar: Klonen en reviewen...* slaan de vraag over.

Twee selectievakjes naast het filter (ze verschijnen zodra een accountbrede lijst wordt gevraagd), bewaard in
`radar.toml`:

| Selectievakje | Toets | Standaard | Effect |
|---|---|---|---|
| `[x] Niet gekloond` | `a` | aan | Toont de rijen van repositories zonder bekende kloon. Uit blijven alleen rijen van bekende klonen over. |
| `[ ] Concepten` | `d` | uit | Toont conceptaanvragen van de accountbrede rijen. Concepten van de eigen repositories van het bereik worden altijd getoond. |

Beide staan op de badges van `Ctrl+N` en in het palet (`Radar: Niet-gekloonde verzoeken tonen`, `Radar: Concepten tonen`).

Het antwoord van een profiel wordt **5 minuten** bewaard in `radar-account.json`: de Radar binnen die
tijd opnieuw openen toont de rijen zonder de host te vragen. **Vernieuwen** (`r` of `F5`) vraagt altijd: elke repository en elke accountbrede lijst, wat de leeftijd van het laatste antwoord ook is. Een host die zei dat het
rustiger aan moest of het token weigerde, wordt in dezelfde ronde niet opnieuw gevraagd. Er worden hoogstens 300 rijen bewaard per profiel en soort,
en een regel onder **Repositories** zegt hoeveel er nog meer waren, of waarom een profiel niet kon worden gelezen
(**Opnieuw verbinden** opent Instellingen > Integraties).

## Grenzen

- Accountbrede rijen komen van GitHub-, GitLab- (ook self-managed), Gitea-, Forgejo-, Azure DevOps-,
  Bitbucket Data Center- en Gerrit-profielen (niet Bitbucket Cloud). Aan jou toegewezen issues komen van de eerste
  vijf; Bitbucket en Gerrit hebben geen issues.
- De lijsten per repository gebruiken GitHub-, GitLab- (ook self-managed), Gitea-, Forgejo-, Azure DevOps-,
  Bitbucket- en Gerrit-profielen; checks, conflicten en vermeldingen worden alleen voor de eerste vier gelezen.
- Vermeldingen op GitLab komen alleen uit openstaande taken; checks van GitHub, Gitea en Forgejo worden voor 15 aanvragen per repository gevraagd.

[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · **Nederlands** · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Een pull of merge request reviewen

Review een pull request (GitHub, Gitea, Forgejo), een merge request (GitLab) of een Gerrit-wijziging (zie [Gerrit](#gerrit))
**zonder het venster van de repository te verlaten**. De reviewweergave toont de beschrijving, het gesprek, de gewijzigde
bestanden met de opmerkingendiscussies onder hun regels en de commits, en daar schrijf je de review: opmerkingen,
goedkeuring en verzoeken om wijzigingen. Je hebt een profiel nodig voor een van die hosts (zie
[Hosting en issuetrackers](hosting-integration.md)) en een repository die eraan gekoppeld is; andere providers bieden
geen reviews. De grenzen van een host worden gerespecteerd: GitLab kent geen oordeel *Wijzigingen aanvragen* en neemt
één regel per opmerking. Deze pagina noemt ze allemaal een *verzoek*.

![De reviewweergave, pagina Overzicht: de kop van het verzoek met zijn metadataknoppen, de beschrijving, een onopgeloste en een opgeloste codediscussie, de opmerkingen en het opmerkingenvak](../images/review-overview.png)

![De pagina Bestanden: de gewijzigde bestanden met hun vakjes voor bekeken, het verschil met een opmerkingendiscussie onder zijn regel, en een opgeloste discussie buiten de getoonde regels](../images/review-files.png)

## Een review openen

Elke ingang leidt naar dezelfde weergave:

- de lijst **PULL REQUESTS** van de zijbalk: selecteer een rij en druk op **Enter**, of klik er met rechts op en kies
  **Reviewen...**;
- de **Radar**: het menu van de rij, **Reviewen...**, of de toets `e` (de repository opent en de review erbij;
  bij een verzoek van een repository die je niet hebt gekloond vraagt het of je die wilt klonen of via de API wilt
  reviewen, zie [Reviewen zonder kloon](#reviewen-zonder-kloon));
- de **werkruimtezoeker** (`Ctrl+Alt+F`): markeer een pull of merge request en druk op **`Alt+R`**;
- het palet: **Hosting: Verzoek reviewen...** toont de verzoeken om uit te kiezen.

De weergave vult het venster over de repository (het is een modus van het repositorytabblad, geen apart venster).
**`Esc`** of de `✕` in de hoek verlaat hem. Terwijl hij opent, wordt het verzoek van de host gelezen en wordt de
code ervan op de achtergrond met git opgehaald; een spinner verschijnt als dat langer dan even duurt.

## De kop

Altijd zichtbaar: het nummer en de titel, de status (Open, Concept, Samengevoegd, Gesloten), de branches
(`main ← feature/moons`), de auteur, de checks (`✓ checks geslaagd`, `✗ checks mislukt`, `● checks bezig`),
of het zonder conflicten kan worden samengevoegd of conflicten heeft, de omvang, en **`3 conceptopmerkingen`** als je er hebt geschreven.

De vier metadataknoppen - **Toegewezen**, **Reviewers**, **Labels**, **Mijlpaal** - openen dezelfde kiezers als
de zijbalk (zie [Toegewezenen, reviewers, labels en mijlpaal](pull-requests-and-issues.md#toegewezenen-reviewers-labels-en-mijlpaal));
de kop toont het resultaat zodra de host het heeft overgenomen. De actieknoppen: **Uitchecken** (de gebruikelijke
checkout van de branch van het verzoek, die ook de weergave sluit), **Openen in browser**, **Vernieuwen**
(**`F5`**) en **Review afronden**.

## Drie pagina's

Een segmentbesturing in de bovenrand wisselt van pagina: **Overzicht**, **Bestanden**, **Commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, of een klik).

### Overzicht

Het Overzicht is **het gesprek van het verzoek**, dezelfde tijdlijn en hetzelfde opmerkingenvak als een issue heeft
([Gesprekken](conversations.md)): eerst de beschrijving (*opende dit*), dan de opmerkingen, de
discussies bij code met hun diff-fragment, de ingediende reviews met hun oordeel (`goedgekeurd`, `wijzigingen
aangevraagd`, `opmerking geplaatst`, `ingetrokken`; op GitLab zijn de oordelen de goedkeuringen) en de gebeurtenissen,
oudste eerst, in het nieuwste venster met bovenaan **Oudere laden**. Het opmerkingenvak zit onder de tijdlijn vast
(**Nieuwe opmerking**): **Schrijven**, **Voorbeeld** en **Gesplitst**, **Annuleren** en **Verzenden**. **Een opmerking die je
hier plaatst, gaat direct naar de host** (er is niets in behandeling: conceptopmerkingen zijn die bij code, die met de
review worden verzonden), verschijnt meteen in de lijst en neemt haar plaats in zodra de host antwoordt; weigert de host
haar, dan opent het vak opnieuw met je tekst en de reden.

De toetsen zijn die van de gespreksweergave: **`Omhoog`** / **`Omlaag`** (en `Ctrl+J` / `Ctrl+K`) gaan tussen de items,
**`r`** antwoordt in de discussie van het geselecteerde item, **`Q`** citeert het in een nieuwe opmerking, **`c`** schrijft
een nieuwe opmerking, **`e`** / **`d`** bewerken en verwijderen je eigen opmerkingen (bij verwijderen wordt eerst gevraagd),
**`l`** of `Enter` opent de links van een item, **`g`** zijn diagram, **`y`** kopieert zijn adres, **`o`** opent het in de
browser, **`n`** gaat naar het volgende ongelezen item, **`v`** toont of verbergt gebeurtenissen, **`m`** laadt oudere
items, `Tab` gaat tussen de tijdlijn en het vak, en **Interne notitie** (`Alt+I`) is er op hosts die zulke notities
hebben. De knoppen van het geselecteerde item (**Antwoorden**, **Citeren**, **Bewerken**, **Verwijderen**, **Links**)
doen hetzelfde, en het palet heeft de rijen *Gesprek: ...* naast die van *Review: ...*. Voor een link wordt eerst om
bevestiging gevraagd voordat hij opent, zoals overal in de review.

Een discussie bij code toont zijn `pad:regel` als link (onderstreept, een echte knop). Een klik erop, **`G`** op de
geselecteerde discussie, of *Gesprek: Code van discussie openen* in het palet gaat naar de pagina **Bestanden**, opent dat
bestand en zet de cursor op de becommentarieerde regel (aan de oude of de nieuwe kant, zoals hij geschreven is) met de
discussie eronder open (een ingeklapte wordt uitgeklapt). **Esc**, de knop **Terug naar Overzicht** links van de paginatabs,
of *Review: Terug naar de discussie in het Overzicht* brengen je terug bij hetzelfde item van het Overzicht. Een discussie
waarvan de regel niet meer in de diff staat (verouderd) opent het bestand bij de dichtstbijzijnde regel en de statusregel
zegt dat; een discussie bij een bestand dat het verzoek niet meer wijzigt, meldt dat het bestand niet in de diff staat. In
een issue- of Radar-weergave van hetzelfde verzoek is er geen code om te openen: `G` antwoordt dat de review die heeft.

Waar een gesprekstoets en een reviewtoets samenkomen, houdt het gesprek de letter op deze pagina: `l` (labels), `m`
(mijlpaal), `Q` (Gerrit's wijziging opgeven), `e` / `d`, `r`. De labels, de mijlpaal en het opgeven blijven op de knoppen
van de kop en in het palet; `a` (toegewezenen), `p` (reviewers), `f` (**Review afronden**), `b` (**Uitchecken**), `P`, `D`,
`U` en `Alt+1` / `Alt+2` / `Alt+3` behouden hier hun betekenis. Zolang het vak het toetsenbord heeft, is elke toets die van
het vak, behalve `Alt+1` / `Alt+2` / `Alt+3`; `Esc` geeft het toetsenbord terug aan de tijdlijn en een tweede `Esc` sluit
de review.

Een opmerking die in een ander venster of via de CLI is geschreven, verschijnt hier vanzelf: elke schrijfactie van
GitPoulpi (dit venster inbegrepen) meldt het aan de sessie, en de vensters die hetzelfde verzoek tonen, lezen het opnieuw.

Het opmerkingenvak is de Markdown-composer die het deelt met [de gespreksweergave](conversations.md#schrijven)
(ook het vak van een antwoord op een discussie, een opmerking bij een regel en de samenvatting van **Review afronden**):
`Ctrl+Enter` (ook `Ctrl+S`, `Alt+Enter`) verzendt, en waar AI aan staat opent `Alt+E` **Schrijven met AI**
([het hoofdstuk over AI](ai.md#schrijfhulp-op-een-host)). Het voorbeeld toont de tekst zoals de host hem toont,
verwijzingen inbegrepen (`#7`, `!12`, `ABC-12`, commit-id's); een niet-verzonden tekst wordt per verzoek bewaard.

### Bestanden

Links de gewijzigde bestanden (statusletter, `+toegevoegd -verwijderd`, een teller `●n` van open discussies, een teller `✎n` van
je conceptopmerkingen) en rechts het verschil van het geselecteerde bestand. Het verschil komt van git: de head van het
verzoek wordt opgehaald naar `refs/gitpoulpi/pr/<n>` (nooit een branch, er wordt niets uitgecheckt) en vergeleken met de
merge base, dus de regelnummers in de marge zijn precies die waar opmerkingen mee werken.

- **`[ ]` voor een bestand**: markeer het als *bekeken* (klik, of druk op **`v`** / `Space` op de lijst). De titel telt ze.
  Bekeken-markeringen worden per repository bewaard en overleven een herstart.
- **Discussies** staan direct onder hun regel: eerste opmerking, reacties en de knoppen **Reageren** en
  **Oplossen** / **Heropenen**. Een opgeloste discussie klapt in tot één rij; **`z`** (of `Enter` erop, of een klik op de
  `▾`) klapt elke discussie in en uit.
- **Verouderde discussies**, en discussies bij regels die het verschil niet toont, staan na de hunks onder
  *Opmerkingen buiten de getoonde regels*, met een badge `verouderd` als de code eronder is gewijzigd.
- **`c`** plaatst een opmerking bij de regel waar de cursor staat (de kleine `+` aan het begin van de regel doet hetzelfde
  met de muis). **`Shift+Up` / `Shift+Down`** (of slepen over de regels) selecteren eerst meerdere regels en de
  opmerking omvat het bereik. GitLab neemt één regel per opmerking, en de weergave zegt dat in plaats van te selecteren.
- **`r`** reageert op de discussie onder de cursor, **`x`** lost haar op of heropent haar. Beide gaan **direct** naar de host.
- Toetsen: `Up` / `Down` / `PageUp` / `PageDown` / `Home` / `End` bewegen; `Tab`, `Right` en `Enter` gaan van de bestandslijst
  naar het verschil, `Left` en `Tab` terug; `]` en `[` springen naar het volgende en vorige bestand.

### De weergaveopties van het verschil

Het verschilpaneel van **Bestanden** en **Commits** is dat van de [Diff-weergave](staging-and-committing.md), met dezelfde balk op zijn eerste
rij en dezelfde onthouden keuzes (het is één instelling: wat je hier kiest is waarmee de Diff-weergave opent,
en omgekeerd):

- **Inline / Split**: één kolom, of voor en na naast elkaar. In Split staat een discussie, een conceptopmerking of een
  suggestie onder de helft waartoe ze behoort: een opmerking bij een verwijderde regel onder de linkerhelft, een opmerking bij een
  toegevoegde of ongewijzigde regel onder de rechterhelft. Klik op de helft die je bedoelt voordat je **`c`** indrukt op een rij die
  een verwijdering en een toevoeging toont; het toetsenbord plaatst opmerkingen aan de nieuwe kant.
- **Hunks**: alleen de wijzigingen met hun context, of het hele bestand met de wijzigingen ter plaatse. In het hele bestand staat een
  discussie bij een regel buiten de wijzigingen onder haar regel in plaats van in *Opmerkingen buiten de getoonde regels*.
  De tekst van het bestand wordt op de achtergrond gelezen; de hunks blijven in beeld met een regel die dat zegt tot hij er is (op
  de pagina Commits worden zo tot 40 bestanden van een commit gelezen).
- **WS**: toon of negeer wijzigingen die alleen in witruimte verschillen.
- **Wrap**: breek lange regels af.
- **Pijlen omhoog / omlaag**: de vorige en volgende hunk.
- **Map** (**`M`**): de minimap naast de schuifbalk, één instelling met die van de Diff-weergave. Klik of sleep erop om te
  scrollen.
- **Auto** (**`E`**, of *Review: Codering kiezen...*): de codering waarin het bestand wordt gelezen, zoals in de Diff-weergave:
  Automatisch detecteren of een van de coderingen uit de lijst, gekozen voor het bestand dat het paneel toont (op de pagina Commits het
  bestand onder de cursor). Beide kanten van het bestand worden op de achtergrond opnieuw in die codering gelezen, en
  de keuze wordt gedeeld met de Diff-weergave voor hetzelfde pad. Een bestand dat zo niet te lezen is, zegt waarom boven
  zijn verschil.
- **`⋯`**: als het paneel te smal is voor alle knoppen, staan de knoppen die niet passen achter een `⋯` aan het einde van
  de balk; zijn lijst voert elk uit alsof erop was geklikt. Hun toetsen en paletrijen werken zoals altijd.

De ruler van de schuifbalk toont waar dingen zijn: de wijzigingen (alleen het hele bestand, zoals in de Diff-weergave), een stip `●`
voor elke discussie, conceptopmerking en suggestie, en de cursor.

Lange regels worden aan de rand van het paneel afgekapt (zet **Wrap** aan om ze heel te zien) en scrollen zijwaarts: **`Right`**
en **`Left`** op het verschil (`Left` aan de linkerrand gaat terug naar de bestandslijst), het zijwaartse wiel of `Shift`
met het wiel, de schuifbalk langs de onderrand, of *Review: Diff naar links / rechts scrollen*.

### Tekst en regels selecteren

Waar een sleepbeweging begint, bepaalt wat ze selecteert. Een sleepbeweging die **in de code** begint, selecteert tekst, zoals in de Diff-weergave
(dubbelklikken selecteert een woord); **`Ctrl+C`** (of *Review: Geselecteerde tekst of regel kopiëren*) kopieert precies die
tekens, zonder regelnummers of markeringen, en kopieert zonder tekstselectie de regel onder de cursor. Een
sleepbeweging die **in de goot** begint (de regelnummers, de markering, de `+`) selecteert regels, om er zoals
eerder opmerkingen bij te plaatsen, en een dubbelklik in de goot start nog steeds een opmerking bij die regel.

De toetsen `n`, `p`, `m`, `w`, `s` van de Diff-weergave zijn die van de review zelf, dus in de review zijn de schakelaars hoofdletters:
**`V`** lay-out, **`H`** hunks of heel bestand, **`S`** witruimte, **`W`** regelafbreking, **`J`** / **`K`** volgende en
vorige hunk, **`E`** de codering, **`M`** de minimap. Elk heeft een paletrij (*Review: Lay-out inline / split
omschakelen*, ...).

Op het **Overzicht** worden het gesprek en het ✦ conceptblok op dezelfde manier geselecteerd: sleep over de tekst,
**`Ctrl+C`** kopieert hem (een link of een knop waarop je drukt houdt zijn actie, en een sleepbeweging die op een van beide begint
selecteert niets). Zonder selectie kopieert **`Ctrl+C`** (of *Gesprek: Tekst kopiëren*) het geselecteerde item zoals de host het
opslaat. De I-beam staat boven de tekst en de hand blijft boven knoppen en links. Een selectie volgt wat op het scherm staat:
scrollen verplaatst haar met haar tekst (ze blijft geselecteerd terwijl ze buiten beeld is), een verversing die de tekst herschrijft
laat haar vallen, en een afgebroken alinea kopieert als de rijen die hij toont.

De opmerkingen in de diff op de pagina **Bestanden** (een thread, je openstaande notities, de ✦ suggesties) worden ook geselecteerd: sleep
over hun woorden, **`Ctrl+C`** (of *Review: Geselecteerde tekst of regel kopiëren*) kopieert ze, **`Esc`** laat de selectie vallen. Een druk op een van
hun knoppen, of een gewone klik op de kop van een thread (hij vouwt), doet nog steeds iets; een sleepbeweging die op de kop begint selecteert zijn
woorden en laat de thread zoals hij is.

### Commits

Alleen de commits van het verzoek (niet de geschiedenis erachter), nieuwste eerst, en het verschil van de geselecteerde
(zijn wijzigingen ten opzichte van zijn parent), met al zijn bestanden onder elkaar en dezelfde weergaveopties.

## Terugkomen bij een review

Een review blijft open terwijl je in een ander tabblad werkt. Verlaat het tabblad met de review open (klik op een ander tabblad, `Ctrl+PageDown`, de paletopdracht **Repository: Volgend tabblad**) en terugkomen heropent haar op dezelfde aanvraag, dezelfde pagina (Overzicht, Bestanden of Commits), hetzelfde bestand en dezelfde regel, met het toetsenbord aan dezelfde kant. Hetzelfde geldt voor een review via de API over de Radar: toon het tabblad Radar opnieuw en de review is er weer overheen.

Een review is **in uitvoering** zolang haar pagina open is. Haar sluiten (`Esc`, het kruisje, **Review: Sluiten**), haar versturen (**Review afronden**, of een Gerrit-wijziging indienen of verlaten) of haar branch uitchecken beëindigt haar, en ze wordt niet heropend. GitPoulpi afsluiten met een open review telt als het tabblad verlaten: de volgende start heropent haar op dezelfde manier.

Is de aanvraag intussen gemerged of gesloten, of antwoordt de host niet, dan opent er geen pagina: de statusbalk zegt waarom en er wordt de volgende keer niets heropend. Openstaande opmerkingen, bekeken bestanden en het concept blijven bewaard zoals altijd (zie **Een review schrijven**). Een suggestierun die nog loopt als je vertrekt wordt niet overgenomen.

## Een review schrijven

Een opmerking bij code is een **concept**: ze wordt op je machine bewaard (in het statusbestand van de repository, dus ze overleeft een
herstart) en geteld in de kop. Er wordt niets verstuurd tot je de review afrondt. Een conceptopmerking staat
onder haar regel met de badge `Concept - regel 12` en twee knoppen: **`e`** (Bewerken) en **`d`** (Verwijderen).

**Review afronden** (**`f`**, de knop, of het palet) opent een klein formulier: een samenvatting (optioneel) en een
lijst **Oordeel**: *Opmerking*, *Goedkeuren*, *Wijzigingen aanvragen*. Alleen wat de host toestaat staat in de lijst: GitLab kent geen
*Wijzigingen aanvragen*. **Review indienen** (`Ctrl+S`) verstuurt de samenvatting, het oordeel en elke conceptopmerking in
één keer. **Review verwerpen...** gooit de conceptopmerkingen weg na een bevestiging.

Wat direct wordt verstuurd, zonder op de review te wachten: reacties, oplossen en heropenen, en opmerkingen in het
gesprek. Het opmerkingenvak zegt welke soort het is (*Deze opmerking wordt direct geplaatst* of *Deze opmerking wordt aan je
concept-review toegevoegd tot je die afrondt*). Als de host een reactie weigert, opent het vak opnieuw met je woorden.

Als het verzoek nieuwe commits kreeg nadat je conceptopmerkingen schreef, waarschuwt de weergave dat hun regels
verschoven kunnen zijn (GitHub en GitLab plaatsen een opmerking bij de commit die je las; een review van een oudere head wordt geweigerd).

## Gerrit

Een Gerrit-server wordt verbonden zoals de andere hosts (zie [Hosting en issuetrackers](hosting-integration.md): server-
URL, gebruikersnaam, HTTP-wachtwoord) en gekoppeld onder **PULL REQUESTS** aan een *project*. Zijn wijzigingen staan als pull
requests in de lijst (`#7`, chip `GR`) en worden in dezelfde weergave gereviewd, met de verschillen die Gerrit meebrengt:

- **Patchsets.** Een wijziging heeft genummerde patchsets (elke upload van de commit is er een). De kop heeft een
  keuzelijst **`Patchset N`** op de bovenrand, naast de paginatabs (**`P`**, of de paletrij *Review: Patchset
  kiezen...*). Hij begint bij de nieuwste; een andere kiezen leest de bestanden en code van die patchset, en de
  discussies die op andere patchsets zijn geschreven staan als *verouderd*. De review die je afrondt, wordt verstuurd op de patchset
  die wordt getoond.
- **Vergelijken met een andere patchset.** Een tweede keuzelijst links ervan, **`Vergelijken: basis`** (**`D`**,
  of *Review: Patchsets vergelijken...*), somt de basis en elke patchset op die ouder is dan de getoonde. Kies
  patchset 1 terwijl patchset 3 wordt getoond en de pagina Bestanden toont, en toont het verschil van, alleen wat
  patchset 3 sinds patchset 1 heeft gewijzigd (het eigen verschil van de server, dus een rebase op een nieuwer
  doel verschijnt niet als wijziging); de pagina Commits toont dat ene verschil. Beide patchsets worden opgehaald
  naar `refs/gitpoulpi/pr/<n>` en `refs/gitpoulpi/pr/<n>-ps<k>`. Discussies die op andere patchsets dan de
  getoonde zijn geschreven, zijn *verouderd*. Een oudere patchset om te lezen kiezen dan de vergeleken zet de
  vergelijking terug op de basis. De CLI-tegenhanger is `review show|fetch --compare-with K`.
- **Indienen en opgeven.** Als de server zegt dat de wijziging kan worden ingediend (de stemmen en checks die
  ze nodig heeft zijn er), komt er een knop **Indienen** bij de knoppen van de kop; **Opgeven** is er zolang je
  de wijziging mag opgeven. Beide (toetsen **`U`** en **`Q`**, palet *Review: Wijziging indienen...* /
  *Review: Wijziging opgeven...*) vragen eerst, in een popup die zegt dat de actie op de server gebeurt en
  **niet ongedaan kan worden gemaakt vanuit GitPoulpi** (een samengevoegde wijziging blijft samengevoegd; een
  opgegeven wijziging kan worden hersteld in de webinterface van Gerrit). Weigeren stuurt niets; bij succes
  leest de weergave de wijziging opnieuw. De CLI-tegenhangers zijn `gitpoulpi hosting review submit-change <n>`
  en `hosting review abandon <n> [--message M]`.
- **Stemmen in plaats van een oordeel.** **Review afronden** toont een keuzelijst per label waarop je mag stemmen - `Code-Review`
  (-2 tot +2) en `Verified` waar het project dat heeft - met alleen de waarden die de server **voor deze
  wijziging en jou** toestaat (een ontwikkelaar zonder het recht kan geen +2 kiezen). Hij begint bij je huidige stem; alleen een stem die je
  hebt gewijzigd wordt verstuurd. De samenvatting en de conceptopmerkingen gaan in dezelfde aanroep mee. Concepten die je in de Gerrit-
  webinterface hebt bewaard, worden met de review gepubliceerd.
- **Discussies** zijn de opmerkingenketens van Gerrit. Een discussie is opgelost als haar nieuwste opmerking `unresolved: false` zegt;
  **oplossen** plaatst een reactie `Done` die dat doet, **heropenen** een reactie `Reopened`. Reacties en oplossen gaan direct naar de server,
  zoals bij de andere hosts.
- **Pushen voor review.** Gerrit maakt een wijziging als je naar `refs/for/<branch>` pusht. Met de sectie PULL REQUESTS
  gekoppeld aan een Gerrit-project vragen de knop **`+`** van de sectie en de paletrij **Hosting: Push voor
  review...** om de doelbranch en pushen `HEAD` naar `refs/for/<branch>` (een topic kan op de opdrachtregel worden
  opgegeven). De push kan niet ongedaan worden gemaakt vanuit GitPoulpi (geef de wijziging in Gerrit op); het bericht zegt dat. De CLI-
  tegenhanger is `gitpoulpi hosting review push`.
- De metadataknoppen zijn voor Gerrit alleen-lezen (de reviewers worden getoond), er is geen oordeel *Wijzigingen aanvragen*
  (een stem van -1 of -2 is hetzelfde) en de checks-chip is het label `Verified`.

## ✦ AI-conceptreview

![Twee ✦ suggesties onder hun regels op de pagina Bestanden, elk met Accepteren, Bewerken en Negeren, naast een bestaande discussie](../images/review-ai.png)

Met AI aan (*Instellingen > AI*, zie [AI](ai.md)) heeft de kop een knop **`✦ Review opstellen...`** (ook **`s`**,
en de paletrij *Review: ✦ Review opstellen...*). Die opent de [reviewopties](#reviewopties) en
**`Enter`** daar vraagt het model. Dat is het enige wat het model aanroept: een review openen
doet dat nooit. Het model leest de titel en beschrijving van het verzoek, de opmerkingen die er al zijn (zodat het ze
niet herhaalt) en het verschil, en antwoordt met een **conceptreview**: de tekst om in te dienen, het oordeel dat het aanbeveelt (Goedkeuren, Wijzigingen aanvragen of Opmerking), een algemene samenvatting en een lijst voorgestelde opmerkingen. Een
lang verschil wordt voor het model ingekort - per bestand en in totaal - en het Overzicht zegt hoeveel bestanden zijn ingekort
of weggelaten. Terwijl het werkt, toont de statusbalk na even een spinner en **`Esc`** (of dezelfde knop)
annuleert het. De instructies *Codereview* van de [vier niveaus](ai.md#instructies) gelden, zodat een team of een
werkruimte kan zeggen hoe een goede review eruitziet.

### Reviewopties

`s` opent het dialoogvenster **✦ Reviewopties**, gevuld met de standaardwaarden van de repository (de globale waar de
repository er geen heeft). `Enter` (of de knop **Starten**) start de run met wat het venster toont; `Esc`, **Annuleren**
of het kruisje sluit het en vraagt niets.

| Bediening | Doet |
|---|---|
| **De agent in de repository laten draaien** | *Profielinstelling* (de werkmap van het AI-profiel beslist), *Ja* of *Nee*. Grijs, met de reden eronder, voor een HTTP-profiel (het heeft geen werkmap) en als het verzoek via de API zonder kloon wordt gereviewd. |
| **CLAUDE.md**, **AGENTS.md** | Geef het model dat bestand van de repository, gelezen op de branch waarop het verzoek mikt. Draait de agent in de repository, dan leest hij ze zelf en het venster zegt dat. |
| **Extra bestand** | Nog een bestand: een pad in de repository, een absoluut pad of een link naar een bestand van een codehost. **Bladeren...** (`Ctrl+B`, of `Enter` op de knop) opent een bestandskiezer die in de wortel van de repository begint (of naast het bestand dat al genoemd is); een bestand binnen de repository wordt geschreven als pad relatief aan de wortel, een bestand daarbuiten als absoluut pad. Typen in het veld werkt zoals voorheen. |
| **Geen extra bestand** | Geen extra bestand voor deze run, ook niet als de globale instelling of de repository er een noemt; een bestand typen of kiezen met **Bladeren...** vinkt het uit. Met **Opslaan als standaard van deze repository** wordt het de schakelaar *Geen extra bestand* van de repository. Een leeg veld telt hier als geen, omdat het dialoogvenster begint met wat de run zou erven. |
| **Opslaan als standaard van deze repository** | Bewaart de waarden ook voor de volgende reviews: in het statusbestand van de repository, of in `remote-reviews.toml` voor een niet-gekloonde repository. Alleen een waarde die van de globale afwijkt wordt bewaard, zodat de rest *Instellingen > AI* blijft volgen. |

Alles is bereikbaar met `Tab`, `Spatie` schakelt een selectievakje om en het padveld is een gewoon tekstveld. Dezelfde
vier instellingen staan in *Instellingen > AI* (*Codereview*) en in *Repositoryspecifiek > AI*; zie
[Reviewcontext](ai.md#reviewcontext). Wat het model niet kon krijgen (een extra bestand dat ontbreekt of
onleesbaar is) en elk bestand dat is ingekort om binnen de limieten te passen, staat op de conceptkaart.

Suggesties zijn **geen opmerkingen**. Ze staan op twee plaatsen:

- **Onder hun regel in Bestanden**, als spookopmerkingen gemarkeerd met `✦` en een ernst (`info`, `waarschuwing`, `probleem`) en
  drie knoppen: **Accepteren**, **Bewerken** en **Negeren**. Een suggestie over een regel die het verschil niet toont, wordt
  bewaard en nooit weggegooid: ze staat na de hunks onder *AI-suggesties over het hele bestand* en komt, eenmaal
  geaccepteerd, terecht op de eerste gewijzigde regel van het bestand met de woorden *(over regel N)* ervoor.
- **In het Overzicht**, in een blok boven het gesprek (alleen na een run): de samenvatting, daarna de lijst met dezelfde knoppen en **Alles accepteren** /
  **Alles negeren**.

**Accepteren** (`y`) maakt van een suggestie een gewone **conceptopmerking** (je kunt haar nog bewerken of verwijderen);
**Bewerken** (`w`) opent het opmerkingenvak met de woorden van het model eerst en accepteert wat je laat staan; **Negeren** (`n`)
gooit haar weg. **Alles accepteren** (`A`) vult nog steeds alleen de conceptlijst. Op de pagina Bestanden werken de toetsen op de
suggestie onder de cursor, op het Overzicht op de eerste van de lijst (`y` en `n` zijn weer *Link kopiëren* en *Volgende ongelezen* van het gesprek zodra de lijst leeg is). **AI-samenvatting gebruiken** (`i`, of de knop
in het formulier Review afronden) zet de samenvatting van het model in de reviewtekst; ze wordt nooit uit zichzelf ingevoegd.
**Er wordt niets naar de host gestuurd tot je de review indient.**

Het Overzicht toont het concept als kaart bovenaan dat blok, boven de suggesties: de oordeelchip, de reviewtekst en drie
knoppen.

- **✦ Concept gebruiken...** (`u`) accepteert elke openstaande suggestie als conceptopmerking en opent *Review afronden* al
  gevuld met de tekst en het oordeel van het concept. Je leest het nog steeds, wijzigt het en dient het zelf in; het
  formulier annuleren behoudt het concept. De limieten van de host blijven gelden: GitLab heeft geen *Wijzigingen aanvragen*, dus het formulier zegt dat en
  valt terug op *Opmerking*; bij Gerrit wordt het oordeel een Code-Review-stem van +1 of -1, alleen als de server die
  waarde voor jou toestaat (anders wordt geen stem gezet en zegt het formulier dat).
- **Tekst bewerken...** (`t`) wijzigt de reviewtekst van het concept voordat je het gebruikt.
- **Concept verwerpen** (`N`) gooit de tekst, het oordeel en de resterende suggesties weg.

Het concept wordt per verzoek bewaard, dus het is er nog als je de review sluit en weer opent. Is de head van het verzoek
verschoven sinds het model het schreef, dan zegt de kaart *Het verzoek is gewijzigd sinds dit concept werd geschreven.*
Het indienen van de review verbruikt het concept.

## Alles heeft een toets en een paletrij

| Actie | Toets | Palet |
|---|---|---|
| Overzicht / Bestanden / Commits | `Alt+1` / `Alt+2` / `Alt+3` | *Review: Overzicht tonen*, *Bestanden tonen*, *Commits tonen* |
| Opmerking bij de regel of het bereik | `c` | *Review: Opmerking bij regel plaatsen...* |
| Opmerking in het gesprek | `c` op Overzicht, `C` | *Review: Opmerking in het gesprek plaatsen...* (opent het vak van het Overzicht), *Gesprek: Nieuwe opmerking* |
| De code van een discussie openen (Overzicht) / terug naar de discussie | `G` / `Esc` | *Gesprek: Code van discussie openen*, *Review: Terug naar de discussie in het Overzicht* |
| Reageren op de discussie | `r` | *Review: Op discussie reageren...* |
| Oplossen of heropenen | `x` | *Review: Discussie oplossen of heropenen* |
| Een discussie in- of uitklappen | `z` | *Review: Discussie inklappen of uitklappen* |
| Een conceptopmerking bewerken / verwijderen | `e` / `d` | *Review: Conceptopmerking bewerken...*, *Conceptopmerking verwijderen* |
| Het bestand als bekeken markeren | `v` | *Review: Bestand als bekeken markeren* |
| Inline / Split | `V` | *Review: Lay-out inline / split omschakelen* |
| Heel bestand / alleen de hunks | `H` | *Review: Heel bestand / alleen hunks omschakelen* |
| Alleen-witruimtewijzigingen | `S` | *Review: Alleen-witruimtewijzigingen omschakelen* |
| Regelafbreking | `W` | *Review: Regelafbreking omschakelen* |
| Volgende / vorige hunk | `J` / `K` | *Review: Volgende hunk*, *Vorige hunk* |
| Volgend / vorig bestand | `]` / `[` | *Review: Volgend bestand*, *Vorig bestand* |
| De review afronden / verwerpen | `f` | *Review: Review afronden...*, *Concept-review verwerpen...* |
| Vernieuwen | `F5`, `Ctrl+R` | *Review: Vernieuwen* |
| Openen in browser / uitchecken | `o` / `b` | *Review: Openen in browser*, *Branch uitchecken* |
| Toegewezenen / reviewers / labels / mijlpaal | `a` / `p` / `l` / `m` | *Review: Toegewezenen bewerken...* en zijn drie broers |
| De patchset kiezen (Gerrit) | `P` | *Review: Patchset kiezen...* |
| Vergelijken met een oudere patchset (Gerrit) | `D` | *Review: Patchsets vergelijken...* |
| De wijziging indienen / opgeven (Gerrit) | `U` / `Q` | *Review: Wijziging indienen...*, *Review: Wijziging opgeven...* |
| ✦ Review opstellen... (AI aan; opent de reviewopties) | `s` | *Review: ✦ Review opstellen...* |
| Een suggestie accepteren / bewerken en accepteren / negeren | `y` / `w` / `n` | *Review: Suggestie accepteren*, *Suggestie bewerken en accepteren...*, *Suggestie negeren* |
| Elke suggestie accepteren / negeren | `A` / `N` | *Review: Alle suggesties accepteren*, *Alle suggesties negeren* |
| De AI-samenvatting in de reviewtekst gebruiken | `i` | *Review: AI-samenvatting in reviewtekst gebruiken* |
| Het ✦ concept gebruiken (vult Review afronden) | `u` | *Review: ✦ Het concept gebruiken...* |
| De tekst van het concept bewerken | `t` | *Review: De ✦ concepttekst bewerken...* |
| Het ✦ concept verwerpen | `N` | *Review: Het ✦ concept verwerpen* |
| Sluiten | `Esc` | *Review: Sluiten* |

Open het palet (`Ctrl+P`) terwijl de weergave open is: zijn rijen werken erop en zeggen waarom er nu niets gebeurt
("zet de cursor eerst op een coderegel").

## Reviewen zonder kloon

Een verzoek van een repository die niet op je machine staat, kan toch worden gereviewd. Vanuit de Radar vraagt **Reviewen**
(`e`) op zijn rij hoe (zie [De Radar](radar.md#een-verzoek-reviewen)); **Reviewen via de API** opent
dezelfde weergave in het Home-venster, over de Radar. Alles op deze pagina werkt hetzelfde, met deze
verschillen:

- De code (de gewijzigde bestanden, het verschil van elk bestand, de commits van het verzoek, het verschil van een commit, het hele bestand)
  wordt uit de API van de host gelezen, dus de eerste blik op een groot verzoek duurt even, en een bestand waarvoor de host
  geen verschil stuurt (een binair of een heel groot) zegt dat in plaats van er een te tonen.
- Je openstaande opmerkingen worden bewaard in `remote-reviews.toml` in de configuratiemap, gesorteerd onder de host
  en het project (`host/project`), in plaats van in de eigen status van een repository. Ze zijn er nog als je de review
  sluit en weer opent, en het versturen van de review verbruikt ze.
- Er is geen codeerkeuze (*Niet beschikbaar zonder kloon*) en geen **Uitchecken** (het zegt eerst de
  repository te klonen).
- `Esc` of het `✕` in de hoek verlaat de weergave en geeft de Radar terug. Het palet (`Ctrl+P`) bereikt de
  rijen van de weergave zoals in het tabblad van een repository.

Wil je liever de kloon houden, kies dan **Klonen en dan reviewen** (of *Radar: Klonen en reviewen...* in het palet):
het dialoogvenster Klonen opent op het gekoppelde account van het verzoek met zijn repository geselecteerd, en de review
opent in het tabblad van de nieuwe repository zodra het klonen klaar is.

## Vanaf de opdrachtregel en voor agents

Dezelfde review is beschikbaar als `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(en de MCP-tools die ervan zijn gemaakt): zie de [CLI-naslag](cli-reference.md). `threads` toont ook de
oordelen die al zijn ingediend (`reviews`). `suggest <n>` vraagt het model (AI aan) en drukt zijn `verdict`, `body`, samenvatting en
suggesties af als JSON zonder iets te plaatsen, zodat een script of een agent de opmerkingen kan kiezen die het aan
`submit --comment` doorgeeft.

Niets hiervan heeft een kloon nodig: `--project owner/name` (met `--id` als er meerdere integraties zijn ingesteld)
laat `show`, `threads`, `comment`, `reply`, `resolve`, `submit` en `suggest` los op een project waar de huidige
map niets van weet. `hosting review diff <n>` drukt het verschil af met zijn hunks (`--api` leest het van
de host, ook binnen een kloon), en `hosting file <path> --project P [--ref R]` leest een bestand op een revisie, bijvoorbeeld
de `CLAUDE.md` van het project. Beide lezen de API van de host: GitHub, GitLab, Gitea, Forgejo en Gerrit.

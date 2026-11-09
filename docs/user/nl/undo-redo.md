[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · [Français](../fr/undo-redo.md) · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · **Nederlands** · [Polski](../pl/undo-redo.md) · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Ongedaan maken en opnieuw

**Je kunt altijd terug.** Elke wijziging die GitPoulpi in je repository aanbrengt
wordt vastgelegd (standaard tot 100 acties, `general.undo_depth`), en ook elke
positie die een branch heeft gehad, inclusief verplaatsingen door andere
programma's. Als je een fout maakt, brengt een van deze het werk terug:

| Je wilt | Gebruik |
|---|---|
| het laatste terugdraaien | `Ctrl+Z` (opnieuw: `Ctrl+Y` of `Ctrl+Shift+Z`), of de werkbalkknoppen **Ongedaan** en **Opnieuw** |
| meerdere dingen tegelijk terugdraaien, of iets van lang geleden | de **Geschiedenis van ongedaan maken** (`Alt+Z`, de pijl naast Ongedaan, `Bewerken: Geschiedenis van ongedaan maken...`) |
| een branch, tag, stash, bestand of hunk terughalen die je een aantal acties geleden hebt verwijderd, **zonder ongedaan te maken wat je sindsdien deed** | **Alleen dit herstellen** in de Geschiedenis van ongedaan maken |
| een branch terugzetten naar waar hij was, ook als een ander programma hem verplaatste | de **Reflog** van de branch (refmenu `Reflog...`) |
| commits vinden die geen enkele branch nog bereikt | **Verloren werk** (`Bewerken: Verloren commits terughalen...`) |
| terugdraaien wat een shell of een **AI-agent** deed met branches, tags, stashes of HEAD | dezelfde `Ctrl+Z`: die wijzigingen staan ook in de geschiedenis, gemarkeerd als **extern** (zie [Wijzigingen buiten GitPoulpi](#wijzigingen-buiten-gitpoulpi)) |

Na elke actie die iets vernietigt en ongedaan kan worden gemaakt (verwerpen, een
bestand verwijderen, een branch of tag verwijderen, hard resetten, een stash
droppen, een remote verwijderen) zegt de statusbalk het: "verwerpen gedaan -
Ctrl+Z om ongedaan te maken", met de toets zoals jij hem hebt toegewezen. De
tooltip van Ongedaan zegt hoeveel acties je terug kunt.

`Ctrl+Z` maakt het laatste ongedaan dat GitPoulpi met je repository deed.
`Ctrl+Y` (of `Ctrl+Shift+Z`) zet het terug. De werkbalk heeft de knoppen
**Ongedaan** en **Opnieuw**, het palet heeft `Bewerken: Ongedaan maken` en
`Bewerken: Opnieuw uitvoeren`, en de tooltip van elke knop zegt wat hij zou terugdraaien
("Commit \"fix typo\" (a81b9e4) ongedaan maken").

![De werkbalk na Alle wijzigingen stagen: Ongedaan is verlicht en zijn tooltip noemt de handeling die hij zou terugdraaien](../images/undo-toolbar.png)

Opnieuw voert de handeling nooit opnieuw uit. Het herstelt de toestand die is
vastgelegd, zodat een opnieuw gedane commit exact dezelfde commit is, met
dezelfde id en hetzelfde tijdstip.

`Ctrl+Z` doet niets zolang een tekstveld het toetsenbord heeft (het
commit-bericht, het filtervak, de editor, die zijn eigen tekst-ongedaanmaken
houdt).

## De Geschiedenis van ongedaan maken

`Alt+Z` (opnieuw toe te wijzen), de pijl naast de knop Ongedaan of
`Bewerken: Geschiedenis van ongedaan maken...` opent een paneel met elke
vastgelegde actie, nieuwste eerst: een pictogram voor het soort actie (commit,
branch, tag, stash, reset, merge en verwanten, verwerpen, staging,
configuratie, wijziging van buitenaf), wat ze deed in jouw taal, hoe lang
geleden, en haar toestand (**afzonderlijk herstelbaar**, **geblokkeerd** met de
reden, of **ongedaan** en klaar om opnieuw te doen). Beslaan de items meerdere
dagen, dan begint elke dag met een dunne kop (Vandaag, Gisteren, daarna de
datum). De kop telt ze ("12 acties kunnen ongedaan worden gemaakt (tot 100)").

De acties die je ongedaan hebt gemaakt staan **boven** de nieuwste actie, onder
een scheidingslijn ("ongedaan gemaakt - Opnieuw brengt ze terug"), gedimd, met
degene die Opnieuw als eerste terugbrengt het dichtst bij de lijn. De selectie
begint op de nieuwste actie, niet daarop. De pijltoetsen bewegen (de koppen en
de scheidingslijn worden overgeslagen), `Tab` wisselt tussen de drie
weergaven, `Esc` sluit. Voor de geselecteerde rij:

- **Ongedaan maken tot hier** (`u`): maakt die actie en elke nieuwere ongedaan,
  de nieuwste eerst, na één bevestiging die ze allemaal opsomt. Een stap die de
  daemon riskant vindt, vraagt opnieuw voordat hij wordt uitgevoerd.
- **Alleen dit herstellen** (`r`): brengt terug wat die ene actie verwijderde en
  laat alles wat erna kwam staan. Een verwijderde branch of tag komt terug op de
  commit die hij had (is zijn naam inmiddels bezet, dan wordt om een andere
  gevraagd, met een vrije al ingevuld); een gedropte stash wordt bovenaan de
  stashlijst teruggezet; een verworpen of verwijderd bestand wordt op zijn plaats
  teruggeschreven als het sindsdien ongewijzigd is, anders ernaast als
  `naam.restored` (er wordt nooit iets overschreven); een verwijderde remote,
  upstream, ignore-regel of attributes-regel wordt opnieuw toegevoegd. Een
  reset, amend, rebase, merge of commit krijgt een branch
  `restore/<branch>-<sha>` op de oude punt - je huidige branch en bestanden
  verplaatsen nooit. De bevestiging toont precies wat er gaat gebeuren, en het
  herstel is een nieuwe actie: `Ctrl+Z` draait het terug.
- **Opnieuw tot hier** (`y`) op een ongedane rij.
- **main herstellen naar waar Commit het achterliet** (`o`, of `Alt+Shift+Z` /
  `Bewerken: De branch herstellen naar waar de geblokkeerde actie hem
  achterliet` van buiten het paneel): als een ander programma de branch
  verplaatste en Ongedaan maken blokkeerde, weet de reflog nog waar de actie hem
  achterliet.

Klik met rechts op een rij (of druk op `.`) voor dezelfde acties als menu.

![De Geschiedenis van ongedaan maken na het verwijderen van een branch en twee commits: de rij van de verwijderde branch is geselecteerd en kan afzonderlijk worden hersteld](../images/undo-history.png)

## De Reflog

Het refmenu van een lokale branch heeft **Reflog...**; `Bewerken: Reflog...` toont
die van HEAD, en de Geschiedenis van ongedaan maken heeft er een tabblad voor. De
Reflog toont de posities die de branch heeft gehad, nieuwste eerst: korte SHA, de
vastgelegde reden, hoe lang geleden, door wie - ook verplaatsingen in een shell.
Het bericht en de gewijzigde bestanden van de geselecteerde positie staan naast
de lijst.

De reflog wordt per 200 posities gelezen. Een langere gaat vanzelf door: de cursor
(of het scrollen) bij het eind van de lijst laadt de volgende pagina, en de laatste
rij toont **... Meer laden** zolang er een is. **Meer laden** (`m`, of `Reflog: Meer
laden` in het palet) laadt hem meteen en meldt "geladen posities: N" op de statusbalk.
Repositories met het reftable-formaat van git worden net als alle andere gelezen.

![De Reflog van main: elke positie die hij heeft gehad, nieuwste eerst, met de geselecteerde beschreven aan de rechterkant](../images/reflog.png)

**Tonen in het Log** (`s`), **Branch hier maken** (`b`) en
**`<branch>` herstellen naar deze positie** (`r`). Het herstel is ongedaan te
maken. Als de branch is uitgecheckt en je lokale wijzigingen hebt die hij niet
kan verplaatsen zonder ze te overschrijven, vraagt een keuzelijst wat ermee moet
gebeuren: mijn wijzigingen behouden, soft, mixed of hard. Hard gooit de
wijzigingen in je bestanden weg maar bewaart ze eerst, zodat `Ctrl+Z` ze nog
terugbrengt (het zegt dat in rood).

## Verloren werk

`Bewerken: Verloren commits terughalen...` (ook in het commit-menu van de Log)
vindt commits die geen branch, tag of stash bereikt: die een reflog nog noemt
(de commits van een verwijderde branch), die alleen een undo-pin vasthoudt, en
losse. De lijst verschijnt meteen en de tragere scan naar losse commits vult
zich aan terwijl een spinner draait; hij blokkeert de interface nooit.

![Verloren werk: de twee commits van een verwijderde branch (gevonden via de reflog) en losse commits](../images/lost-work.png)

**Branch hier maken** (`b`) herstelt een commit onder een naam naar keuze,
**Cherry-pick** (`c`) past hem toe op de huidige branch, **Tonen in het Log**
(`s`) werkt voor de commits die nog bereikbaar zijn.

## Vanuit het opdrachtenpalet

Elke rijactie van de drie weergaven is ook een rij in het opdrachtenpalet, zodat het paneel geen muis en geen te onthouden letter nodig heeft. Druk op `Ctrl+P` terwijl het paneel open is: het palet opent erboven en de rijen **Geschiedenis:**, **Reflog:** en **Verloren werk:** werken op de geselecteerde rij van de weergave die op het scherm staat.

| Weergave | Rijen in het palet |
|---|---|
| Geschiedenis van ongedaan maken | `Geschiedenis: Ongedaan maken tot hier...`, `Opnieuw tot hier...`, `Alleen dit herstellen...`, `De branch herstellen naar waar hij was...`, `Vernieuwen`, `Sluiten` |
| Reflog | `Reflog: Tonen in het Log`, `Branch hier maken...`, `Herstellen naar deze positie...`, `Meer laden`, `Vernieuwen` |
| Verloren werk | `Verloren werk: Tonen in het Log`, `Branch hier maken...`, `Cherry-pick`, `Opnieuw zoeken` |

De rijen zijn grijs (met de reden) zolang hun weergave niet open is; de drie openers, `Bewerken: Geschiedenis van ongedaan maken...`, `Bewerken: Reflog...` en `Bewerken: Verloren commits terughalen...`, schakelen een open paneel ook om naar hun weergave. Een rij die de selectie niet kan uitvoeren zegt in de statusbalk waarom, in plaats van niets te doen: "deze actie is al ongedaan gemaakt: gebruik Opnieuw tot hier", de reden waarom een item niet afzonderlijk kan worden hersteld, "alleen een branch of HEAD kan worden teruggezet", enzovoort. De rijen met `...` openen dezelfde bevestiging of naamprompt als de knop.

## Wat ongedaan kan worden gemaakt

| Gebied | Handelingen |
|---|---|
| Commits | commit, amend, reword, de AI-commit-composer (al zijn commits als één stap) |
| Branches en tags | maken, verwijderen, hernoemen, de upstream instellen of wissen, een trackingbranch maken, een tag maken / verwijderen / annoteren, fast-forward, een pull request als branch uitchecken |
| Rondbewegen | checkout van een branch of een commit (ook een detached HEAD) |
| Geschiedenis | merge (fast-forward of merge-commit), rebase (gewoon en interactief, zodra hij klaar is), cherry-pick, revert, reset (soft, mixed en hard), het lokale deel van een pull |
| Lokale wijzigingen | stage en unstage (bestand, map, hunk, regels, alles), verwerpen (bestand, map, hunk, regels, een bestandscheckout), een patch toepassen, **een bestand van schijf verwijderen**, **negeren** (bestand, extensie of map: de regel in het `.gitignore` dat je koos), **als binair of tekst behandelen** (de regel in `.gitattributes`) |
| Stashes | push, apply, pop, drop |
| Remotes en configuratie | een remote toevoegen, verwijderen, hernoemen, de URL of push-URL wijzigen, Git LFS track en untrack |
| Op een server | **een branch of tag op een remote verwijderen** (via het menu van de ref, het palet, `branch delete-remote`, `tag delete-remote`, `tag delete --both`, of een workflow-afronding die de remote branch verwijdert): zie hieronder |

Terwijl een merge, cherry-pick of revert op conflicten is gestopt, zijn
**Opslaan & als opgelost markeren**, stagen en unstagen ook ongedaan te maken
(elk wordt teruggedraaid vóór degene eronder), en is **Merge afbreken** (of
cherry-pick, revert) ongedaan te maken: het ongedaan maken brengt de hele
gestopte handeling terug, met haar conflicten (de drie versies van elk onopgelost
bestand), de al opgeloste en gestagede bestanden, de conflictmarkeringen in de
werkmap en het merge-bericht; opnieuw breekt weer af. Staging en oplossingen zijn
niet meer ongedaan te maken zodra de handeling eindigt (gecommit of afgebroken).
*Bestand unstagen* en *Alle wijzigingen unstagen* raken nooit een bestand dat nog in conflict is: het
heeft geen gestagede wijziging om terug te draaien.

Alles wat een verwerping of een harde reset weggooit wordt eerst bewaard, dus
ongedaan maken brengt de exacte bytes terug, inclusief untracked bestanden,
uitvoerbaarheidsbits en symbolische links. Een gedropte stash ongedaan maken
geeft dezelfde stash terug, met dezelfde commit-id (hij komt bovenaan de
stash-lijst, als `stash@{0}`, niet op zijn oude plek). Het verwijderen van een
remote ongedaan maken geeft zijn remote-trackingbranches, zijn
`origin/HEAD`-alias en de upstream-instellingen van de branches die hem volgden
terug.

### Een branch of tag op een remote verwijderen

Vóór het verwijderen vraagt GitPoulpi de server wat de branch of tag op dat moment
bevat en legt dat vast: de remote, de naam en de exacte commit (bij een
geannoteerde tag het tag-object zelf, zodat zijn annotatie behouden blijft). Het
verwijderen gebeurt dan alleen als de server nog die waarde bevat.

**Ongedaan maken pusht hem terug** onder dezelfde naam, op dezelfde commit, zonder ooit
te forceren. Omdat dat een wijziging is op een server die anderen zien, vraagt Ongedaan maken
**altijd eerst** en zegt dat ("Dit ongedaan maken maakt branch topic opnieuw aan op origin op
3f2a1c0..."); de stappen van het item zeggen het ook (`gitpoulpi undo --dry-run`). Het gebruikt
dezelfde inloggegevens als een push. De remote-trackingbranch komt mee terug.
**Opnieuw** verwijdert hem weer, met dezelfde veiligheid.

Ongedaan maken weigert, zonder iets te wijzigen, wanneer:

- de branch of tag sindsdien **opnieuw op de remote is aangemaakt**, of is verplaatst: hij
  staat op een andere commit dan de vastgelegde, en pushen zou het werk van
  iemand anders overschrijven (de weigering noemt beide commits);
- de commit (of het tag-object) **niet meer in je repository staat** (hij is door
  garbage collection opgeruimd): het ongedaan maken kan niet.

"Lokaal en op de remote verwijderen" is **één item**: Ongedaan maken herstelt de lokale en
de remote branch of tag samen, Opnieuw verwijdert beide weer. Alleen de remote branch
verwijderen laat de lokale ongemoeid; het ongedaan maken doet hetzelfde. Een
workflow-afronding die ook de remote branch verwijdert, is als geheel ongedaan te maken.

## Wat niet ongedaan kan worden gemaakt, en waarom

- **Het meeste op een server**: push, een tag pushen, LFS push, lock en unlock,
  een pull request openen. Die toestand wordt met anderen gedeeld; een push
  ongedaan maken zou een force-push of een verwijdering aan hun kant zijn, en dat doet
  GitPoulpi niet voor je. (Een remote branch of tag verwijderen is de uitzondering: het
  ongedaan maken zet alleen terug wat de server bevatte, en weigert als daar iets is veranderd.)
  Ongedaan maken blokkeert ook nooit vanwege zo'n handeling.
- **Fetch**: het verplaatst alleen remote-trackingrefs, die de server
  weerspiegelen. De volgende fetch zou het opnieuw doen.
- **Een rebase afbreken**: ongedaan maken wordt er niet voor aangeboden (een
  rebase verplaatst branches en houdt zijn eigen toestand; het afbreken is zelf
  de weg terug). Een **merge, cherry-pick of revert** afbreken wordt *wel*
  vastgelegd, zie hierboven.
- **Een worktree maken**, **een patchbestand maken**, clone, init,
  hostingacties (pull requests, issues) en AI-aanroepen: ze maken geen deel uit
  van de geschiedenis van de repository.
- **Een nieuw leeg bestand maken** vanuit het bestandsmenu, het automatisch
  verwijderen van `*.orig`-bestanden na een merge en een bestand opslaan in de
  ingebouwde editor worden niet vastgelegd. Het wijzigen van de **SSH-opdracht**
  (`core.sshCommand`) ook niet, en evenmin het wijzigen van de **gedeeltelijke checkout**
  (er gaat niets verloren; *Uitschakelen* brengt elk bestand terug).
- **Schrijven naar `.git/info/exclude` of `.git/info/attributes`** (de keuze "alleen deze kloon" in de lijsten voor negeren en als binair/tekst behandelen): die bestanden staan buiten de werkmap en worden daarom niet vastgelegd.
- **Bewerkingen van bestanden in de werkmap buiten GitPoulpi** (een editor, een
  script, een AI-agent die code schrijft) worden nooit vastgelegd: er bestaat
  geen momentopname van, dus een `git reset --hard` of `git checkout -- file` in
  een shell verliest niet-gecommitte bewerkingen voorgoed. Wat van het werk van
  een shell *wel* wordt vastgelegd, is wat in refs leeft: zie de volgende
  sectie.

## Wijzigingen buiten GitPoulpi

Werk je met een AI-agent (of een shell naast GitPoulpi), dan gebeurt het meeste
git-werk buiten het venster: commits, nieuwe branches en worktrees,
verwijderde branches, resets, rebases, stashes. GitPoulpi ziet dat ook. De
daemon houdt de laatste refs bij die hij zag (lokale branches, tags, HEAD, de
stashlijst, de worktreelijst) en legt, telkens als de repository veranderde
zonder dat GitPoulpi het deed, **één item per samenhangende wijziging** vast
in de Geschiedenis van ongedaan maken, gemarkeerd als **extern**, met de reden
die git logde (`commit: fix login`, `rebase (finish)`, `reset: moving to
HEAD~1`, `checkout: moving from a to b`) en de naam die git vastlegde voor wie
het deed. GitPoulpi kan een shell niet van een AI-agent onderscheiden en
gokt nooit; het item zegt alleen dat het buiten GitPoulpi gebeurde.

- **Een reeks is één item.** Twintig commits tussen twee blikken zijn één item,
  van de eerste waarde tot de laatste; een rebase is één item, vastgelegd als
  hij eindigt (er wordt niets vastgelegd zolang een rebase of merge stilstaat).
- **Verwijderde refs worden vastgepind.** Een branch, tag of stash die
  verdween blijft in leven onder `refs/gitpoulpi/undo/` op zijn oude commit
  voordat iets hem kan opruimen, dus **Alleen dit herstellen** (of `Ctrl+Z`)
  brengt de branch terug die de agent verwijderde, ook na de reflog en een
  garbage collection.
- **Terwijl GitPoulpi gesloten was.** De momentopname wordt naast het journaal
  bewaard (`.git/gitpoulpi/refs-snapshot.json`), dus wat gebeurde terwijl geen
  GitPoulpi draaide, wordt bij de volgende start gevonden en gemarkeerd als
  "terwijl GitPoulpi gesloten was".
- **`Ctrl+Z` werkt erop zoals op elk item**: de refs gaan terug, een
  verwijderde branch wordt opnieuw gemaakt, een gedropte stash wordt opnieuw
  opgeslagen, een checkout van buitenaf checkt de oude branch weer uit. Een
  uitgecheckte branch gaat terug zoals `git reset --keep`: bestanden volgen, en
  zit een van je lokale wijzigingen in de weg, dan **weigert het en verandert
  niets**. Opnieuw past het weer toe. **Alleen dit herstellen** werkt ook: een
  verwijderde branch komt terug en de latere acties blijven staan; een
  verplaatste branch komt terug als `restore/<branch>-<sha>`.
- **De statusbalk zegt het één keer**: "wijziging van buitenaf vastgelegd -
  Ctrl+Z om ongedaan te maken" als een nieuw extern item binnenkomt en het
  venster op de voorgrond staat.
- Omdat de wijziging van buitenaf nu het nieuwste item is, verdwijnt de oude
  weigering "geblokkeerd door een wijziging van buitenaf" grotendeels: je maakt
  eerst de wijziging van buitenaf ongedaan.

Niet vastgelegd worden: verplaatsingen van remote-trackingbranches die alleen
een fetch deed (ze weerspiegelen de server), de eigen verborgen refs van
GitPoulpi, een worktree die zonder enige branchwijziging werd toegevoegd of
verwijderd (het item van de branch die hij draagt zegt "worktree <pad>
toegevoegd"; ongedaan maken verplaatst alleen refs en verwijdert de worktree
niet), en alles binnen bestanden. `general.journal_outside = false`
(Instellingen > Algemeen, "Wijzigingen buiten GitPoulpi vastleggen") schakelt
dit uit: er wordt niets van buitenaf vastgelegd en een wijziging van buitenaf
blokkeert het nieuwste item, zoals voorheen.

## Wanneer ongedaan maken weigert

Voordat het handelt, controleert GitPoulpi dat alles wat de handeling raakte nog
precies is zoals de handeling het achterliet. Zo niet, dan **verandert het niets**
en zegt het wat er verschoven is:

- `main` staat op 3f2a1c0, niet op a81b9e4 waar Commit het achterliet (een
  commit gemaakt in een shell, bijvoorbeeld);
- een bestand, de index, een stash of een configuratiesectie is sindsdien gewijzigd;
- een merge, rebase of cherry-pick is bezig;
- de handeling is nooit voltooid omdat GitPoulpi werd gestopt terwijl hij liep
  (het item is gemarkeerd als *onderbroken*, wordt nooit automatisch ongedaan
  gemaakt, en `gitpoulpi undo --list` toont wat hij op het punt stond te
  wijzigen, zodat je het met de hand kunt herstellen).

Ongedaan maken is strikt last-in, first-out: een geblokkeerd nieuwste item
blokkeert ook de oudere, want een oudere handeling ongedaan maken onder een
nieuwere die nog geldt, zou gokken zijn. Zet je zelf de dingen terug, dan wordt
het item weer geldig. Wijzigingen die niets raken wat een item raakte (een
commit vanuit een shell op een andere branch, een fetch) blokkeren niets.

### Een geblokkeerd item dat je niet kunt opruimen: Vergeten

Als het nieuwste item definitief vastzit (de repository is buiten GitPoulpi
verder gegaan, of het item is nooit afgerond), kan er ook niets onder worden
ongedaan gemaakt. De tooltip van Ongedaan en de weigering zeggen dat dan en
noemen de uitweg: **Bewerken: Geblokkeerd ongedaan-maken-item vergeten** in het
palet (het vraagt eerst), of `gitpoulpi undo-forget` op de opdrachtregel.
Vergeten verwijdert alleen dat ene item uit het journaal; de repository wordt
nooit aangeraakt, en het item kan niet meer ongedaan of opnieuw worden gemaakt.
Het item eronder wordt dan het nieuwste.

Twee gevallen lossen zichzelf op:

- een merge of rebase in conflict die je met gewone git afrondt: het item wordt
  gesloten de volgende keer dat GitPoulpi de repositorytoestand leest;
- de bestanden van een stash pop of apply in conflict die je oplost met
  **Opslaan & als opgelost markeren**: Ongedaan maken zet het conflict terug
  zolang het opgeloste bestand nog gestaged en niet gecommit is.

## Bevestigingen

Ongedaan maken vraagt alleen als het riskant is:

- de commit (of merge, rebase, cherry-pick, revert) die je terugdraait staat al
  op de upstream van de branch: het ongedaan maken herschrijft geschiedenis die
  je hebt gepubliceerd, en je zou het resultaat moeten force-pushen;
- je hebt niet-gecommitte wijzigingen en het ongedaan maken verplaatst de
  werkmap: je wijzigingen blijven behouden, maar het ongedaan maken stopt zonder
  iets te wijzigen als een ervan in een bestand zit dat het moet herschrijven;
- het item verwijderde een branch of tag op een remote: het ongedaan maken **pusht hem
  terug naar de server** (zie hierboven), dus het vraagt altijd, en de Opnieuw die hem daar
  weer verwijdert ook.

Opnieuw, en elke ongedaanmaking van een alleen-ref-verplaatsing, vraagt nooit:
de andere richting is één toets verderop.

## Hoe ver terug

`general.undo_depth` (Instellingen > Algemeen) is het aantal bewaarde handelingen.
De standaard is **100**; `0` betekent onbeperkt. Verlagen vergeet meteen de
oudste items. Een nieuwe handeling starten na een ongedaanmaking gooit weg wat
je opnieuw had kunnen doen, zoals in elke editor.

De geschiedenis wordt per worktree bewaard (een gekoppelde worktree heeft een
eigen), in de git-map (`.git/gitpoulpi/journal.json`), zodat ze het sluiten van
het programma overleeft en nooit deel uitmaakt van `git status` of een commit.

## Uitschakelen

`general.undo_enabled = false` (Instellingen > Algemeen, "Ongedaan maken
ingeschakeld") schakelt de hele functie uit (en daarmee ook het vastleggen van wijzigingen van buitenaf). Er wordt niets vastgelegd of
bewaard, dus stagen, verwerpen en committen zijn iets sneller. De knoppen
Ongedaan en Opnieuw verdwijnen en `Ctrl+Z` zegt "Ongedaan maken staat uit".

**Uitschakelen wist de geschiedenis**: het journaalbestand en de hieronder
beschreven verborgen refs worden verwijderd, de eerste keer dat een verzoek de
daemon bereikt terwijl het uit staat. Weer inschakelen begint met een lege
geschiedenis.

## Wat wordt opgeslagen

Om zeker te zijn dat ongedaan maken nooit gegevens kan verliezen, wordt elke
commit, tagobject, stash of snapshot die een item nodig kan hebben in leven
gehouden door een verborgen ref onder `refs/gitpoulpi/undo/` zolang het item in
de geschiedenis staat. Ze worden uitgesloten van de eigen lijsten van
GitPoulpi, maar een `git log --all` of `git for-each-ref` in een shell toont
ze, en `git push --mirror` zou ze pushen (gebruik in plaats daarvan
`git push --all`). Ze worden verwijderd als hun item uit de geschiedenis valt of
als ongedaan maken wordt uitgeschakeld. Verder wordt nooit iets verwijderd.

## Vanaf de opdrachtregel en MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` beschrijft de stappen en verandert niets.
- `--list` drukt het journaal af, nieuwste laatst (`--all` voegt items toe die
  niet meer opnieuw te doen zijn maar nog bewaard worden), met de id en
  toestand van elk item.
- `--expect <id>` laat de aanroep als *verouderd* weigeren tenzij dat item nog
  het nieuwste is: een script (of een tweede venster) maakt nooit iets anders
  ongedaan dan waar het naar keek.
- `undo-forget` laat het geblokkeerde nieuwste item uit het journaal vallen
  (`--redo`: het nieuwste opnieuw-item). Het weigert als dat item niet geblokkeerd
  is, en eindigt met **3** zonder `--yes`. Voor MCP is het een mutatie, maar geen
  destructieve: de repository blijft onaangeroerd.
- Afsluitcode **0** gedaan, **2** geweigerd (de JSON noemt de reden, bijvoorbeeld
  `RemoteRefMoved` wanneer een verwijderde remote branch opnieuw is aangemaakt), **3**
  riskant en vereist `--yes` (het ongedaan maken van het verwijderen van een remote branch of tag,
  wat naar de server pusht).

Teruggaan zonder alles erna ongedaan te maken:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (ook: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (ook: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` toont bij elk item wat **alleen dit herstellen** zou kunnen
  terugbrengen (`restore.items`) of waarom niets (`restore.not_restorable`), en
  wie het deed: `"source": "gitpoulpi"`, of `"source": "outside"` met
  `outside.message` (de reden van git) en `outside.committer` voor een wijziging
  van een shell of een AI-agent. `undo`, `redo` en `undo-restore` nemen externe
  items zoals elk ander.
- `undo-restore <id>` brengt terug wat dat ene item verwijderde (een
  verwijderde branch of tag, een gedropte stash, een verworpen of verwijderd
  bestand, een verwijderde remote met zijn trackingbranches en upstreams, een
  ignore-regel) en laat elke latere bewerking staan. Het overschrijft nooit: een
  bestand dat sindsdien is gewijzigd krijgt de bewaarde versie ernaast als
  `<naam>.restored`; een bezette branchnaam is een weigering (afsluitcode 2) die
  een vrije voorstelt, geef die mee met `--name`; na een reset, amend of rebase
  maakt het een nieuwe branch op de oude punt en verplaatst het nooit jouw
  branch. Het herstel is één journaalitem, dus `gitpoulpi undo` draait het terug.
- `reflog` toont waar `HEAD` of een branch was, met de reden van git, ook wat een
  shell deed. `reflog-restore` verplaatst een branch (of `HEAD`) naar een van die
  posities; een uitgecheckte branch met niet-gecommitte wijzigingen vraagt om
  `--mode`. Het wordt ook gejournaald.
- `lost-commits` toont commits die geen branch, tag, remote of stash bereikt.
- Als een verplaatsing in een shell het nieuwste item blokkeert, bevat de
  weigering de reflog-positie die bevat wat het item verwacht
  (`reason.RefMoved.restore`), om aan `reflog-restore` te geven.

Er bestaan twee opdrachten zodat de bovenstaande acties vanaf elk oppervlak
ongedaan te maken zijn: `gitpoulpi rm-file <paden>` (bestanden uit de werkmap
verwijderen) en `gitpoulpi ignore <patroon> [--file <pad/.gitignore>]` (een
regel aan een `.gitignore` toevoegen; er gebeurt niets als de regel er al
staat; ongedaan maken zet het bestand terug zoals het was, of verwijdert het als
de opdracht het maakte, en weigert als het bestand sindsdien is bewerkt).

De MCP-server heeft één tool per opdracht, dus `undo`, `redo`, `rm-file` en
`ignore` zijn ook tools. `undo`, `redo` en `rm-file` zijn als destructief
ingedeeld (afgeschermd zoals `reset`, zie [MCP en agents](mcp-and-agents.md));
`ignore` is een gewone mutatie. Het journaal is een bestand in de repository,
dus `gitpoulpi undo` in een shell draait terug wat het venster deed, en het
venster kan ongedaan maken wat een script deed.

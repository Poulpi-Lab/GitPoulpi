[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · [Français](../fr/undo-redo.md) · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · [Polski](../pl/undo-redo.md) · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · **Svenska** · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Ångra och gör om

**Du kan alltid gå tillbaka.** Varje ändring GitPoulpi gör i ditt repository
registreras (upp till 100 åtgärder som standard, `general.undo_depth`), och det
gör också varje position en gren har haft, även flyttar som andra program har
gjort. Om du gör ett misstag kan något av detta få tillbaka arbetet:

| Du vill | Använd |
|---|---|
| ta tillbaka det senaste | `Ctrl+Z` (gör om: `Ctrl+Y` eller `Ctrl+Shift+Z`), eller verktygsfältets knappar **Ångra** och **Gör om** |
| ta tillbaka flera saker på en gång, eller en sak från långt tillbaka | **Ångrahistoriken** (`Alt+Z`, pilen bredvid Ångra, `Redigera: Ångrahistorik...`) |
| få tillbaka en gren, tag, stash, fil eller hunk du tog bort för flera åtgärder sedan, **utan att ångra det du gjort sedan dess** | **Återställ bara den här** i Ångrahistoriken |
| flytta en gren tillbaka dit den var, även om ett annat program flyttade den | grenens **Reflog** (refmenyns **Reflog...**) |
| hitta commits som ingen gren når längre | **Förlorat arbete** (`Redigera: Rädda förlorade commits...`) |
| ta tillbaka det ett skal eller en **AI-agent** gjorde med grenar, taggar, stashar eller HEAD | samma `Ctrl+Z`: de ändringarna finns också i historiken, markerade **utanför** (se [Ändringar gjorda utanför GitPoulpi](#ändringar-gjorda-utanför-gitpoulpi)) |

Efter varje åtgärd som förstör något och går att ångra (förkasta, ta bort en
fil, ta bort en gren eller tag, hard reset, droppa en stash, ta bort en remote)
säger statusraden det: ”förkasta ändringar klart - Ctrl+Z för att ångra”, med
tangenten så som du har bundit den. Verktygstipset för Ångra säger hur många
åtgärder du kan gå tillbaka.

`Ctrl+Z` tar tillbaka det sista GitPoulpi gjorde med ditt repository. `Ctrl+Y`
(eller `Ctrl+Shift+Z`) sätter tillbaka det. Verktygsfältet har knapparna
**Ångra** och **Gör om**, paletten har `Redigera: Ångra` och `Redigera: Gör om`,
och verktygstipset på varje knapp säger vad den skulle ta tillbaka (”Ångra
Commit 'fix typo' (a81b9e4)”).

![Verktygsfältet efter Stage alla ändringar: Ångra är tänd och dess verktygstips namnger operationen den skulle ta tillbaka](../images/undo-toolbar.png)

Gör om kör aldrig operationen igen. Den återställer det tillstånd som
registrerades, så en omgjord commit är exakt samma commit, med samma id och
samma tidsstämpel.

`Ctrl+Z` gör ingenting medan ett textfält har tangentbordsfokus
(commit-meddelandet, filterrutan och redigeraren, som har sin egen textångra).

## Ångrahistoriken

`Alt+Z` (kan bindas om), pilen bredvid knappen Ångra eller `Redigera:
Ångrahistorik...` öppnar en panel med varje registrerad åtgärd, nyaste först: en
ikon för slags åtgärd (commit, gren, tag, stash, reset, merge och liknande,
förkasta, staging, konfiguration, ändring utanför), vad den gjorde på ditt språk,
hur länge sedan, och dess tillstånd (**återställbar**, **blockerad** med skälet,
eller **ångrad** och redo att göras om). När posterna sträcker sig över flera
dagar inleds varje dag av en tunn rubrik (I dag, I går, sedan datumet). Rubriken
räknar dem (”12 åtgärder kan ångras (upp till 100)”).

De åtgärder du har ångrat ligger **ovanför** den nyaste åtgärden, under en
avdelare (”ångrade - Gör om hämtar tillbaka dem”), dämpade, med den som Gör om
hämtar tillbaka härnäst närmast avdelaren. Markeringen börjar på den nyaste
åtgärden, inte på dem. Piltangenterna flyttar (rubrikerna och avdelaren hoppas
över), `Tab` växlar mellan de tre vyerna, `Esc` stänger. För den markerade raden:

- **Ångra hit** (`u`): ångrar den åtgärden och varje nyare, nyaste först, efter en
  enda bekräftelse som listar dem alla. Ett steg som daemonen bedömer som
  riskabelt frågar igen innan det körs.
- **Återställ bara den här** (`r`): ger tillbaka det som den enda åtgärden tog
  bort och behåller allt som gjorts efter den. En borttagen gren eller tag kommer
  tillbaka på den commit den hade (är namnet upptaget nu får du frågan om ett
  annat, med ett ledigt förifyllt); en droppad stash läggs tillbaka överst i
  stash-listan; en förkastad eller borttagen fil skrivs tillbaka på sin plats om
  den är orörd sedan dess, annars bredvid som `name.restored` (inget skrivs
  någonsin över); en borttagen remote, upstream, ignoreringsrad eller
  attributes-rad läggs till igen. En reset, amend, rebase, merge eller commit får
  en gren `restore/<gren>-<sha>` vid den gamla spetsen - din nuvarande gren och
  dina filer flyttas aldrig. Bekräftelsen visar exakt vad som kommer att göras,
  och återställningen är en ny åtgärd: `Ctrl+Z` tar tillbaka den.
- **Gör om hit** (`y`) på en ångrad rad.
- **Återställ main till där Commit lämnade den** (`o`, eller `Alt+Shift+Z` /
  `Redigera: Återställ grenen till där den blockerade åtgärden lämnade den`
  utanför panelen): när ett annat program har flyttat grenen och blockerat Ångra
  vet reflog fortfarande var åtgärden lämnade den.

Högerklicka på en rad (eller tryck `.`) för samma åtgärder som en meny.

![Ångrahistoriken efter att en gren tagits bort och två commits gjorts: den borttagna grenens rad är markerad och kan återställas för sig](../images/undo-history.png)

## Reflog

Refmenyn för en lokal gren har **Reflog...**; `Redigera: Reflog...` visar HEAD:s,
och Ångrahistoriken har en flik för den. Den listar de positioner grenen har
haft, nyaste först: kort SHA, den registrerade orsaken, hur länge sedan, vem -
även flyttar gjorda i ett skal. Den markerade positionens meddelande och ändrade
filer visas bredvid listan.

Reflog läses 200 positioner åt gången. En längre reflog fortsätter av sig själv:
flyttar du markören (eller rullar) nära slutet av listan hämtas nästa sida, och
sista raden lyder **... Ladda fler** så länge det finns en. **Ladda fler** (`m`,
eller `Reflog: Ladda fler` i paletten) hämtar den direkt och skriver ”inlästa
positioner: N” på statusraden. Repositories som använder gits reftable-format
läses som alla andra.

![Reflog för main: varje position den har haft, nyaste först, med den markerade beskriven till höger](../images/reflog.png)

**Visa i Log** (`s`), **Skapa gren här** (`b`) och **Återställ `<gren>` till den
här positionen** (`r`). Återställningen går att ångra. När grenen är utcheckad och
du har lokala ändringar som den inte kan flytta utan att skriva över, frågar en
rullgardin vad som ska hända med dem: behåll mina ändringar, soft, mixed eller
hard. Hard förkastar ändringarna i dina filer men sparar dem först, så `Ctrl+Z`
ger ändå tillbaka dem (det står med rött).

## Förlorat arbete

`Redigera: Rädda förlorade commits...` (finns också i Logs commitmeny) hittar
commits som ingen gren, tag eller stash når: de som en reflog fortfarande nämner
(en borttagen grens commits), de som bara en ångra-pin håller kvar, och lösa
(dinglande) sådana. Listan visas direkt och den långsammare sökningen efter
dinglande commits fylls på medan en snurra visas; den blockerar aldrig
gränssnittet.

![Förlorat arbete: de två commits som hörde till en borttagen gren (hittade via reflog) och dinglande commits](../images/lost-work.png)

**Skapa gren här** (`b`) räddar en commit under ett namn du väljer,
**Cherry-pick** (`c`) tillämpar den på den aktuella grenen, **Visa i Log** (`s`)
fungerar för de som fortfarande går att nå.

## Från kommandopaletten

Varje radåtgärd i de tre vyerna är också en rad i kommandopaletten, så rutan
behöver varken mus eller någon bokstav att komma ihåg. Tryck `Ctrl+P` medan rutan
är öppen: paletten öppnas över den och dess rader **Ångrahistorik:**, **Reflog:**
och **Förlorat arbete:** verkar på den valda raden i den vy som syns.

| Vy | Palettrader |
|---|---|
| Ångrahistorik | `Ångrahistorik: Ångra hit...`, `Gör om hit...`, `Återställ bara den här...`, `Återställ grenen till där den var...`, `Uppdatera`, `Stäng` |
| Reflog | `Reflog: Visa i Log`, `Skapa gren här...`, `Återställ till den här positionen...`, `Ladda fler`, `Uppdatera` |
| Förlorat arbete | `Förlorat arbete: Visa i Log`, `Skapa gren här...`, `Cherry-pick`, `Sök igen` |

Raderna är gråade (med skälet) medan deras vy inte är öppen; de tre öppnarna,
`Redigera: Ångrahistorik...`, `Redigera: Reflog...` och `Redigera: Rädda förlorade
commits...`, växlar också en öppen ruta till sin vy. En rad som markeringen inte
kan utföra anger skälet i statusraden i stället för att inte göra något: "den här
åtgärden är redan ångrad: använd Gör om hit", skälet till att en post inte kan
återställas för sig, "bara en gren eller HEAD kan flyttas tillbaka" och så
vidare. Raderna med `...` öppnar samma bekräftelse eller namnfråga som knappen.

## Vad som kan ångras

| Område | Operationer |
|---|---|
| Commits | commit, amend, reword, AI-commit-composern (alla dess commits som ett steg) |
| Grenar och tags | skapa, ta bort, byta namn, ställa in eller ta bort upstream, skapa en spårande gren, skapa / ta bort / annotera en tag, fast-forward, checka ut en pull request som gren |
| Förflyttning | checkout av en gren eller en commit (även en frånkopplad HEAD) |
| Historik | merge (fast-forward eller merge-commit), rebase (vanlig och interaktiv, när den har avslutats), cherry-pick, revert, reset (soft, mixed och hard), den lokala delen av en pull |
| Lokala ändringar | stage och unstage (fil, mapp, hunk, rader, alla), förkasta (fil, mapp, hunk, rader, en fil-checkout), tillämpa en patch, **ta bort en fil från disk**, **ignorera** (fil, filändelse eller mapp: raden i den `.gitignore` du valde), **behandla som binär eller text** (raden i `.gitattributes`) |
| Stashes | push, apply, pop, drop |
| Remotes och konfiguration | lägga till, ta bort, byta namn på, ändra URL eller push-URL för en remote, Git LFS track och untrack |
| På en server | **att ta bort en gren eller en tag på en remote** (från refens meny, paletten, `branch delete-remote`, `tag delete-remote`, `tag delete --both`, eller ett arbetsflödes avslut som tar bort remote-grenen): se nedan |

Medan en merge, cherry-pick eller revert står stilla på konflikter går också
**Spara och markera löst**, stage och unstage att ångra (var och en tas tillbaka
före den under den), och **Avbryt merge** (eller cherry-pick, revert) går att
ångra: att ångra den ger tillbaka hela den stoppade operationen, med dess
konflikter (de tre versionerna av varje olöst fil), de redan lösta och stageade
filerna, konfliktmarkörerna i arbetskatalogen och merge-meddelandet; gör om avbryter
igen. Stage och lösningar går inte längre att ångra när operationen har tagit
slut (committad eller avbruten). *Unstage* och *Unstage alla* rör aldrig en fil
som fortfarande är i konflikt: den har ingen staged ändring att ta tillbaka.

Allt som en förkastning eller en hard reset kastar bort sparas först, så att
ångra ger tillbaka exakt samma byte, inklusive ospårade filer, körbarhetsflaggor
och symboliska länkar. Att ångra en droppad stash ger tillbaka samma stash, med
samma commit-id (den hamnar överst i stash-listan, som `stash@{0}`, inte på sin
gamla plats). Att ångra borttagningen av en remote ger tillbaka dess
remote-spårande grenar, dess alias `origin/HEAD` och uppströmsinställningarna för
de grenar som spårade den.

### Ta bort en gren eller tag på en remote

Före borttagningen frågar GitPoulpi servern vad grenen eller taggen innehåller just
nu och registrerar det: remoten, namnet och den exakta commiten (för en
annoterad tag själva tagobjektet, så att dess annotation bevaras). Borttagningen
görs sedan bara om servern fortfarande har det värdet.

**Ångra pushar tillbaka den** under samma namn, på samma commit, utan att någonsin
tvinga. Eftersom det är en ändring på en server som andra ser frågar Ångra
**alltid först** och säger det ("Att ångra detta skapar grenen topic på origin på
3f2a1c0 igen..."); postens steg säger det också (`gitpoulpi undo --dry-run`). Den
använder samma inloggningsuppgifter som en push. Den remote-spårande grenen kommer
tillbaka med den. **Gör om** tar bort den igen, med samma säkerhet.

Ångra vägrar, utan att ändra något, när:

- grenen eller taggen **skapades igen på remoten** sedan dess, eller flyttades: den
  är på en annan commit än den registrerade, och en push skulle skriva över någon
  annans arbete (vägran nämner båda commitarna);
- commiten (eller tagobjektet) **inte längre finns i ditt repository** (den
  städades bort av skräpsamlingen): ångringen kan inte göras.

"Ta bort lokalt och på remoten" är **en post**: Ångra återställer den lokala och
den remote grenen eller taggen tillsammans, Gör om tar bort båda igen. Att bara ta
bort remote-grenen lämnar den lokala ifred; att ångra det gör detsamma. Ett
arbetsflödes avslut som också tar bort remote-grenen går att ångra som helhet.

## Vad som inte kan ångras, och varför

- **Det mesta på en server**: push, pusha en tag, LFS push, lås och lås upp,
  öppna en pull request. Det tillståndet delas med andra människor; att ångra en
  push vore en force-push eller en borttagning hos dem, och det gör inte
  GitPoulpi åt dig. (Att ta bort en remote-gren eller tag är undantaget: ångringen
  lägger bara tillbaka det servern höll, och vägrar om något ändrats där.)
  Ångra blockeras heller aldrig av en sådan operation.
- **Fetch**: den flyttar bara remote-spårande refs, som speglar servern. Nästa
  fetch skulle göra det igen.
- **Att avbryta en rebase**: ångra erbjuds inte för den (en rebase flyttar grenar
  och har sitt eget tillstånd; avbrottet är självt vägen tillbaka). Att avbryta
  en **merge, cherry-pick eller revert** *registreras*, se ovan.
- **Att skapa en worktree**, **att skapa en patchfil**, clone, init,
  värdtjänståtgärder (pull requests, ärenden) och AI-anrop: de ingår inte i
  repositoryts historik.
- **Att skapa en ny tom fil** från filmenyn, den automatiska borttagningen av
  `*.orig`-filer efter en merge, och att spara en fil i den inbyggda redigeraren
  registreras inte. Inte heller ändring av **SSH-kommandot**
  (`core.sshCommand`) eller av den **delvisa utcheckningen** (inget går förlorat;
  *Stäng av* ger tillbaka alla filer).
- **Att skriva till `.git/info/exclude` eller `.git/info/attributes`** (valet ”endast den här klonen” i listorna för ignorera och behandla som binär/text): de filerna ligger utanför arbetskatalogen och registreras därför inte.
- **Ändringar av filer i arbetskatalogen som görs utanför GitPoulpi** (en
  redigerare, ett skript, en AI-agent som skriver kod) fångas aldrig: ingen
  ögonblicksbild av dem finns, så en `git reset --hard` eller `git checkout --
  fil` i ett skal förlorar oincheckade ändringar för gott. Det som *fångas* av ett
  skals arbete är det som ligger i refs: se nästa avsnitt.

## Ändringar gjorda utanför GitPoulpi

När du arbetar med en AI-agent (eller ett skal bredvid GitPoulpi) sker det mesta
av git-arbetet utanför fönstret: commits, nya grenar och worktrees, borttagna
grenar, resets, rebases, stashar. GitPoulpi ser också dessa. Daemonen håller de
senaste refs den såg (lokala grenar, taggar, HEAD, stash-listan, worktree-listan)
och när repositoryt har ändrats utan att GitPoulpi själv gjort det registrerar
den **en post per sammanhängande ändring** i ångrahistoriken, markerad
**utanför**, med den orsak git loggade (`commit: fix login`, `rebase (finish)`,
`reset: moving to HEAD~1`, `checkout: moving from a to b`) och det namn git
registrerade för den som gjorde det. GitPoulpi kan inte skilja ett skal från en
AI-agent och gissar aldrig; posten säger bara att den gjordes utanför.

- **En skur är en post.** Tjugo commits gjorda mellan två titthål blir en post
  från det första värdet till det sista; en rebase är en post, registrerad när
  den slutar (ingenting registreras medan en rebase eller merge står stilla).
- **Borttagna refs fästs.** En gren, tag eller stash som försvann hålls vid liv
  under `refs/gitpoulpi/undo/` vid sin gamla commit innan något kan rensa bort
  den, så **Återställ bara den här** (eller `Ctrl+Z`) ger tillbaka grenen agenten
  tog bort, även efter reflog och en skräpinsamling.
- **Medan GitPoulpi var stängt.** Ögonblicksbilden sparas bredvid journalen
  (`.git/gitpoulpi/refs-snapshot.json`), så det som hände medan inget GitPoulpi
  kördes hittas vid nästa start och markeras ”medan GitPoulpi var stängt”.
- **`Ctrl+Z` fungerar på dem som på vilken post som helst**: refs flyttas tillbaka,
  en borttagen gren skapas igen, en droppad stash lagras igen, en checkout
  utanför checkar ut den gamla grenen igen. En utcheckad gren flyttas tillbaka
  som `git reset --keep`: filerna följer med, och när en av dina lokala ändringar
  är i vägen **vägrar den och ändrar ingenting**. Gör om tillämpar den igen.
  **Återställ bara den här** fungerar också: en borttagen gren kommer tillbaka och
  de senare åtgärderna blir kvar; en flyttad gren kommer tillbaka som
  `restore/<gren>-<sha>`.
- **Statusraden säger det en gång**: ”ändring utanför inspelad - Ctrl+Z för att
  ångra” när en ny post utanför kommer och fönstret är i förgrunden.
- Eftersom ändringen utanför nu är den nyaste posten försvinner det gamla
  vägrandet ”blockerad av en ändring utanför” för det mesta: du ångrar först
  ändringen utanför.

Det som inte registreras: fetch-flyttar av remote-spårande grenar (de speglar
servern), GitPoulpis egna dolda refs, en worktree som lades till eller togs bort
utan någon grenändring (posten för grenen den bär säger ”worktree tillagd
<sökväg>”; att ångra flyttar bara refs, det tar inte bort worktreen), och allt
inuti filer. `general.journal_outside = false` (Inställningar > Allmänt, ”Spela in
ändringar gjorda utanför GitPoulpi”) stänger av detta: ingenting utanför
registreras och en ändring utanför blockerar den nyaste posten, som förut.

## När ångra vägrar

Innan den agerar kontrollerar GitPoulpi att allt operationen rörde fortfarande är
exakt som operationen lämnade det. Om så inte är fallet **ändrar den ingenting**
och säger vad som har flyttats:

- `main` är vid 3f2a1c0, inte vid a81b9e4 där Commit lämnade den (till exempel en
  commit gjord i ett skal);
- en fil, indexet, en stash eller ett konfigurationsavsnitt har ändrats sedan;
- en merge, rebase eller cherry-pick pågår;
- operationen slutfördes aldrig eftersom GitPoulpi stoppades medan den kördes
  (posten markeras *avbruten*, ångras aldrig automatiskt, och `gitpoulpi
  undo --list` visar vad den var på väg att ändra så att du kan rätta det för
  hand).

Ångra är strikt sist in, först ut: en blockerad nyaste post blockerar också de
äldre, eftersom att ångra en äldre operation under en nyare som fortfarande gäller
vore att gissa. Om du sätter tillbaka saker själv blir posten giltig igen.
Ändringar som inte rör något en post rörde (en shell-commit på en annan gren, en
fetch) blockerar ingenting.

### En blockerad post du inte kan rensa: Glöm

När den nyaste posten har fastnat för gott (repositoryt har gått vidare utanför
GitPoulpi, eller posten blev aldrig klar) kan inte heller något under den ångras.
Verktygstipset för Ångra och avslaget säger då det och anger vägen ut:
**Redigera: Glöm blockerad ångrapost** i paletten (den frågar först), eller
`gitpoulpi undo-forget` på kommandoraden. Att glömma tar bara bort den enda
posten från journalen; repositoryt rörs aldrig, och posten kan inte längre
ångras eller göras om. Posten under blir då den nyaste.

Två fall löser sig av sig självt:

- en merge eller rebase i konflikt som du slutför med vanlig git: posten stängs
  nästa gång GitPoulpi läser repositoryts tillstånd;
- filerna i en stash pop eller apply i konflikt som du löser med **Spara och
  markera löst**: Ångra sätter tillbaka konflikten medan den lösta filen
  fortfarande är staged och inte committad.

## Bekräftelser

Ångra frågar bara när det är riskabelt:

- commiten (eller merge, rebase, cherry-pick, revert) du tar tillbaka finns redan
  på grenens upstream: att ångra den skriver om historik du publicerat, och du
  skulle behöva force-pusha resultatet;
- du har ocommittade ändringar och ångra flyttar arbetskatalogen: dina ändringar
  behålls, men ångra stoppar, utan att ändra något, om en av dem ligger i en fil
  den måste skriva om.

- posten tog bort en gren eller tag på en remote: att ångra den **pushar tillbaka
  den till servern** (se ovan), så den frågar alltid, och det gör även gör om som
  tar bort den där igen.

Gör om, och varje ångring av en ren ref-flytt, frågar aldrig: andra riktningen
är en tangent bort.

## Hur långt tillbaka

`general.undo_depth` (Inställningar > Allmänt) är antalet operationer som
sparas. Standardvärdet är **100**; `0` betyder obegränsat. Att sänka det glömmer
de äldsta posterna direkt. Att starta en ny operation efter en ångra kastar bort
det du kunde ha gjort om, som i vilken redigerare som helst.

Historiken sparas per worktree (en länkad worktree har sin egen), i git-katalogen
(`.git/gitpoulpi/journal.json`), så den överlever att programmet stängs och är
aldrig en del av `git status` eller en commit.

## Stänga av det

`general.undo_enabled = false` (Inställningar > Allmänt, ”Ångra aktiverat”)
stänger av hela funktionen (och med den registreringen av ändringar utanför).
Ingenting registreras eller sparas, så stage,
förkasta och commit går lite snabbare. Knapparna Ångra och Gör om försvinner och
`Ctrl+Z` säger ”Ångra är avstängt (Inställningar > Allmänt)”.

**Att stänga av det raderar historiken**: journalfilen och de dolda refs som
beskrivs nedan tas bort, första gången någon begäran når daemonen medan det är
av. Att slå på det igen börjar från en tom historik.

## Vad som lagras

För att säkerställa att en ångra aldrig kan förlora data hålls varje commit,
tag-objekt, stash eller ögonblicksbild som en post kan behöva vid liv av en dold
ref under `refs/gitpoulpi/undo/` så länge posten finns i historiken. De utesluts
från GitPoulpis egna listor, men ett skal-`git log --all` eller
`git for-each-ref` visar dem, och `git push --mirror` skulle pusha dem (använd
`git push --all` i stället). De tas bort när deras post faller ur historiken
eller när ångra stängs av. Inget annat tas någonsin bort.

## Från kommandoraden och MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` beskriver stegen och ändrar ingenting.
- `--list` skriver ut journalen, nyaste sist (`--all` lägger till poster som
  inte längre går att göra om men ändå sparas), med varje posts id och tillstånd.
- `--expect <id>` får anropet att vägra som *inaktuellt* om inte den posten
  fortfarande är den nyaste: ett skript (eller ett andra fönster) ångrar aldrig
  något annat än det som det tittade på.
- `undo-forget` släpper den blockerade nyaste posten från journalen (`--redo`: den
  nyaste gör-om-posten). Den vägrar när posten inte är blockerad, och avslutas
  med **3** utan `--yes`. För MCP ändrar det något, men det är inte destruktivt:
  repositoryt rörs inte.
- Slutkod **0**: klart, **2**: vägrat (JSON anger skälet, till exempel
  `RemoteRefMoved` när en borttagen remote-gren skapades igen), **3**: riskabelt och
  kräver `--yes` (att ångra borttagningen av en remote-gren eller tag, vilket
  pushar till servern).

Gå tillbaka utan att ångra allt som kom efter:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (även: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (även: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` visar, för varje post, vad **återställ bara den här** kan få
  tillbaka (`restore.items`) eller varför inget (`restore.not_restorable`), och
  vem som gjorde den: `"source": "gitpoulpi"`, eller `"source": "outside"` med
  `outside.message` (gits orsak) och `outside.committer` för en ändring som ett
  skal eller en AI-agent gjort. `undo`, `redo` och `undo-restore` tar poster
  utanför som vilka andra som helst.
- `undo-restore <id>` ger tillbaka det som den posten tog bort (en borttagen
  gren eller tag, en droppad stash, en förkastad eller borttagen fil, en
  borttagen remote med sina spårande grenar och upstreams, en ignoreringsrad) och
  behåller varje senare operation. Den skriver aldrig över: en fil som ändrats
  sedan dess får den sparade versionen bredvid sig som `<name>.restored`; ett
  upptaget gren-namn är ett avslag (slutkod 2) som föreslår ett ledigt, ange det
  med `--name`; efter en reset, amend eller rebase skapar den en ny gren vid den
  gamla spetsen och flyttar aldrig din gren. Återställningen är en journalpost,
  så `gitpoulpi undo` tar tillbaka den.
- `reflog` listar var `HEAD` eller en gren har varit, med gits orsak, även vad ett
  skal gjorde. `reflog-restore` flyttar en gren (eller `HEAD`) till en av de
  positionerna; en utcheckad gren med oincheckade ändringar kräver `--mode`. Den
  journalförs också.
- `lost-commits` listar commits som ingen gren, tag, remote eller stash når.
- När en flytt i ett skal blockerar den nyaste posten bär avslaget den
  reflog-position som håller det posten förväntar sig (`reason.RefMoved.restore`),
  att ges till `reflog-restore`.

Två kommandon finns så att åtgärderna ovan går att ångra från varje yta:
`gitpoulpi rm-file <paths>` (ta bort filer från arbetskatalogen) och
`gitpoulpi ignore <pattern> [--file <path/.gitignore>]` (lägg till en rad i en
`.gitignore`; inget händer när raden redan finns; ångra sätter tillbaka filen
som den var, eller tar bort den om kommandot skapade den, och vägrar om filen
har redigerats sedan).

MCP-servern har ett verktyg per kommando, så `undo`, `redo`, `rm-file` och
`ignore` är också verktyg. `undo`, `redo` och `rm-file` klassas som destruktiva
(spärrade som `reset`, se [MCP och agenter](mcp-and-agents.md)); `ignore` är ett
vanligt verktyg som ändrar något. Journalen är en fil i repositoryt, så `gitpoulpi
undo` i ett skal tar tillbaka vad fönstret gjorde, och fönstret kan ångra vad ett
skript gjorde.

[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · **Svenska** · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# Radar

Radar är en egen flik, direkt efter Hem: en enda lista över det som väntar på dig i **flera repositories**:
de pull (eller merge) requests du skrev, de som ber om din granskning, ärendena
som är tilldelade dig eller skapade av dig, och repositoriesen där du lämnade
pågående arbete. Det är Hem-fönstret med Radar-vyn ovanpå,
så att inget under den kan stjäla ett klick eller en tangent.

Den kräver en kodvärdsprofil (se [Värdtjänster och
ärendehantering](hosting-integration.md)) och repositories länkade till den, eller en profil med
sina kontoövergripande listor påslagna (se ”Bortom omfånget” nedan).
Ett repository som inte är länkat tillfrågas aldrig: det visar en rad **Länka...** i
stället (se ”Repositories med något att säga” nedan).

![Radar över två repositories: en fäst begäran, sedan Väntar på min granskning, Mina pull requests, Tilldelade mig, ärenden och pågående arbete](../images/radar.png)

**Granska...** på en rad för en pull eller merge request (eller `e`) öppnar repositoryt med begärans granskning
i appen ([Granska en pull request](review.md)).

## Öppna den

- **`Alt+R`** (kan bindas om: `global.radar`), varifrån som helst.
- Paletten: `Radar: Öppna`.
- Fliken **Radar**, direkt efter Hem i flikraden.
- Raden **Radar** i menyn **`+`**, medan fliken är stängd.

Från en repositoryflik tar den dig till Hem-fönstret med Radar öppen; `Esc`
eller **Tillbaka** återvänder till listan med repositories, och `⌂` gör detsamma.

## Stänga fliken eller stänga av Radar

Fliken stängs som en repositoryflik: med sitt **x**, ett mittenklick, `Ctrl+W` medan den visas, eller palettkommandot `Radar: Stäng fliken`. Hem är den enda flik som inte kan stängas. En stängd flik förblir stängd i den sessionen, även efter en omstart, tills du öppnar Radar igen (`Alt+R`, `Radar: Öppna` eller menyn `+`); då kommer den tillbaka direkt efter Hem. Att stänga den ändrar ingen inställning.

Om du aldrig använder den, stäng av den under **Inställningar > Allmänt > Aktivera Radar** (`general.radar_enabled`, se [konfigurationsreferensen](config-reference.md)). Avstängd försvinner fliken, dess palettkommandon och `Alt+R`, och GitPoulpi hämtar inget för den. Det gäller direkt, utan omstart.

## Vad den täcker: omfånget

Listan **Omfång** väljer vilka repositories som tillfrågas:

- en **arbetsyta**, med namn (dess repositories, även de utan öppen flik);
- **Öppna flikar**: repositoriesen som är öppna som flikar i den här sessionen.
- **Standardmapp för kloning** och **Standardmapp för bläddring**: varje repository som finns i mappen som anges i
  [Inställningar](config-reference.md) (`repos.clone_root`, `repos.browse_root`), oavsett om det har en flik eller inte.

Den öppnas på det senaste omfånget du valde här (sparas i `radar.toml`), så länge det
fortfarande finns; före ditt första val, eller när den arbetsytan togs bort, på sessionens
aktiva arbetsyta, annars på de öppna flikarna. Valet gäller bara den här vyn: det ändrar inte
fönstrets aktiva arbetsyta. När två repositories har samma mappnamn
läggs den överordnade mappen till i chipet så att du kan skilja dem åt.

### De två mappomfången

Ett mappomfång letar efter repositories **två nivåer ned**: mapparna direkt i den, och mapparna inuti
dem som inte själva är repositories (`~/src/work/app`). Dolda mappar hoppas över, ett repository
öppnas aldrig för att leta vidare, och högst 200 repositories listas. Sökningen körs i bakgrunden, aldrig
på tråden som ritar, och resultatet behålls tills du uppdaterar: knappen Uppdatera, `r` eller
`Radar: Uppdatera` söker igenom mappen på nytt (ett repository som klonats sedan dess dyker upp).

Hosting är fortfarande valfritt per repository: ett repository i mappen som **inte är länkat** till en
värd frågas aldrig. Det bidrar bara med sin rad för pågående arbete, har en rad `inte länkat` med
**Länka...** under Repositories (den öppnar det repositoryts avsnitt för pull requests, där du väljer
leverantör och projekt), och statusraden räknar dem (`3 inte länkade (bara pågående arbete)`).

En mapp som inte är inställd är nedtonad i listan (`Standardmapp för bläddring (ange den i Inställningar)`);
att välja den öppnar dess rad i Inställningar (Inställningar > Allmänt) och lämnar omfånget som det var.
Kloningsmappen räknas som inställd genom sitt standardvärde per operativsystem (mappen som
Klona-dialogen börjar i) så länge den mappen finns; bläddringsmappen har inget standardvärde.
Omfångslistan läses från inställningarna varje gång den öppnas, så en ändring där gäller direkt.

## Listan

| Grupp | Vad den innehåller |
|---|---|
| Fästa | Allt du fäste, oavsett typ, i den ordning du fäste det |
| Väntar på min granskning | Begäranden som ber om din granskning (och som du inte skrev) |
| Mina pull requests | Begäranden du skrev; chipet säger `utkast` för ett utkast |
| Tilldelade mig | Begäranden tilldelade dig |
| Nämner mig | Öppna begäranden och ärenden som nämner dig och som inte redan finns i en grupp ovan (ett ärende som varken är tilldelat eller skapat av dig visas här bara när det nämner dig) |
| Övriga pull requests | De återstående öppna begärandena |
| Ärenden tilldelade mig | Öppna ärenden tilldelade dig |
| Ärenden skapade av mig | Öppna ärenden du skapade |
| Pågående arbete | Repositories i omfånget med oincheckade ändringar eller opushade commits |
| Snoozade | Poster du dolde en stund; hopfälld tills du öppnar den |
| Repositories | Repositories med något att säga (offline, token att förnya, inte länkade) |

Orden ”pull request” och ”merge request” följer den vanliga inställningen för
termer och värdtjänsten.

Varje rad visar postens typ (`⇄` begäran, `●` ärende, `✎` pågående arbete),
repositorychipet (i arbetsytans färg), numret, titeln, författaren, åldern och,
där det finns plats, etiketterna. En rad för pågående arbete visar grenen och
vad som väntar i den: `✎` ändrade filer, `+` nya filer, `↑` commits att pusha,
`↓` commits att pulla.

Antalen står i varje rubrik. Inom en grupp kommer den senast uppdaterade posten
först.

Statusraden under filtret visar en snurra när värdtjänsterna behöver mer än en
stund för att svara. De cachade listorna (samma cache som sidopanelen använder)
visas direkt och ersätts när svaren kommer.

### Kolumner

Repository-chipet, `#` (nummer och kontrollmarkeringar), författaren och åldern är **kolumner som går att storleksändra**. På automatiskt är repositorykolumnen lika bred
som det längsta namnet på skärmen (upp till 40 celler, och högst en tredjedel av fönstret), så `owner/name` och en
kontolistas `group/sub/project` inte kapas i en vanlig terminal. Titeln får det som blir över. Ett namn som
inte ryms slutar med en ellips (`group/sub...`), och:

- **Vila pekaren** på ett kapat repository, en titel, en författare eller en etikettlista så visar en tooltip hela texten. Markera
  raden så namnger statusraden under filtret också hela projektet och titeln, så tangentbordet ser
  samma sak.
- **Dra separatorn** (cellen direkt efter repositorynamnet, efter `#`, eller strax före författaren eller åldern) för att ändra kolumnens
  storlek; **dubbelklicka** på den för att anpassa kolumnen igen. Listan har en **rubrikrad** (repository, `#`,
  titel, författare, ålder) och separatorerna följer `ui.column_resize` som Logs: med `line` (standard) vilar
  ett dämpat `│` i separatorn på rubrikraden, och medan pekaren står på separatorn (på rubriken eller på en rad i listan),
  medan kolumnen är vald med tangenterna, eller medan du drar, ritas en tunn accentlinje nedåt hela listan; en dragning
  från var som helst längs den ändrar storlek. När du släpper försvinner linjen om inte pekaren fortfarande står på separatorn
  (då följer den med pekaren): en dragning med musen lämnar aldrig en kolumn vald. Med `header` finns varken markering eller linje: första
  cellen efter separatorn, på rubrikraden, lyses upp och är det enda stället att dra från. Pekaren blir en
  storleksändringspekare över en separator i terminaler som stöder det. Tillståndschipet har en
  fast bredd, och det har åldern också (den bredaste ålder som visas, tills du ändrar storlek på den; en ålder som kapas av en smal kolumn visar
  sitt fulla datum i ett verktygstips), så att författaren börjar i samma cell på varje rad.
- Eller använd tangenterna: **`C`** väljer en kolumn (igen, eller `Tab`, för nästa), **`]`** gör den två
  celler bredare, **`[`** smalare, **`0`** sätter tillbaka alla kolumner till automatiskt, `Esc` lämnar. Paletten har
  samma: `Radar: Välj kolumn`, `Radar: Bredda kolumnen`, `Radar: Smalna av kolumnen`, `Radar: Återställ kolumnbredder`.

Bredderna sparas i `radar.toml` (`[columns]`, `repo`, `number`, `author` och `age`, i celler) och finns kvar efter en omstart. Det är
samma storleksgester som för Logs kolumner.

### Kontroller och konflikter

En begärans rad bär två små märken efter sitt nummer när värden har sagt något:

| Märke | Betydelse |
|---|---|
| `✓` | head-commitens kontroller godkändes |
| `✗` | en kontroll misslyckades |
| `●` | kontroller köas eller körs |
| `!` (bredvid) | begäran har konflikter med sin målgren |

Vila pekaren på märkena så säger en tooltip det i ord (”Kontroller misslyckades”, ”Konflikter med main”);
markera raden så säger statusraden under filtret detsamma, så att tangentbordet också ser det. En begäran
utan märke är en som värden inte sa något om (ingen CI konfigurerad, eller inte tillfrågad).

- **GitLab** skickar pipelinestatus och konfliktflaggan med listan över merge requests: varje rad har
  sina märken.
- **GitHub** gör inte det: listan har ingetdera. GitPoulpi frågar efter dem per begäran (själva begäran för
  dess merge-tillstånd, head-commitens samlade status och dess kontrollkörningar: tre anrop), för högst
  **15 öppna begäranden per repository**, dina först. Om GitHub vägrar ett av de anropen (en gräns för
  antal anrop) förblir resten omärkta till nästa uppdatering. Svaren sparas med hosting-cachen.

- **Gitea och Forgejo** läses som GitHub: listan bär konfliktflaggan (`mergeable`), commitstatusen
  efterfrågas per begäran (högst 15 per repository, dina först), och ärendena är samma grupper.
  Omnämnanden använder serverns sökning (`mentioned_by`).

### Omnämnanden

**Nämner mig** och filtret **Visa: Nämner mig** använder det värden vet om omnämnanden. GitHub
tillfrågas genom sin sökning (`mentions:`), GitLab genom din att göra-lista: bara omnämnanden som fortfarande är **väntande**
där räknas, så en uppgift du markerade som klar lämnar gruppen.

Stängda och sammanslagna begäranden och stängda ärenden listas inte.

## Vyer och filter

Tre listor och ett textfält avgränsar listan. Varje val är en rullgardinslista:
klicka på den, eller använd dess tangent.

- **Vy** (`v`): Allt, begäranden, Ärenden, WIP (pågående arbete), Snoozade.
- **Visa** (`f`): Alla, Mina (begäranden du skrev, ärenden du skapade),
  Tilldelade mig, Granskning begärd, Nämner mig.
- **Omfång** (`w`): se ovan.
- **Filtertext** (`/` eller `Ctrl+F`): fuzzy-matchning på repository, nummer,
  titel, författare, gren och etiketter. `Enter` ger tillbaka tangenterna till
  listan och behåller texten, `Esc` rensar den.

`Ctrl+N` visar ett märke på filtret, varje rullgardinslista, knapparna Uppdatera och Tillbaka och varje sektionsrubrik på skärmen. Tryck på märkets tecken: filtret får tangentbordet, en rullgardinslista öppnas, en knapp körs och en rubrik får markeringen (`Enter` eller `Space` viker ihop den sedan). `Esc` avbryter. Se [Märken](keybindings.md#märken).

## Åtgärder

Markera en rad och använd dess meny (högerklick, `⋮` i radens slut, `.` eller
`Menu`), en tangent eller paletten (`Radar: ...`). En musåtgärd har alltid en
tangent och en palettmotsvarighet.

| Åtgärd | Tangent | Palett |
|---|---|---|
| Öppna repository (dess flik öppnas, eller växlas till) | `Enter` / dubbelklick | `Radar: Öppna repository` |
| Granska... (en begäran; en begäran som inte är klonad frågar hur) | `e` | `Radar: Granska begäran...` |
| Granska en begäran via värdens API (ingen klon) | `a` i frågan | `Radar: Granska via API:et` |
| Klona repositoryt, granska sedan begäran | `c` i frågan | `Radar: Klona och granska...` |
| Öppna i webbläsaren | `o` | `Radar: Öppna i webbläsaren` |
| Checka ut gren (en begäran) | `c` | `Radar: Checka ut gren` |
| Skapa worktree... (en begäran) | `t` | `Radar: Skapa worktree...` |
| Visa i repository (dess lista) | `s` | `Radar: Visa i repository` |
| Börja arbeta med ärende... (ett ärende) | `n` | `Radar: Börja arbeta med ett ärende...` |
| Fäst överst, eller Lossa | `p` / klicka på `★` | `Radar: Fäst eller lossa` |
| Redigera tilldelade... / granskare... / etiketter..., Ange milstolpe... (GitHub, GitLab) | radmenyn | `Radar: Redigera tilldelade...`, `Radar: Redigera granskare...`, `Radar: Redigera etiketter...`, `Radar: Ange milstolpe...` |
| Snooza... | `z` | `Radar: Snooza...` |
| Sluta snooza | `u` | |
| Visa eller dölj begäranden från repositories som inte är klonade | `a` | `Radar: Visa ej klonade begäranden` |
| Visa eller dölj utkast bland de kontoövergripande raderna | `d` | `Radar: Visa utkast` |
| Slå på en värds kontoövergripande granskningsbegäranden | `T` / `Enter` på dess tipsrad / **Slå på** | `Radar: Slå på kontoövergripande granskningsbegäranden...` |
| Uppdatera | `r` | `Radar: Uppdatera` |
| Radmenyn | `.` | `Radar: Radmeny...` |
| Tillbaka till Hem | `Esc` | `Radar: Tillbaka till Hem` |

Åtgärder som behöver repositoryts eget fönster (checka ut, worktree, visa, börja
arbeta, länka, metadataväljarna) växlar först till dess flik och körs där när den har laddats:

- **Checka ut** använder sidopanelens egen utcheckning: grenen när den är känd
  lokalt, annars begärans head hämtad till en lokal gren (en forks, till
  exempel).
- **Skapa worktree...** öppnar dialogen Skapa worktree på begärans gren.
- **Visa i repository** markerar posten i listan PULL REQUESTS eller ÄRENDEN.
- **Börja arbeta med ärende** öppnar prompten Ny gren med ett namn gjort av
  ärendet (`12-fix-the-thing`).

### Fästa och snooza

En **fästning** håller en post överst. En **snoozning** döljer den i **1 timme**,
**4 timmar**, **Till i morgon** (08:00 nästa morgon, din lokala tid, även när klockan ännu inte är 08:00 nu) eller **Tills det ändras**
(värdtjänsten rapporterar en uppdatering av posten, eller, för en rad med
pågående arbete, att dess räknare flyttas). Båda sparas i `radar.toml` i din
konfigurationsmapp, nycklade på värd, repository, typ och nummer, och delas av
varje session på maskinen.

### Repositories med något att säga

Ett repository med ett problem får en egen rad sist i listan, och dess cachade
poster stannar på skärmen:

- **offline, 5m gammal**: värdtjänsten gick inte att nå;
- **Anslut igen**: token saknas eller nekas; knappen öppnar Inställningar >
  Integrationer;
- **pausad denna runda**: värdtjänsten sa åt oss att sakta ner (eller nekade
  token), så GitPoulpi frågade den inte igen för de andra repositoriesen i den
  här rundan;
- **inte länkad, GitHub föreslås**: repositoryts remote matchar en profil, men
  värdtjänst är opt-in per repository, så inget frågas förrän du länkar det.
  **Länka...** tar dig till repositoryts avsnitt PULL REQUESTS, där du väljer
  leverantör och projekt.

Högst fyra repositories tillfrågas samtidigt, och aldrig på tråden som ritar
skärmen.

## Bortom omfånget: kontoövergripande rader

Omfånget täcker de repositories du har klonat. En begäran som väntar på din granskning i ett repository du
aldrig klonat är osynlig där. **Kontoövergripande rader** frågar själva värden, en gång per profil, och
läggs till i listan oavsett omfång: de handlar om dig, inte om en mapp.

De är **av som standard** (en värds sökning kostar API-kvot). Medan en ansluten kodvärd inte har någon av
sina listor påslagen visar listans topp en rad för den:

`GitHub: lista begäranden som väntar på din granskning, även i repositories du inte har klonat  [Slå på]`

Klicka på **Slå på**, tryck `Enter` på raden, tryck `T`, eller kör `Radar: Slå på kontoövergripande granskningsbegäranden...`:
det skriver `integrations.radar.review_requests = true` för den profilen i `config.toml` och
frågar värden direkt. De andra listorna (alla öppna begäranden i dina namnrymder, ärenden tilldelade dig,
utkast, namnrymder) ställs in med `gitpoulpi hosting integrations radar` eller i profilens inställningar; se
[konfigurationsreferensen](config-reference.md) och [CLI-referensen](cli-reference.md). GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center och Gerrit svarar; Bitbucket Cloud har ingen kontoövergripande
slutpunkt, så dess begäranden visas bara för de länkade repositories i omfånget.

Så placeras raderna:

- En rad som omfånget redan listar (samma värd, projekt, slag och nummer) visas en gång: **omfångets
  rad vinner**.
- En rad vars projekt du har klonat någonstans (senaste repositories, arbetsytor, mapparna som
  mappomfången hittade; matchat på remote-URL:en) bär den klonen: **Enter**, checka ut, worktree och de
  andra öppnar den som vilken rad som helst.
- Varje annan rad är **inte klonad**: repositorychippet är dämpat och visar projektet (`acme/app`), och
  statusraden säger `Inte klonade` när den är markerad. **Öppna i webbläsaren** fungerar, **Granska** frågar hur
  den ska granskas (nedan), och checka ut, worktree, visa i repository och börja arbeta säger
  `klona repositoryt först`.

### Granska en begäran

**Granska** (`e`) på en begäran i omfångets egna repositories öppnar det repositoryts flik med granskningen
i. På en begäran från en kontoövergripande rad beror sättet på om en klon är känd och på
profilens inställning **Begäran från ett repository som inte är klonat** (Inställningar > Integrationer, `review_mode`
i [konfigurationsreferensen](config-reference.md)):

| Inställning | En klon är känd | Ingen klon är känd |
|---|---|---|
| Fråga (standard) | frågar: **Öppna den lokala klonen** (`l`) eller **Granska via API:et** (`a`) | frågar: **Klona, granska sedan** (`c`) eller **Granska via API:et** (`a`) |
| Klona det | öppnar klonen | öppnar dialogen Klona |
| Granska via API:et | granskar via API:et | granskar via API:et |

Frågan är en liten dialog: bokstäverna väljer, **Enter** väljer den markerade raden, `Esc`, **Avbryt**,
`✕` och ett klick utanför stänger den, och **Kom ihåg för GitHub** (`r`) skriver profilens inställning så att
frågan inte ställs igen (raden i Inställningar ändrar tillbaka den). `Ctrl+N` märker dess kontroller.

- **Granska via API:et** öppnar granskningen i Hem-fönstret, över Radar, med koden hämtad från
  värden (se [Granska utan en klon](review.md#granska-utan-en-klon)). `Esc` eller `✕` ger
  tillbaka Radar; palettens *Granskning: ...*-rader fungerar på den.
- **Klona, granska sedan** öppnar dialogen Klona på det anslutna konto som raden kom från (samma lista med repositories,
  grupperad efter ägare), med begärans repository markerat och standardmappen för kloner ifylld; ändra
  någon av dem och klona sedan, med samma inloggning som vid vilken klon som helst från det kontot. Om repositoryt inte finns i
  kontots lista (du är inte medlem i det) förblir listan öppen med sökningen på dess namn och en notis säger
  det; för en värd som inte kan lista repositories öppnas dialogen på **Klona med URL** med adressen ifylld. När klonen har landat länkas det nya repositoryt till värden
  (dess källa för PULL REQUESTS, och dess källa för ÄRENDEN när värden har ärenden och ingen är satt), dess flik öppnas
  och granskningen av den begäran är öppen i den. Att avbryta dialogen, eller en klon som misslyckas, länkar ingenting.
- Begäranden från Azure DevOps och Bitbucket har ingen granskning i appen: **Granska** öppnar begärans sida i
  webbläsaren och säger det på statusraden.

Palettens rader *Radar: Granska via API:et* och *Radar: Klona och granska...* hoppar över frågan.

Två kryssrutor bredvid filtret (de visas när en kontoövergripande lista efterfrågas), sparade i
`radar.toml`:

| Kryssruta | Tangent | Standard | Effekt |
|---|---|---|---|
| `[x] Inte klonade` | `a` | på | Listar raderna för repositories utan känd klon. Av stannar bara raderna med kända kloner kvar. |
| `[ ] Utkast` | `d` | av | Listar utkast till begäranden bland de kontoövergripande raderna. Utkast i omfångets egna repositories listas alltid. |

Båda finns bland `Ctrl+N`-märkena och i paletten (`Radar: Visa ej klonade begäranden`, `Radar: Visa utkast`).

En profils svar sparas i `radar-account.json` i **5 minuter**: att öppna Radar igen inom den
tiden visar raderna utan att fråga värden. **Uppdatera** (`r`) frågar alltid. En värd som sa åt oss att sakta ner
eller nekade token tillfrågas inte igen i samma runda. Högst 300 rader behålls per profil och slag,
och en rad under **Repositories** säger hur många fler som fanns, eller varför en profil inte kunde läsas
(**Anslut igen** öppnar Inställningar > Integrationer).

## Begränsningar

- Kontoövergripande rader kommer från GitHub-, GitLab- (även självhanterade), Gitea-, Forgejo-, Azure DevOps-,
  Bitbucket Data Center- och Gerrit-profiler (inte Bitbucket Cloud). Ärenden tilldelade dig kommer från de
  fem första; Bitbucket och Gerrit har inga ärenden.
- Listorna per repository använder GitHub-, GitLab- (även självhanterade), Gitea-, Forgejo-, Azure DevOps-,
  Bitbucket- och Gerrit-profiler; kontroller, konflikter och omnämnanden läses bara för de fyra första.
- GitLab-omnämnanden kommer bara från väntande uppgifter; GitHub-, Gitea- och Forgejo-kontroller efterfrågas för 15 begäranden per repository.

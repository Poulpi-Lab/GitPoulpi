[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · **Svenska** · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull requests och ärenden

Att läsa, skapa och hantera pull / merge requests och ärenden från sidopanelen, tavlan och verktygsfältet. Anslut först ett konto och länka ett repository: se [Värdtjänster och ärendehantering](hosting-integration.md). Allt som sägs om ett enskilt ärende eller en enskild begäran (kommentarer, granskningar, trådar) finns i [Konversationer](conversations.md); att läsa och granska en begäran finns i [Granska en pull request](review.md).

## Pull requests och ärenden i sidopanelen

### PULL REQUESTS

Sektionen har en rubrik med antalet och ett `+` (skapa en pull request),
leverantörs- och projektmenyerna, en sökruta och en tillståndsmeny (*Öppna*,
*Mergade / stängda*, *Alla*). Sedan kommer fyra grupper, var och en med sitt antal:

- **Mina pull requests**: du är författaren.
- **Tilldelade mig**.
- **Väntar på min granskning**: du är en begärd granskare.
- **Alla pull requests**: hopfälld från början.

Sökrutan filtrerar varje grupp (titel, nummer, författare, gren). Varje
ändring av tillståndsmenyn frågar värden igen.

En rad börjar med en tillståndsglyf: `●` öppen (grön), `○` utkast, `◆` mergad
(lila), `✕` stängd (röd). Begäran för den utcheckade grenen är fetstilt.
Raderna lyder `#12  titel` (GitHub, Gitea, Forgejo, Bitbucket) eller `!53  titel` (GitLab,
Azure DevOps).

### ÄRENDEN

Sektionens titel nämner leverantören: `GITHUB-ÄRENDEN`, `JIRA-ÄRENDEN` och så
vidare. Den lyder `...-ARBETSOBJEKT` för Azure DevOps och `...-KORT` för Trello.
Under menyerna: en sökruta, tillståndsmenyn (*Öppna*,
*Tilldelade mig*, *Stängda*, *Alla*) och ärendena. En Jira-rad lyder
`ABC-12  titel`, med typ och status nedtonade när sidopanelen är tillräckligt
bred.

![Sektionen ÄRENDEN i sidopanelen med högerklicksmenyn för ärende #7 öppen, Öppna konversation först](../images/issues-sidebar.png)

### Åtgärder på en rad

- `Enter` eller ett dubbelklick öppnar raden: en pull request i [granskningen](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), ett ärende, och en begäran på Azure DevOps eller Bitbucket, som en [konversation](conversations.md)
  i fönstret, och allt annat i din webbläsare. Radmenyns **Öppna i webbläsare** öppnar alltid webbläsaren.
- Högerklick, eller `.`, öppnar radmenyn:
  - **Öppna konversation** (ärenden, och begäranden som saknar granskningsvy), **Öppna i webbläsare**, **Kopiera URL**,
    **Kopiera nummer**.
  - För en pull request: **Redigera på <provider>**, **Checka ut källgrenen**,
    **Visa källgrenens commits**. För en begäran från en fork lyder checkout
    **Checka ut pull request (fork)**, och **Visa** är nedtonat.
  - För ett ärende: **Skapa gren från ärende**. Den öppnar prompten för ny gren
    med ett namn som `12-fix-the-thing` (eller `ABC-123-fix-the-thing` för
    Jira).
  - På GitHub, GitLab, Gitea och Forgejo har en pull request-rad också **Granska...**
    (även **Enter** på raden): granskningen i appen, se
    [Granska en pull request](review.md).
  - På GitHub, GitLab, Gitea och Forgejo har båda raderna också **Redigera tilldelade...**,
    **Redigera etiketter...** och **Ange milstolpe...** (och **Redigera granskare...** på
    en pull request): se
    [Tilldelade, granskare, etiketter och milstolpe](#tilldelade-granskare-etiketter-och-milstolpe).
- **Redigera på <provider>** har tre andra ingångar: pennan `✎` längst till höger
  på en pull request-rad som pekas på eller är vald, `e` på den valda raden, och
  palettkommandot *Integrationer: Redigera länkad pull request på värden*
  (för den utcheckade grenens begäran).
- Om källgrenen ännu inte finns i repositoryt säger GitPoulpi det. Gör en fetch
  först (`Ctrl+G f`).

**Checkout av en pull request.** GitPoulpi hämtar begäran till en lokal
gren `pr/<n>` (`mr/<n>` på GitLab) som spårar den, och byter till den. På
GitHub, GitLab, Gitea och Forgejo fungerar detta även för forkar (GitPoulpi hämtar värdens
särskilda head-ref). Azure DevOps och Bitbucket hämtar begärans källgren.
En begäran från en fork nekas där: lägg till forken som remote och
checka ut dess gren själv. Ångra återgår till föregående gren. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Granska en pull request från kommandoraden (och en agent).** På GitHub
(inklusive Enterprise), GitLab (inklusive self-managed), Gitea och Forgejo kan en begäran
läsas och granskas utan att lämna GitPoulpi: `hosting review show <n>` (filer,
merge-tillstånd, kontroller), `hosting review threads <n>` (granskningstrådar och
konversation), `hosting review comment`, `hosting review reply`, `hosting review resolve`
och `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. Diffen kommer från ditt repository, inte från
värden: `hosting review fetch <n>` hämtar begärans head till
`refs/gitpoulpi/pr/<n>` och dess målgren till `refs/gitpoulpi/pr/<n>-base`
(ingen av dem är en gren, inget checkas ut). Radnumren är de för den
diffen: `new` är raden i head, `old` raden i merge-basen. GitLab
har inget granskningstillstånd ”begär ändringar”: det vägras där, utan att något skickas.
Gitea och Forgejo tar en kommentar på bara en rad (inga intervall); en tråd är
kommentarerna på en rad i en fil. Gitea kan lösa en tråd, Forgejo 11 kan det inte
(värden har ingen slutpunkt för det: anropet säger det). Ett svar på en server
utan svarsslutpunkten publiceras som en granskning med en kommentar på samma rad.
Gitea vägrar låta dig godkänna eller avvisa din egen begäran.
Utan klon: lägg till `--project owner/name` (GitLab: `group/project`) i `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` och `suggest` för att granska ett projekt som inte är det aktuella repositoryt;
`hosting review diff <n> --project P` läser diffen (hunkar per fil) från värdens API, och
`hosting file <path> --project P [--ref R]` läser en fil vid en revision (GitHub, GitLab, Gitea, Forgejo och Gerrit; på Gerrit läses diffen för ett
patch set fil för fil och filen kommer från grenen eller, för ett commit-id, från committen). En fil som värden
inte visar i diffen (för stor) markeras `omitted`.
MCP-verktygen är samma kommandon (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Tilldelade, granskare, etiketter och milstolpe

På GitHub (inklusive Enterprise), GitLab (inklusive self-managed), Gitea och
Forgejo kan du ändra vem en pull request eller ett ärende är tilldelad, vem som granskar den, dess etiketter
och dess milstolpe utan att lämna GitPoulpi. Andra leverantörer visar inget av det.

- **Radmeny**: **Redigera tilldelade...**, **Redigera granskare...** (bara pull
  requests), **Redigera etiketter...**, **Ange milstolpe...**.
- **Palett**: *Hosting: Redigera tilldelade...*, *Hosting: Redigera granskare...*,
  *Hosting: Redigera etiketter...*, *Hosting: Ange milstolpe...* listar först var och en
  begäranden och ärendena i sidofältet att välja bland. I
  [Radar](radar.md) verkar samma fyra på den valda raden.
- Väljaren öppnas direkt och läser in projektets personer eller etiketter (och vad
  objektet har nu) från värden. En snurrrad visas efter ungefär 150 ms.
  Skriv för att filtrera (fuzzy-matchning), `Up` / `Down` flyttar, `Space` kryssar i eller ur,
  `Enter` verkställer, `Esc` avbryter, `Tab` når knapparna. Ett klick
  på en rad kryssar den, och hjulet rullar. Personer visar ett initialmärke,
  etiketter sin färg.
- Milstolpen är en rullgardinsmeny (med **Ingen milstolpe** för att rensa den).
- Efter **Verkställ** uppdateras värden, och listorna och Radar visar
  ändringen efter sin nästa uppdatering, som startar direkt.

Skapa-formulären (en pull request, ett ärende) har samma val som en rad med
knappar: `Tilldelade: ingen`, `Granskare: ingen`, `Etiketter: ingen`,
`Milstolpe: ingen`. På GitHub, GitLab, Gitea och Forgejo ersätter knappen Etiketter det skrivna
etikettfältet i ärendeformuläret. Om en värd tar emot den nya pull requesten men
vägrar några av dess etiketter eller personer säger statusraden det.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, och `--assignee`, `--reviewer`, `--label`,
`--milestone` på `hosting pr` och `hosting issue create`; se
[CLI-referensen](cli-reference.md).

### Tangenter i sektionerna

| Tangent | Åtgärd |
|---|---|
| `Enter` / dubbelklick | öppna konversationen i GitPoulpi där värden har en (en pull request på GitHub, GitLab, Gitea eller Gerrit öppnar granskningen), annars webbläsaren |
| `o` | öppna raden i webbläsaren |
| `.` / högerklick | radmeny |
| `e` | redigera den valda pull requesten hos värden |
| `n` | ny pull request (PULL REQUESTS) eller nytt ärende (ÄRENDEN) |
| `v` | öppna tillståndsmenyn |
| `r` | uppdatera |
| `Enter` / `Space` på en rubrik | fäll ihop eller ut sektionen |

### Anslut igen och offline

När token saknas, avvisas eller saknar en behörighet lyder sektionens första rad
`Anslut igen (token avvisad)`. Klicka på den, eller tryck `Enter`, för att öppna
profilen i Inställningar > Integrationer.

När värden inte kan nås ligger de senaste listorna kvar på skärmen och rubriken
visar deras ålder (`· 5m gammal`). Det senaste svaret cachas också på disk
(`.gitpoulpi/hosting-cache.json`), så listorna visas innan nätverket
svarar.

## Ärendetavlan

Knappen **Tavla** i ÄRENDEN-huvudet (palett: **Ärenden: Öppna tavlan**) visar ärendena i den länkade källan som en Kanban-
tavla över fönstret. Kolumnerna beror på källan:

| Källa | Kolumner | Att flytta ett kort |
|---|---|---|
| Lokala Markdown-anteckningar | statusarna (`statuses` i profilen, sedan varje annan som används) | skriver om anteckningens `status` |
| GitHub, GitLab, Gitea, Forgejo | **Öppna**, sedan etiketterna du valde, sedan **Stängda** | byter statusetikett; till **Stängda** stänger det ärendet, till en kolumn igen öppnar det om |
| Trello | tavlans öppna listor, från vänster till höger | flyttar kortet till listan |
| Redmine | ärendestatusarna | ändrar ärendets status (Redmines arbetsflödesregler gäller) |
| Jira (Cloud och Data Center) | projektets statusar, att göra först, klart sist | kör arbetsflödesövergången som leder till kolumnen (`GET` sedan `POST .../transitions`); utan en sådan övergång går kortet tillbaka och statusraden säger `no transition from X to Y` |
| Azure DevOps (Services och Server) | tillstånden för projektets work item-typer, efter tillståndskategori | sätter `System.State`; ett work item vars typ saknar kolumnens tillstånd går tillbaka, med orsaken (`a Bug has no state Done`) |

På ett GitHub- eller GitLab-projekt frågar **Kolumner** (eller `c`, eller **Tavla: Ange kolumnetiketter...**) efter etiketterna som är
kolumnerna, i ordning, till exempel `status::todo, status::doing, status::review`. Den föreslår projektets etiketter som
ser ut som statusar. Valet sparas för repositoryt. Utan etiketter är tavlan Öppna / Stängda. Att flytta ett kort
rör bara statusetiketterna du listade: dess andra etiketter stannar. En värdtavla listar de öppna ärendena och de 60 senast
stängda.

En Jira-övergång som kräver obligatoriska fält (en lösning, till exempel) vägras av Jira och kortet går
tillbaka med Jiras eget meddelande: gör den på webbplatsen.

Samma tavla finns för skript och agenter: `gitpoulpi hosting board` skriver ut kolumnerna med sina kort och
`gitpoulpi hosting board move <card> <column>` flyttar ett (se [CLI-referensen](cli-reference.md)).

Ett kort visar numret, titeln, de tilldelade som initialer, etiketterna och milstolpen. Dra det till en annan kolumn,
eller `Shift+Left` / `Shift+Right`, eller **Tavla: Flytta kort till...** (`m`). Det flyttas direkt; efter ungefär 150 ms säger en snurra
att värden fortfarande arbetar, och om värden vägrar går kortet tillbaka och statusraden säger varför. `Enter` öppnar
ärendets [konversation](conversations.md) i GitPoulpi, och tavlan kommer tillbaka när du stänger den; `e` redigerar tilldelade, etiketter och milstolpe; `n` startar ett nytt ärende i
den valda kolumnen; `a`, `l`, `M` och `/` filtrerar på tilldelad, etikett, milstolpe och text, `x` rensar filtren; `r`
läser värden igen. Tavlan läses om när sidofältets listor uppdateras. Radar har ingen tavla.

## Skapa en pull request

Öppna formuläret med `+` i rubriken PULL REQUESTS, `n` i sektionen, eller
palettkommandot *Pull requests: Skapa pull request...*. En grenmeny
har också **Starta en pull request från <gren>...**.

Formuläret har:

- En flik per ansluten profil som har pull requests (`Left` / `Right` på
  flikraden).
- Kombinationsrutor för **Från repo / Gren** och **Till repo / Gren**. Långa listor är
  sökbara. Du kan skriva en gren eller ett repository som listorna inte visar.
- **Titel**, ifylld med ämnet för den översta committen.
- **Beskrivning**, på flera rader (`Enter` lägger till en ny rad, `Tab` lämnar
  fältet).
- Knapparna **Tilldelade**, **Granskare**, **Etiketter** och **Milstolpe** (GitHub,
  GitLab, Gitea och Forgejo), som öppnar väljaren.
- **Skicka som utkast**.
- Med [AI](ai.md) på: **Generera titel och beskrivning**.

Formuläret öppnas med profil, repository och grenar för den utcheckade
grenen (`feature -> main`). Ett annat källrepository ger en begäran
från en fork.

**Skapa pull request** (eller `Ctrl+S`) skapar begäran hos värden. Resultatet
öppnas i din webbläsare och statusraden nämner den.

## Skapa ett ärende

När sektionen ÄRENDEN har en leverantör **och** ett projekt har dess rubrik ett
`+` (*Nytt ärende*). Också: `n` i sektionen, eller palettkommandot
*Hosting: Nytt ärende...*.

Formuläret har **Titel** (obligatorisk), **Beskrivning** (flera rader) och
**Etiketter** (kommaseparerade, till exempel `bug, ui`; på GitHub, GitLab, Gitea och Forgejo öppnar
knapparna Etiketter, Tilldelade och Milstolpe väljaren i stället). Jira och Azure DevOps
lägger till en rullgardinsmeny **Typ**. Trello lägger till en rullgardinsmeny **Lista**. Linear och YouTrack
lägger till en rullgardinsmeny **Status** (arbetsflödestillståndet, eller värdet `State`, som ärendet
startar i). Redmine lägger till en rullgardinsmeny **Typ** med projektets trackers (Bug, Feature,
Support...); det har inga etiketter, så det finns ingen knapp Etiketter för det och en etikett
som anges på kommandoraden vägras. Formuläret fyller
dessa menyer från värden när det öppnas.

**Skapa ärende** (eller `Ctrl+S`) skapar det hos värden. `Esc` eller **Avbryt**
stänger formuläret. Om värden nekar (avvisad token, saknad behörighet, okänt
projekt, okänd etikett) visar formuläret skälet och behåller det du skrivit.
Vid lyckat resultat nämner statusraden det nya ärendet och sektionen ÄRENDEN
uppdateras.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Skapa på värdens egen webbsida

Formuläret Nytt ärende och formuläret Skapa pull request har en
knapp **Skapa på <provider>**, uppe till höger. Tangenten är `Alt+W`. Den öppnar
värdens webbformulär i din webbläsare, ifyllt med det du skrivit hittills. Dialogen
förblir öppen. Den behöver ingen token och gör inga API-anrop, så den fungerar även
efter ett API-fel.

- GitHub har ingen URL-parameter för *utkast*, så utkastrutan förs inte
  över. GitLab får ett titelprefix `Draft: `.
- GitLab tar etiketter bara via id, så de läggs till som en snabbåtgärd
  `/label ~"x"` i slutet av beskrivningen.
- En mycket lång beskrivning kapas så att URL:en håller sig under 6000 tecken,
  med en notering `[...truncated: paste the rest]`.
- Bitbucket Data Center har ingen adress till något webbformulär, så det har ingen
  knapp **Skapa på <provider>**.

CLI: `gitpoulpi hosting issue create ... --web` och `gitpoulpi hosting pr --web`
skriver ut samma URL.

## Chippet i verktygsfältet

Längst bort i verktygsfältet (som standard till höger) visar ett chip begäran för den utcheckade
grenen: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) eller
`BB #12` (Bitbucket). Dess färg visar tillståndet: grön öppen, grå utkast,
lila mergad, röd stängd.

- Peka på det för titel, tillstånd och `källa → mål`.
- Klicka på det, eller tryck `Ctrl+G g`, för att öppna begäran i webbläsaren.
- Högerklicka på det för **Öppna i webbläsare**, **Kopiera URL**, **Uppdatera** och
  **Byt profil**.

En pushad gren utan begäran visar ett nedtonat `+ PR`-chip. Klicka på det för att starta en
pull request. Det finns inget chip när ingen profil matchar, när grenen inte
är pushad eller när värden inte kunde nås. I ett smalt fönster döljs chippet
för att ge plats åt verktygsfältets knappar.

## När listorna uppdateras

Listorna uppdateras:

- när du öppnar ett repository;
- när den utcheckade grenen ändras;
- efter en push eller en fetch;
- var *Uppdatering av pull requests och ärenden (minuter)*
  (`general.hosting_refresh_minutes`, standard 5, `0` stänger av timern;
  Inställningar > Integrationer);
- när du stänger Inställningar;
- när du trycker `r`, kör *Integrationer: Uppdatera pull requests och ärenden*, eller
  ändrar ett filter.

Förfrågningar blockerar aldrig fönstret.

Värdar begränsar hur många anrop du får göra. GitPoulpi skickar tillbaka `ETag` från
det senaste svaret, så en oförändrad lista kostar lite (ingenting på GitHub). När en
värd rapporterar att få anrop återstår säger statusraden det. När inga
återstår slutar GitPoulpi fråga den värden tills gränsen nollställs, och listorna
förblir som de är.

## Terminologi

GitHub, Azure DevOps och Bitbucket säger ”pull request”. GitLab säger ”merge
request”. *Terminologi för granskningsbegäran* (`ui.pr_terminology`, under
Inställningar > Integrationer) väljer ordet för varje etikett, meny, palettkommando
och meddelande:

- **Auto (per värd)**: följer repositoryts länkade leverantör (”Pull
  Request” så länge inget är länkat).
- **Pull request** eller **Merge request**: alltid det ordet.

Ändringen gäller direkt. CLI:t och MCP säger alltid ”pull/merge request”.
Den här sidan säger ”pull request”.

## Referenser i commit-meddelanden och grennamn

När repositoryts ärenden eller pull requests är länkade är referenser i
commit-meddelandet i Commit-detaljer understrukna. Ett klick öppnar den länkade
sidan. Tangentbord: palettkommandot
*Integrationer: Öppna commit-referens (#ärende / !request)...* (en meny när
meddelandet har flera referenser). Ett repository utan något länkat har inga
länkar.

Vad som räknas som en referens beror på leverantören för sektionen ÄRENDEN:

| ÄRENDEN länkat till | I ett meddelande | I ett grennamn |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` och `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, bara när `ABC` är nyckeln för det länkade projektet. Text som `UTF-8`, `SHA-256` eller `ISO-8601` är aldrig en länk. | `ABC-123` var som helst i namnet (`feature/ABC-123-x`), med versaler |
| Linear | `ENG-123`, bara när `ENG` är nyckeln för det länkade teamet (eller för ett team i profilens teamlista). `UTF-8` är aldrig en länk. | Linears eget `branchName` när du skapar grenen från ett ärende; `ENG-123` var som helst i namnet läses tillbaka |
| YouTrack | `ABC-123`, bara när `ABC` är det korta namnet för det länkade projektet (eller för ett i profilens projektlista) | `ABC-123` var som helst i namnet |
| Redmine | `#123`, bara när Redmine är den länkade ÄRENDEN-källan (en pull request-källa behåller sitt eget tecken) | `123-fix-login` |
| Trello | en adress `https://trello.com/c/<shortLink>`; `#12` bara när ett cachat kort har numret 12 | `12-fix-login` |

| PULL REQUESTS länkat till | I ett meddelande |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

När båda listorna matchar `#N` vinner ärendet. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**I konversationer.** Texten i ett ärende eller en begäran läses med värdens grammatik, som är bredare än
tabellen ovan: `owner/repo#7`, `!7`, `GH-7`, `@login`, commit-id:n och de andra formerna som listas i
[Korsreferenser](conversations.md#korsreferenser). Repositoryts länkade källor matchas först, så en Jira-nyckel i en GitHub-
kommentar öppnar fortfarande sitt ärende i appen. `gitpoulpi hosting markdown --body TEXT --id I` visar vad appen hittar i
en text.

### Infoga en ärendereferens

I commit-composern öppnar knappen `#▾` längst till höger i Sammanfattning
en filtrerbar lista över de länkade ärendena. Ärendet som den aktuella grenen
nämner kommer först, markerat *(från grenen)*. `Enter` eller ett klick infogar dess referens
(`ABC-123`, `#12`, `AB#12`) vid markören. Tangentbord: `Alt+R` i composern.
Palett: *Commit: Infoga ärendereferens...*. Utan en länkad ärendelista är
knappen nedtonad och säger *länka först en ärendelista i sidofältet*.
GitPoulpi lägger aldrig till en referens av sig självt.

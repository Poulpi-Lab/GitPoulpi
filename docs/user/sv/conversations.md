[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · **Svenska** · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Konversationer

Konversationsvyn för ett ärende, ett arbetsobjekt, ett kort eller en pull / merge request. Den kräver ett konto och ett länkat repository: se [Värdtjänster och ärendehantering](hosting-integration.md); listorna som öppnar den finns i [Pull requests och ärenden](pull-requests-and-issues.md); att granska kod finns i [Granska en pull request](review.md).

En **konversation** är allt som sagts om ett ärende, ett arbetsobjekt, ett kort, en ändring eller en pull / merge request: beskrivningen,
kommentarerna, granskningarna, trådarna på kod och händelserna (etiketter, tilldelningar, tillståndsändringar,
pushar). GitPoulpi läser den från värden, visar den formaterad och låter dig svara utan att lämna fönstret.

### Öppna en

| Från | Hur |
|---|---|
| En ÄRENDEN-rad | `Enter` eller ett dubbelklick; radmenyns **Öppna konversation**; paletten *Värd: Öppna ärendekonversation...* (en lista med ärendena på skärmen). Radmenyns **Öppna i webbläsare** behåller webbläsaren. |
| En PULL REQUESTS-rad på Azure DevOps eller Bitbucket | `Enter` eller **Öppna konversation**. På GitHub, GitLab, Gitea, Forgejo och Gerrit öppnar raden [granskningen](review.md), vars Översikt-sida är samma konversation (tidslinjen, tangenterna och kommentarsrutan nedan). |
| Ett kort på [ärendetavlan](pull-requests-and-issues.md#ärendetavlan) | `Enter` eller ett dubbelklick. |
| [Radar](radar.md) | `i` på en ärende- eller begäranrad, radmenyns **Öppna konversation**, eller *Radar: Öppna ärendekonversation*. |
| En referens i en annan konversation | Klicka på den (eller `l` och välj den): `#7`, `!12`, `ABC-12` i en länkad källa öppnas i appen och läggs på en Tillbaka-stack. |
| En agent | `gitpoulpi ui show issue <n> --view conversation` (också `ui show pr`), se [CLI-referensen](cli-reference.md). |

**Repositoryn som inte är klonade.** En Radar-rad för ett repository som du inte har någon klon av öppnar också sin konversation:
vyn behöver bara integrationsprofilen och projektet, så Radar visar den över sitt eget fönster, och `Esc`
ger tillbaka Radar. Det som kräver en klon utelämnas: en commit-referens öppnas hos värden i stället för i Log, och
*Låt AI:n läsa repositoryt* är nedtonat med sin orsak (se [AI-kapitlet](ai.md#skrivhjälp-på-en-värd)). En
begäran i ett klonat repository har sin granskning i stället (`e`), och Radars **Öppna konversation** säger att det inte finns någon
konversation att läsa här för den; ett ärende i ett klonat repository öppnar den i det repositoryts flik.

Ett ärende på en värd som inte har något att läsa (Bitbucket och Gerrit har inga ärenden) öppnar webbläsaren som förut, och ett bygge
utan funktionen `conversation` gör likadant överallt ([anpassade byggen](custom-builds.md)).

### Vyn

En helfönstervy: rubriken (tillstånd, vem som öppnade det och när, etiketterna, antalet kommentarer, **Visa händelser**,
**Tillbaka**, **Uppdatera**, **Öppna i webbläsare**), raden **Publicerar som**, tidslinjen och under den kommentarsrutan.

![Konversationen för ärende #7: beskrivningen med en länk till #9, två kommentarer, en etikettshändelse som visas av Visa händelser, och den tomma rutan Ny kommentar med Skriv, Förhandsvy och Delad](../images/issue-conversation.png)

![Sidan Översikt för merge request !58: beskrivningen, en olöst och en löst kodtråd med sina fil- och radlänkar, kommentarerna, Svara och Citera på den markerade tråden, och kommentarsrutan](../images/pr-conversation.png)

- Tidslinjen läses äldst till nyast och börjar på det nyaste fönstret (högst 300 poster). **Läs in äldre** överst
  (`m`, paletten *Konversation: Läs in äldre poster*) läser fönstret före, med antalet som återstår
  när värden anger det.
- En kommentar visar sin författare, datum, ett märke `ändrad`, ett märke `bot` för botar, `intern` för privata anteckningar, och
  knapparna **Svara**, **Citera**, **Redigera**, **Ta bort** och **Länkar** som den valda posten tillåter. En tråd på kod
  visar `sökväg:rad`, `löst` eller `olöst`, `föråldrad` och sina kommentarer. En granskning visar sitt omdöme.
- Händelser är en dämpad rad var (*carol lade till etiketten lfs*). **Visa händelser** (`v`) döljer dem; valet sparas
  per repository. Värdar utan händelsehistorik (YouTrack, work items på Azure DevOps, lokala anteckningar) har ingen sådan ruta.
- Texten visas som renderad Markdown: rubriker, nästlade listor och uppgiftslistor, citat, varningsblock i GitHub-stil, markerad kod,
  tabeller, länkar, bilder som `[image: alt]`. Text som kom i värdens eget markeringsspråk konverteras först (se
  [Markupkonvertering](#markupkonvertering)). Ett kodblock `mermaid` får en knapp **Öppna diagram** som visar
  diagrammets källtext i en rullbar visare (`Enter` kopierar den); paletten har *Konversation: Öppna postens diagram*.
  Det kräver ett bygge med både `conversation` och `mermaid`.
- I granskningens Översikt är en kodtråds `path:line` en länk till koden (`G`, *Konversation: Öppna trådens kod*;
  se [Granskning](review.md)); i en konversation som öppnats för sig (ett ärende, en Radar-rad) finns ingen kodsida att gå till.
- Vyn läser igen var 60:e sekund och med `F5` (**Uppdatera**).

**Tangenter.** Var och en har en knapp, en menyrad eller en palettrad (*Konversation: ...*), och `Ctrl+N` märker knapparna.

| Tangent | Åtgärd |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Föregående / nästa post |
| `PageUp`, `PageDown`, `Home`, `End`, hjulet, rullningslisten | Rulla |
| `Tab`, `Shift+Tab` | Tidslinje, kommentarsruta, sedan rubrikens knappar |
| `r` | Svara: i den valda postens tråd, eller med dess text citerad där värden saknar trådar |
| `Q` | Citera den valda posten i en ny kommentar |
| `c` | Ny kommentar |
| `e`, `d` | Redigera, ta bort en av dina egna kommentarer (en fråga kommer före en borttagning) |
| `l`, `Enter` | Postens länkar: `Enter` följer den enda, eller öppnar menyn med dem |
| `y`, `o` | Kopiera postens adress, öppna den i webbläsaren |
| `n` | Nästa olästa post |
| `v`, `m`, `F5` | Visa eller dölj händelser, Läs in äldre, Uppdatera |
| `Alt+Left`, `Backspace` | Tillbaka längs Tillbaka-stacken |
| `q`, `Esc` | Stäng (frågar först när rutan innehåller text) |

### Skriva

**Ny kommentar** (`c`, eller `Tab` till rutan), **Svara** (`r`), **Citera** (`Q`) och **Redigera** (`e`) använder den enda Markdown-
redigeraren: **Skriv**, **Förhandsvy** och **Delad** (`Alt+P` växlar mellan Skriv och Förhandsvy, `Alt+L` slår på Delad och kräver ett
fönster som är 100 kolumner brett). `Ctrl+Enter` (också `Ctrl+S` och `Alt+Enter`) eller **Skicka** publicerar, **Avbryt** släpper rutan, och
`Alt+E` öppnar **Skriv med AI** där AI är på ([AI-kapitlet](ai.md#skrivhjälp-på-en-värd)). Förhandsvyn läggs
ut av samma renderare som tidslinjen, med samma referenser. När värden inte tar emot Markdown nämner
sidfoten markeringsspråket texten skickas i (*Skickas som Jira wiki markup*, *Skickas som Textile*).

![Ett ärendesvar som skrivs i vyn Delad: Markdown-utkastet till vänster, den renderade förhandsvyn till höger och Skicka](../images/issue-reply-composer.png)

- Kommentaren läggs till i tidslinjen direkt och tar sin plats när värden svarar. Om värden vägrar försvinner
  posten och rutan öppnas igen med din text och orsaken.
- En text du inte har skickat sparas per objekt (`remote-conversations.toml` i konfigurationsmappen) och finns där nästa
  gång du öppnar rutan; att skicka eller förkasta den tar bort den. Att stänga en ruta som innehåller text frågar först.
- **Svara** på en tråd (en granskningstråd, en GitLab-diskussion, en Linear-kommentar, en tråd på Bitbucket eller Azure DevOps)
  publicerar i den tråden. Där värden bara har en platt lista startar **Svara** en ny kommentar med den valda posten
  citerad (`> **författare**`), på samma sätt som **Citera**.
- **Intern anteckning** (ruta, `Alt+I`): på GitLab och Redmine är kommentaren bara synlig för projektets medlemmar. Den
  visas bara på värdar som har sådana anteckningar, och bara på en ny kommentar.
- Att publicera är en utåtriktad åtgärd: inget journalförs och **inget kan ångras från GitPoulpi**. Utan
  anslutning stannar rutan, **Skicka** är nedtonad med orsaken, och rubriken säger att den sparade kopian visas.

**Vem som får redigera eller ta bort.** Bara dina egna kommentarer har **Redigera** och **Ta bort**. Värdens eget ord kommer först när
den ger ett (GitHubs `viewerCanUpdate` och `viewerCanDelete`, `can_edit` på andra), därefter ditt login mot
kommentarens författare, jämfört utan hänsyn till versaler; en App-bots kommentarer måste dessutom komma från ett bot-konto. Så länge
loginet för en GitHub App-bot inte är känt räknas bara kommentarer som den här installationen publicerade (repositoryt behåller deras id:n):
inget gissas utifrån ett namn. I en tråd verkar **Redigera** och **Ta bort** på den sista kommentaren i den som är din. En värd
som inte kan redigera eller ta bort (se [tabellen](#vad-varje-värd-inte-kan-göra)) visar knapparna nedtonade med orsaken.

### Publicera som

Raden under rubriken anger vem dina meddelanden publiceras som, och samma rad finns i varje ruta:

| Rad | Betydelse |
|---|---|
| *Publicerar som ann (personlig token)* | Profilens token: ditt eget login (`GET /user`). |
| *Publicerar som ci-bot (repositoryts tokenkommando)* | Repositoryts [tokenkommando](hosting-integration.md#8-att-agera-som-en-github-app-bot) används för värd-API:et. |
| *Publicerar som my-app[bot] (GitHub App-bot)* | En installationstoken. Namnet kommer från **Publicera som**, annars från värden. |
| *Publicerar som GitHub App-boten (dess namn är inte känt än: ställ in Publicera som i Inställningar)* | Inget namn inställt, inget inlärt än. |
| *Publicerar som ... (projektbot)* | En GitLab-åtkomsttoken för projekt eller grupp. |
| *Skriver som kai (lokal anteckning)* | En mapp med Markdown-anteckningar: profilens `local.me`, annars den anslutna användaren. |
| *Vem du publicerar som är inte känt än. Ställ in Publicera som i Inställningar.* | Inget kunde läras. |

**Publicera som** är ett namn: för en GitHub App är det `my-app[bot]`. Ställ in det för alla repositoryn i **Inställningar > Integrationer >
Publicera som** (`general.hosting_post_as`), eller för ett repository i **Inställningar > Repo-specifikt: SSH & inloggning > Värd**,
som åsidosätter det. Lämnas det tomt frågar GitPoulpi GitHub (GraphQL `viewer { login }`, som en installationstoken kan
svara på eller inte) och lär sig annars namnet från författaren till den första kommentar den publicerar och behåller det med
repositoryt. En personlig token behöver det aldrig. Se
[Att agera som en GitHub App-bot](hosting-integration.md#8-att-agera-som-en-github-app-bot).

### Korsreferenser

Referenser i en konversation är understrukna och klickbara. Repositoryts länkade ÄRENDEN- och PULL REQUESTS-källor
matchas först, så `ABC-12` i ett länkat Jira-projekt öppnas i appen även inuti en GitHub-kommentar; det värdens
egen grammatik lägger till öppnar dess sida efter länkbekräftelsen. Ett commit-id öppnas i Log när repositoryt
är klonat och id:t har minst 7 tecken, annars hos värden. Inget inuti en kodsnutt, ett kodblock, en
Markdown-länk, en e-postadress eller en URL är en referens, och `C#1`, `UTF-8`, `SHA-256` och `ISO-8601` är det inte.

| Värd | Känns igen |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, commit-id:n |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (ett syskonprojekt), `@user`, `~label` och `%milestone` som chip, commit-id:n |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, commit-id:n |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, `{uuid}`-omnämnanden, commit-id:n |
| Gerrit | `Change-Id`-värden (`I` och 40 hexadecimala siffror), commit-id:n |
| Redmine | `#7`, `r123`-revisioner, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, bara för ett projekt eller team i en länkad källa |
| Trello | kortadresser (`trello.com/c/...`) |
| Lokala anteckningar | `[[note]]`, `[[note\|alias]]` och relativa `.md`-länkar till en annan anteckning i mappen |

`gitpoulpi hosting markdown --body T --id I` skriver ut vad appen gör av en text, referenser inräknade.

### Markupkonvertering

Värdar som inte talar Markdown konverteras åt båda hållen, så att du läser och skriver Markdown överallt. Det en
konverterare inte förstår förblir läsbar text.

- **Jira** (Cloud och Data Center) lagrar kommentarer som wiki markup över API v2. Det du läser konverteras till
  Markdown; det du skriver konverteras till wiki markup, och sidfoten säger *Skickas som Jira wiki markup*.
- **Redmine** formaterar text som Textile eller Markdown, vilket API:et inte säger: profilens rad **Serverns textformat**
  (Inställningar > Integrationer, `markup`, Textile som standard) talar om det för GitPoulpi. Textile konverteras till
  Markdown för läsning och tillbaka till Textile för skrivning.
- **Azure DevOps Server** returnerar kommentarer på work items som HTML och tar emot dem som HTML: de visas som Markdown
  konverterad från den och skrivs som HTML. Azure DevOps Services och pull request-trådar tar emot Markdown.
- **Gerrit** är ren text: den renderas inte som Markdown och det du skriver skickas som du skrev det.
- Varje importerare tar först bort styr- och escape-sekvenser, så en kommentar kan inte flytta terminalen.

### Den sparade kopian, Läs in äldre, olästa

- Varje konversation du öppnar sparas (`<tillståndsmapp>/conversations/<integration>/<projekt>/<sort>-<n>.json`, bara för användaren,
  ingen token i den, högst 2 MB per fil, de 50 senaste filerna behålls). Den ritas direkt när objektet öppnas och
  uppdateras bakom den. När värden inte kan nås ligger den sparade kopian kvar på skärmen under *Offline: visar kopian som
  sparades för 3 min sedan (orsak)* och Skicka är nedtonad.
- **Olästa:** GitPoulpi minns per konversation vad du såg senast (de 200 senaste behålls). Poster av någon annan
  som är nyare får en accentstapel och ett märke `ny`, och `n` hoppar till nästa. En konversation du öppnar för
  första gången har inget nytt.

### Vad varje värd inte kan göra

Varje `nej` nedan är en verklig gräns: kontrollen döljs eller tonas ned med orsaken, och CLI:t svarar med ett fel som nämner
värden innan något skickas.

| Värd | Objekt | Svara i en tråd | Redigera och ta bort egna | Lösa en tråd | Händelser | Interna anteckningar | Skrivs som |
|---|---|---|---|---|---|---|---|
| GitHub | ärenden, pull requests | pull requests | båda | pull requests | ja | nej | GitHub Markdown |
| GitLab | ärenden, merge requests | ja (diskussioner) | båda | merge requests | ja | ja | GitLab Markdown |
| Gitea | ärenden, pull requests | pull requests | båda | pull requests | ja | nej | Markdown |
| Forgejo | ärenden, pull requests | pull requests | båda | nej (värden har ingen ändpunkt) | ja | nej | Markdown |
| Azure DevOps | work items, pull requests | pull requests | båda | nej (status visas, ändras inte) | bara pull requests | nej | Markdown (HTML på work items i Server) |
| Bitbucket (Cloud, Data Center) | bara pull requests | ja | båda | nej (visas, ändras inte) | ja | nej | Markdown |
| Gerrit | bara ändringar | ja | nej (ingetdera kan redigeras eller tas bort) | ja | ja | nej | ren text |
| Jira | ärenden | nej (platt lista) | båda | nej | ja (changelog) | nej (kräver en projektroll) | wiki markup |
| Linear | ärenden | ja (svar på en kommentar) | båda | nej | ja | nej | Markdown |
| YouTrack | ärenden | nej | båda | nej | nej | nej | Markdown |
| Redmine | ärenden | nej | bara redigera (Redmine 5 och senare), aldrig ta bort | nej | ja (journaler) | ja (privata anteckningar) | Markdown eller Textile |
| Trello | kort | nej | båda | nej | ja (flyttar) | nej | Markdown |
| Lokala anteckningar | anteckningar | nej | den lokala författarens egna | nej | nej | nej | Markdown |

Dessutom: ingen värd låter dig starta en ny tråd på en rad från konversationen (gör det i granskningen), och reaktioner visas
bara på GitHub. En text som är längre än värden tar emot avvisas (GitHub 65 536 tecken, GitLab 1 000 000, Azure
DevOps 150 000, Jira 32 767, Trello 16 384): CLI:t kontrollerar först, i appen visas värdens eget meddelande och din
text sparas. En lokal anteckning behåller sina kommentarer i ett avslutande avsnitt `## Comments` i själva anteckningsfilen, en
post `### författare - tid` var, så att Obsidian och ett skal ser dem också.

På kommandoraden är samma konversationer `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit`
och `comment-delete`, och `hosting review conversation` (se [CLI-referensen](cli-reference.md)); för agenter är de
MCP-verktyg med samma namn (se [MCP och agenter](mcp-and-agents.md)).

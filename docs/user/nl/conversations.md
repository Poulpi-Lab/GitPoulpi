[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · **Nederlands** · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Gesprekken

De gespreksweergave van een issue, werkitem, kaart of pull / merge request. Ze heeft een account en een gekoppelde repository nodig: zie [Integraties met hosting en issuetrackers](hosting-integration.md); de lijsten die haar openen staan in [Pull requests en issues](pull-requests-and-issues.md); code reviewen staat in [Een pull of merge request reviewen](review.md).

Een **gesprek** is alles wat over één issue, werkitem, kaart, wijziging of pull / merge request is gezegd: de
beschrijving, de opmerkingen, de reviews, de discussies bij code en de gebeurtenissen (labels, toewijzingen,
statuswijzigingen, pushes). GitPoulpi leest het van de host, toont het opgemaakt en laat je antwoorden zonder het venster te
verlaten.

### Een gesprek openen

| Vanuit | Hoe |
|---|---|
| Een rij ISSUES | `Enter` of een dubbelklik; **Gesprek openen** in het rijmenu; palet *Hosting: Issuegesprek openen...* (een lijst van de issues op het scherm). **Openen in browser** in het rijmenu houdt de browser. |
| Een rij PULL REQUESTS op Azure DevOps of Bitbucket | `Enter` of **Gesprek openen**. Op GitHub, GitLab, Gitea, Forgejo en Gerrit opent de rij de [review](review.md), waarvan de pagina Overzicht ditzelfde gesprek is (de tijdlijn, de toetsen en het opmerkingenvak eronder). |
| Een kaart van het [issuebord](pull-requests-and-issues.md#het-issuebord) | `Enter` of een dubbelklik. |
| De [Radar](radar.md) | `i` op een issue- of verzoekrij, **Gesprek openen** in het rijmenu, of *Radar: Issuegesprek openen*. |
| Een verwijzing in een ander gesprek | Klik erop (of `l`, kies dan): `#7`, `!12`, `ABC-12` van een gekoppelde bron openen in de app en komen op een Terug-stapel. |
| Een agent | `gitpoulpi ui show issue <n> --view conversation` (ook `ui show pr`), zie [de CLI-naslag](cli-reference.md). |

**Repositories die niet gekloond zijn.** Een Radar-rij van een repository waarvan je geen kloon hebt, opent ook zijn gesprek: de
weergave heeft alleen het integratieprofiel en het project nodig, dus de Radar toont hem over zijn eigen venster, en `Esc` geeft
de Radar terug. Wat een kloon nodig heeft, valt weg: een commitverwijzing opent op de host in plaats van in het Log, en
*AI de repository laten lezen* is grijs met zijn reden (zie [het hoofdstuk over AI](ai.md#schrijfhulp-op-een-host)). Een verzoek
van een gekloonde repository heeft in plaats daarvan zijn review (`e`), en **Gesprek openen** in de Radar zegt dat er hier
geen gesprek te lezen is; een issue van een gekloonde repository opent het in het tabblad van die repository.

Een issue van een host die er geen heeft om te lezen (Bitbucket en Gerrit hebben geen issues) opent de browser zoals eerder, en een
build zonder de feature `conversation` doet overal hetzelfde ([aangepaste builds](custom-builds.md)).

### De weergave

Een weergave over het hele venster: de kop (status, wie het opende en wanneer, de labels, het aantal opmerkingen,
**Gebeurtenissen tonen**, **Terug**, **Vernieuwen**, **Openen in browser**), de regel **Plaatsen als**, de tijdlijn, en eronder het
opmerkingsvak.

![Het gesprek van issue #7: de beschrijving met een link naar #9, twee opmerkingen, een labelgebeurtenis getoond door Gebeurtenissen tonen, en het lege vak Nieuwe opmerking met Schrijven, Voorbeeld en Gesplitst](../images/issue-conversation.png)

![De pagina Overzicht van merge request !58: de beschrijving, een onopgeloste en een opgeloste codediscussie met hun bestands- en regellinks, de opmerkingen, Antwoorden en Citeren op de geselecteerde discussie, en het opmerkingenvak](../images/pr-conversation.png)

- De tijdlijn leest van oud naar nieuw en begint op het nieuwste venster (hoogstens 300 items). **Oudere laden** bovenaan
  (`m`, palet *Gesprek: Oudere items laden*) leest het venster ervoor, met het aantal dat nog komt als de host dat zegt.
- Een opmerking toont zijn auteur, datum, een chip `bewerkt`, een chip `bot` voor bots, `intern` voor privénotities, en de
  knoppen **Antwoorden**, **Citeren**, **Bewerken**, **Verwijderen** en **Links** die het geselecteerde item toestaat. Een
  discussie bij code toont `pad:regel`, `opgelost` of `niet opgelost`, `verouderd`, en zijn opmerkingen. Een review toont zijn
  oordeel.
- Gebeurtenissen zijn elk één gedempte regel (*carol voegde het label lfs toe*). **Gebeurtenissen tonen** (`v`) verbergt ze; de
  keuze wordt per repository bewaard. Hosts zonder gebeurtenissengeschiedenis (YouTrack, werkitems van Azure DevOps, lokale
  notities) hebben dat selectievakje niet.
- De tekst wordt als Markdown weergegeven: koppen, geneste lijsten en takenlijsten, citaten, meldingen in GitHub-stijl,
  gemarkeerde code, tabellen, links, afbeeldingen als `[image: alt]`. Tekst die in de eigen opmaak van de host binnenkwam, wordt
  eerst omgezet (zie [Opmaakconversie](#opmaakconversie)). Een codeblok `mermaid` krijgt een knop **Diagram openen** die de bron
  van het diagram in een scrollbare weergave toont (`Enter` kopieert hem); het palet heeft *Gesprek: Diagram van item openen*.
  Het vereist een build met zowel `conversation` als `mermaid`.
- In het Overzicht van de review is de `pad:regel` van een codediscussie een link naar de code (`G`, *Gesprek: Code van discussie openen*;
  zie [Review](review.md)); in een gesprek dat los is geopend (een issue, een Radar-rij) is er geen codepagina om naartoe te gaan.
- De weergave leest elke 60 seconden opnieuw en bij `F5` (**Vernieuwen**).

**Toetsen.** Elke heeft een knop, een menurij of een paletrij (*Gesprek: ...*), en `Ctrl+N` geeft de knoppen een badge.

| Toets | Actie |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Vorig / volgend item |
| `PageUp`, `PageDown`, `Home`, `End`, het wiel, de schuifbalk | Scrollen |
| `Tab`, `Shift+Tab` | Tijdlijn, opmerkingsvak, daarna de knoppen in de kop |
| `r` | Antwoorden: in de discussie van het geselecteerde item, of met zijn tekst geciteerd waar de host geen discussies heeft |
| `Q` | Het geselecteerde item citeren in een nieuwe opmerking |
| `c` | Nieuwe opmerking |
| `e`, `d` | Een van je eigen opmerkingen bewerken, verwijderen (voor een verwijdering komt een vraag) |
| `l`, `Enter` | De links van het item: `Enter` volgt de enige, of opent het menu ervan |
| `y`, `o` | Het adres van het item kopiëren, het in de browser openen |
| `n` | Volgend nieuw item |
| `v`, `m`, `F5` | Gebeurtenissen tonen of verbergen, Oudere laden, Vernieuwen |
| `Alt+Left`, `Backspace` | Terug langs de Terug-stapel |
| `q`, `Esc` | Sluiten (vraagt eerst als het vak tekst bevat) |

### Schrijven

**Nieuwe opmerking** (`c`, of `Tab` naar het vak), **Antwoorden** (`r`), **Citeren** (`Q`) en **Bewerken** (`e`) gebruiken
dezelfde Markdown-composer: **Schrijven**, **Voorbeeld** en **Gesplitst** (`Alt+P` wisselt Schrijven en Voorbeeld, `Alt+L` zet
Gesplitst aan en vraagt een venster van 100 kolommen). `Ctrl+Enter` (ook `Ctrl+S` en `Alt+Enter`) of **Verzenden** plaatst,
**Annuleren** laat het vak vallen, en `Alt+E` opent **Schrijven met AI** waar AI aan staat ([het hoofdstuk over
AI](ai.md#schrijfhulp-op-een-host)). Het voorbeeld wordt door dezelfde renderer opgemaakt als de tijdlijn, met dezelfde
verwijzingen. Als de host geen Markdown aanneemt, noemt de voettekst de opmaak waarin de tekst wordt verstuurd (*Verzonden als
Jira wiki markup*, *Verzonden als Textile*).

![Een antwoord op een issue dat in de weergave Gesplitst wordt geschreven: het Markdown-concept links, het opgemaakte voorbeeld rechts, en Verzenden](../images/issue-reply-composer.png)

- De opmerking wordt direct aan de tijdlijn toegevoegd en neemt zijn plaats in als de host antwoordt. Weigert de host, dan
  verdwijnt het item en opent het vak opnieuw met je tekst en de reden.
- Een tekst die je niet hebt verzonden, wordt per item bewaard (`remote-conversations.toml` in de configuratiemap) en staat er de
  volgende keer dat je dat vak opent; verzenden of weggooien verwijdert hem. Een vak met tekst sluiten vraagt eerst.
- **Antwoorden** op een discussie (een reviewdiscussie, een GitLab-discussie, een Linear-opmerking, een discussie van Bitbucket
  of Azure DevOps) plaatst in die discussie. Waar de host alleen een platte lijst heeft, begint **Antwoorden** een nieuwe
  opmerking met het geselecteerde item geciteerd (`> **auteur**`), net als **Citeren**.
- **Interne notitie** (selectievakje, `Alt+I`): op GitLab en Redmine is de opmerking alleen zichtbaar voor projectleden. Het
  wordt alleen getoond op hosts die zulke notities hebben, en alleen bij een nieuwe opmerking.
- Plaatsen is een uitgaande actie: niets wordt gejournaliseerd en **niets kan vanuit GitPoulpi ongedaan worden gemaakt**. Zonder
  verbinding blijft het vak staan, is **Verzenden** grijs met de reden, en zegt de kop dat de bewaarde kopie wordt getoond.

**Wie mag bewerken of verwijderen.** Alleen je eigen opmerkingen hebben **Bewerken** en **Verwijderen**. Het eigen woord van de host
gaat voor als hij er een geeft (`viewerCanUpdate` en `viewerCanDelete` van GitHub, elders `can_edit`), daarna je login tegen de
auteur van de opmerking, vergeleken zonder hoofdlettergevoeligheid; opmerkingen van een App-bot moeten ook van een botaccount
komen. Zolang de login van een GitHub App-bot niet bekend is, tellen alleen opmerkingen die deze installatie plaatste (de
repository bewaart hun id's): uit een naam wordt niets geraden. In een discussie werken **Bewerken** en **Verwijderen** op de
laatste opmerking ervan die van jou is. Een host die niet kan bewerken of verwijderen (zie [de tabel](#wat-elke-host-niet-kan))
toont de knoppen grijs met de reden.

### Plaatsen als

De regel onder de kop zegt als wie je berichten worden geplaatst, en dezelfde regel staat in elk vak:

| Regel | Betekenis |
|---|---|
| *Plaatsen als ann (persoonlijk token)* | Het token van het profiel: je eigen login (`GET /user`). |
| *Plaatsen als ci-bot (tokencommando van de repository)* | Het [tokencommando](hosting-integration.md#8-optreden-als-github-app-bot) van de repository wordt voor de host-API gebruikt. |
| *Plaatsen als my-app[bot] (GitHub App-bot)* | Een installatietoken. De naam komt van **Plaatsen als**, anders van de host. |
| *Plaatsen als de GitHub App-bot (zijn naam is nog niet bekend: stel Plaatsen als in bij Instellingen)* | Geen naam ingesteld, nog geen geleerd. |
| *Plaatsen als ... (projectbot)* | Een toegangstoken van een GitLab-project of -groep. |
| *Schrijven als kai (lokale notitie)* | Een map Markdown-notities: de `local.me` van het profiel, anders de verbonden gebruiker. |
| *Nog niet bekend als wie je zou plaatsen. Stel Plaatsen als in bij Instellingen.* | Er kon niets worden geleerd. |

**Plaatsen als** is een naam: voor een GitHub App is het `my-app[bot]`. Stel hem in voor elke repository in **Instellingen >
Integraties > Plaatsen als** (`general.hosting_post_as`), of voor één repository in **Instellingen > Repositoryspecifieke
instellingen > SSH & inloggegevens > Hosting**, wat voorgaat. Leeg gelaten vraagt GitPoulpi het aan GitHub (GraphQL `viewer {
login }`, wat een installatietoken wel of niet kan beantwoorden) en leert anders de naam van de auteur van de eerste opmerking
die het plaatst en bewaart hem bij de repository. Een persoonlijk token heeft hem nooit nodig. Zie
[Optreden als GitHub App-bot](hosting-integration.md#8-optreden-als-github-app-bot).

### Kruisverwijzingen

Verwijzingen in een gesprek zijn onderstreept en klikbaar. De gekoppelde bronnen ISSUES en PULL REQUESTS van de repository worden
eerst gematcht, dus `ABC-12` van een gekoppeld Jira-project opent in de app, zelfs binnen een GitHub-opmerking; wat de eigen
grammatica van de host toevoegt, opent zijn pagina na de linkbevestiging. Een commit-id opent in het Log als de repository
gekloond is en het id minstens 7 tekens heeft, anders op de host. Niets in een codefragment, een afgeschermd blok, een
Markdown-link, een e-mailadres of een URL is een verwijzing, en `C#1`, `UTF-8`, `SHA-256` en `ISO-8601` ook niet.

| Host | Herkend |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, commit-id's |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (een zusterproject), `@user`, `~label` en `%milestone` als chips, commit-id's |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, commit-id's |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, vermeldingen met `{uuid}`, commit-id's |
| Gerrit | `Change-Id`-waarden (`I` en 40 hexadecimale cijfers), commit-id's |
| Redmine | `#7`, revisies `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, alleen voor een project of team van een gekoppelde bron |
| Trello | kaartadressen (`trello.com/c/...`) |
| Lokale notities | `[[note]]`, `[[note\|alias]]` en relatieve `.md`-links naar een andere notitie van de map |

`gitpoulpi hosting markdown --body T --id I` drukt af wat de app van een tekst maakt, verwijzingen inbegrepen.

### Opmaakconversie

Hosts die geen Markdown spreken, worden in beide richtingen omgezet, zodat je overal Markdown leest en schrijft. Wat een
converter niet begrijpt, blijft leesbare tekst.

- **Jira** (Cloud en Data Center) bewaart opmerkingen als wiki-markup via API v2. Wat je leest, wordt naar Markdown omgezet; wat
  je schrijft, wordt naar wiki-markup omgezet, en de voettekst zegt *Verzonden als Jira wiki markup*.
- **Redmine** maakt tekst op als Textile of Markdown, wat de API niet zegt: de rij **Tekstindeling van de server** van het profiel
  (Instellingen > Integraties, `markup`, standaard Textile) vertelt het aan GitPoulpi. Textile wordt voor het lezen naar Markdown
  omgezet en voor het schrijven terug naar Textile.
- **Azure DevOps Server** geeft opmerkingen van werkitems als HTML en neemt ze als HTML aan: ze worden getoond als de Markdown
  die ervan is omgezet en als HTML geschreven. Azure DevOps Services en discussies van pull requests nemen Markdown.
- **Gerrit** is platte tekst: het wordt niet als Markdown weergegeven en wat je schrijft, wordt verstuurd zoals getypt.
- Elke importeur verwijdert eerst stuur- en escapesequenties, zodat een opmerking de terminal niet kan verplaatsen.

### De bewaarde kopie, Oudere laden, ongelezen

- Elk gesprek dat je opent, wordt bewaard (`<statusmap>/conversations/<integratie>/<project>/<soort>-<n>.json`, alleen voor de
  gebruiker, geen token erin, hoogstens 2 MB per bestand, de 50 meest recente bestanden bewaard). Het wordt direct getekend als
  het item opent en erachter vernieuwd. Als de host niet bereikbaar is, blijft de bewaarde kopie op het scherm onder *Offline:
  kopie getoond die 3 min geleden is bewaard (reden)* en is Verzenden grijs.
- **Ongelezen:** wat je het laatst zag, wordt per gesprek onthouden (de laatste 200 worden bewaard). Items van iemand anders die
  nieuwer zijn, krijgen een accentbalk en een chip `nieuw`, en `n` springt naar het volgende. Een gesprek dat je voor het eerst
  opent, heeft niets nieuws.

### Wat elke host niet kan

Elke `nee` hieronder is een echte limiet: het bedieningselement is verborgen of grijs met de reden, en de CLI antwoordt met een
fout die de host noemt voordat er iets wordt verstuurd.

| Host | Items | Antwoord in een discussie | Eigen bewerken en verwijderen | Een discussie oplossen | Gebeurtenissen | Interne notities | Geschreven als |
|---|---|---|---|---|---|---|---|
| GitHub | issues, pull requests | pull requests | beide | pull requests | ja | nee | GitHub Markdown |
| GitLab | issues, merge requests | ja (discussies) | beide | merge requests | ja | ja | GitLab Markdown |
| Gitea | issues, pull requests | pull requests | beide | pull requests | ja | nee | Markdown |
| Forgejo | issues, pull requests | pull requests | beide | nee (de host heeft geen endpoint) | ja | nee | Markdown |
| Azure DevOps | werkitems, pull requests | pull requests | beide | nee (status getoond, niet gewijzigd) | alleen pull requests | nee | Markdown (HTML bij werkitems van Server) |
| Bitbucket (Cloud, Data Center) | alleen pull requests | ja | beide | nee (getoond, niet gewijzigd) | ja | nee | Markdown |
| Gerrit | alleen wijzigingen | ja | nee (geen van beide kan worden bewerkt of verwijderd) | ja | ja | nee | platte tekst |
| Jira | issues | nee (platte lijst) | beide | nee | ja (changelog) | nee (vraagt een projectrol) | wiki-markup |
| Linear | issues | ja (antwoorden op een opmerking) | beide | nee | ja | nee | Markdown |
| YouTrack | issues | nee | beide | nee | nee | nee | Markdown |
| Redmine | issues | nee | alleen bewerken (Redmine 5 en later), nooit verwijderen | nee | ja (journals) | ja (privénotities) | Markdown of Textile |
| Trello | kaarten | nee | beide | nee | ja (verplaatsingen) | nee | Markdown |
| Lokale notities | notities | nee | alleen die van de lokale auteur | nee | nee | nee | Markdown |

Ook: geen host laat je vanuit het gesprek een nieuwe discussie bij een regel beginnen (doe dat in de review), en reacties worden
alleen op GitHub getoond. Een tekst die langer is dan de host aanneemt, wordt geweigerd (GitHub 65.536 tekens, GitLab 1.000.000,
Azure DevOps 150.000, Jira 32.767, Trello 16.384): de CLI controleert eerst, in de app wordt het eigen bericht van de host getoond
en blijft je tekst bewaard. Een lokale notitie bewaart zijn opmerkingen in een laatste sectie `## Comments` van het notitiebestand
zelf, elk als een item `### auteur - tijd`, dus ook Obsidian en een shell zien ze.

Op de opdrachtregel zijn dezelfde gesprekken `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit` en
`comment-delete`, en `hosting review conversation` (zie [de CLI-naslag](cli-reference.md)); voor agents zijn het MCP-tools met
dezelfde namen (zie [MCP en agents](mcp-and-agents.md)).


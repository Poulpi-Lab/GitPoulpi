[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · **Nederlands** · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull requests en issues

Pull / merge requests en issues lezen, maken en beheren vanuit de zijbalk, het bord en de werkbalk. Verbind eerst een account en koppel een repository: zie [Integraties met hosting en issuetrackers](hosting-integration.md). Alles over één issue of verzoek (opmerkingen, reviews, discussies) staat in [Gesprekken](conversations.md); een verzoek lezen en reviewen staat in [Een pull of merge request reviewen](review.md).

## Pull requests en issues in de zijbalk

### PULL REQUESTS

De sectie heeft een kop met het aantal en een `+` (een pull request maken),
de keuzelijsten voor provider en project, een zoekvak en een statuskeuzelijst (*Open*,
*Merged / gesloten*, *Alle*). Daarna komen vier groepen, elk met hun aantal:

- **Mijn Pull Requests**: jij bent de auteur.
- **Aan mij toegewezen**.
- **Wacht op mijn review**: jij bent een gevraagde reviewer.
- **Alle Pull Requests**: aan het begin ingeklapt.

Het zoekvak filtert elke groep (titel, nummer, auteur, branch). Elke wijziging van de
statuskeuzelijst vraagt de host opnieuw.

Een rij begint met een statussymbool: `●` open (groen), `○` concept, `◆` merged
(paars), `✕` gesloten (rood). Het verzoek van de uitgecheckte branch is vetgedrukt.
Rijen lezen `#12  title` (GitHub, Bitbucket) of `!53  title` (GitLab,
Azure DevOps).

### ISSUES

De sectietitel noemt de provider: `GITHUB ISSUES`, `JIRA ISSUES`, enzovoort.
Hij leest `... WERKITEMS` voor Azure DevOps en `... KAARTEN` voor Trello.
Onder de keuzelijsten: een zoekvak, de statuskeuzelijst (*Open*,
*Aan mij toegewezen*, *Gesloten*, *Alle*) en de issues. Een Jira-rij leest
`ABC-12  title`, met het type en de status gedimd als de zijbalk breed genoeg
is.

![De sectie ISSUES van de zijbalk met het rechtsklikmenu van issue #7 open, Gesprek openen bovenaan](../images/issues-sidebar.png)

### Acties op een rij

- `Enter` of een dubbelklik opent de rij: een pull request in de [review](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), een issue, en een verzoek op Azure DevOps of Bitbucket, als [gesprek](conversations.md)
  in het venster, en al het andere in je browser. **Openen in browser** in het rijmenu opent altijd de browser.
- Rechtsklik, of `.`, opent het rijmenu:
  - **Gesprek openen** (issues, en verzoeken die geen reviewweergave hebben), **Openen in browser**, **URL kopiëren**,
    **Nummer kopiëren**.
  - Voor een pull request: **Bewerken op <provider>**, **Bronbranch uitchecken**,
    **Commits van bronbranch tonen**. Voor een verzoek uit een fork heet uitchecken
    **Pull request uitchecken (fork)**, en **Tonen** is grijs.
  - Voor een issue: **Branch maken van issue**. Het opent de prompt voor een nieuwe branch
    met een naam als `12-fix-the-thing` (of `ABC-123-fix-the-thing` voor
    Jira).
  - Op GitHub, GitLab, Gitea en Forgejo heeft een rij met pull request ook **Reviewen...**
    (ook **Enter** op de rij): de review in de app, zie
    [Een pull of merge request reviewen](review.md).
  - Op GitHub, GitLab, Gitea en Forgejo hebben beide soorten rijen ook **Toegewezenen bewerken...**,
    **Labels bewerken...** en **Mijlpaal instellen...** (en **Reviewers bewerken...** bij
    een pull request): zie
    [Toegewezenen, reviewers, labels en mijlpaal](#toegewezenen-reviewers-labels-en-mijlpaal).
- **Bewerken op <provider>** heeft drie andere ingangen: het potlood `✎` aan het
  rechteruiteinde van een rij met pull request waar je overheen beweegt of die is geselecteerd, `e` op de geselecteerde rij, en
  de opdracht *Integraties: Gekoppelde pull request bewerken op de host*
  (voor het verzoek van de uitgecheckte branch).
- Als de bronbranch nog niet in de repository zit, zegt GitPoulpi dat. Fetch
  eerst (`Ctrl+G f`).

**Een pull request uitchecken.** GitPoulpi fetcht het verzoek in een lokale
branch `pr/<n>` (`mr/<n>` op GitLab) die het volgt, en schakelt ernaar over. Op
GitHub, GitLab, Gitea en Forgejo werkt dit ook voor forks (GitPoulpi fetcht de speciale head-
ref van de host). Azure DevOps en Bitbucket fetchen de bronbranch van het
verzoek. Een verzoek uit een fork wordt daar geweigerd: voeg de fork als remote toe en
check zijn branch zelf uit. Ongedaan maken keert terug naar de vorige branch. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Een pull of merge request reviewen vanaf de opdrachtregel (en door een agent).** Op GitHub
(ook Enterprise), GitLab (ook self-managed), Gitea en Forgejo kan een verzoek worden
gelezen en gereviewd zonder GitPoulpi te verlaten: `hosting review show <n>` (bestanden,
samenvoegstatus, checks), `hosting review threads <n>` (reviewdiscussies en
gesprek), `hosting review comment`, `hosting review reply`, `hosting review resolve`
en `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. Het verschil komt uit je repository, niet van de
host: `hosting review fetch <n>` haalt de head van het verzoek op naar
`refs/gitpoulpi/pr/<n>` en zijn doelbranch naar `refs/gitpoulpi/pr/<n>-base`
(geen van beide is een branch, er wordt niets uitgecheckt). Regelnummers zijn die van dat
verschil: `new` is de regel in de head, `old` de regel in de merge base. GitLab
kent geen reviewstatus "wijzigingen aanvragen": die wordt daar geweigerd, zonder dat er iets wordt verstuurd.
Gitea en Forgejo nemen een opmerking op slechts één regel (geen bereiken); een discussie zijn de
opmerkingen bij één regel van een bestand. Gitea kan een discussie oplossen, Forgejo 11 niet
(de host heeft er geen endpoint voor: de aanroep zegt het). Een reactie op een server
zonder reactie-endpoint wordt geplaatst als een review met één opmerking op dezelfde regel.
Gitea weigert je eigen verzoek goed te keuren of af te wijzen.
Zonder kloon: voeg `--project owner/name` (GitLab: `group/project`) toe aan `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` en `suggest` om een project te reviewen dat niet de huidige repository is,
`hosting review diff <n> --project P` leest het verschil (hunks per bestand) uit de API van de host, en
`hosting file <path> --project P [--ref R]` leest een bestand op een revisie (GitHub, GitLab, Gitea, Forgejo en Gerrit; bij Gerrit wordt het verschil van een
patchset bestand voor bestand gelezen en komt het bestand van de branch of, voor een commit-id, van de commit). Een bestand dat de host
niet in het verschil toont (te groot) is gemarkeerd als `omitted`.
De MCP-tools zijn dezelfde opdrachten (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Toegewezenen, reviewers, labels en mijlpaal

Op GitHub (ook Enterprise), GitLab (ook self-managed), Gitea en
Forgejo kun je wijzigen aan wie een pull request of issue is toegewezen, wie het reviewt, zijn labels
en zijn mijlpaal zonder GitPoulpi te verlaten. Andere providers tonen niets daarvan.

- **Rijmenu**: **Toegewezenen bewerken...**, **Reviewers bewerken...** (alleen pull requests),
  **Labels bewerken...**, **Mijlpaal instellen...**.
- **Palet**: *Hosting: Toegewezenen bewerken...*, *Hosting: Reviewers bewerken...*,
  *Hosting: Labels bewerken...*, *Hosting: Mijlpaal instellen...* tonen elk eerst de
  verzoeken en issues van de zijbalk om uit te kiezen. In het
  [Radar](radar.md) werken dezelfde vier op de geselecteerde rij.
- De kiezer opent meteen en laadt de mensen of labels van het project (en wat
  het item nu heeft) van de host. Na ongeveer 150 ms verschijnt een spinnerrij.
  Typ om te filteren (een fuzzy match), `Up` / `Down` bewegen, `Space` vinkt aan of
  uit, `Enter` past toe, `Esc` annuleert, `Tab` bereikt de knoppen. Een klik
  op een rij vinkt hem aan, en het wiel scrolt. Mensen tonen een initialenbadge,
  labels hun kleur.
- De mijlpaal is een keuzelijst (met **Geen mijlpaal** om hem te wissen).
- Na **Toepassen** is de host bijgewerkt, en tonen de lijsten en de Radar de
  wijziging na hun volgende vernieuwing, die meteen start.

De formulieren voor maken (een pull request, een issue) hebben dezelfde keuzes als een rij
knoppen: `Toegewezen: geen`, `Reviewers: geen`, `Labels: geen`,
`Mijlpaal: geen`. Op GitHub, GitLab, Gitea en Forgejo vervangt de knop Labels het getypte
labelveld van het issueformulier. Als een host de nieuwe pull request aanneemt maar
sommige van zijn labels of mensen weigert, zegt de statusbalk dat.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, en `--assignee`, `--reviewer`, `--label`,
`--milestone` bij `hosting pr` en `hosting issue create`; zie de
[CLI-naslag](cli-reference.md).

### Toetsen in de secties

| Toets | Actie |
|---|---|
| `Enter` / dubbelklik | open het gesprek in GitPoulpi waar de host er een heeft (een pull request op GitHub, GitLab, Gitea of Gerrit opent de review), anders de browser |
| `o` | open de rij in de browser |
| `.` / rechtsklik | rijmenu |
| `e` | bewerk de geselecteerde pull request op de host |
| `n` | nieuwe pull request (PULL REQUESTS) of nieuw issue (ISSUES) |
| `v` | open de statuskeuzelijst |
| `r` | vernieuwen |
| `Enter` / `Space` op een kop | klap de sectie in of uit |

### Opnieuw verbinden en offline

Als het token ontbreekt, is geweigerd of een scope mist, leest de eerste rij van de
sectie `Opnieuw verbinden (token geweigerd)`. Klik erop, of druk op `Enter`, om
het profiel te openen in Instellingen > Integraties.

Als de host niet bereikbaar is, blijven de laatste lijsten op het scherm staan en toont de kop
hun leeftijd (`· 5m oud`). Het laatste antwoord wordt ook op schijf gecachet
(`.gitpoulpi/hosting-cache.json`), zodat de lijsten verschijnen voordat het netwerk
antwoordt.

## Het issuebord

De knop **Bord** in de kop van ISSUES (palet: **Issues: Bord openen**) toont de issues van de gekoppelde bron als kanban-
bord over het venster. De kolommen hangen van de bron af:

| Bron | Kolommen | Een kaart verplaatsen |
|---|---|---|
| Lokale Markdown-notities | de statussen (`statuses` van het profiel, daarna elke andere die in gebruik is) | herschrijft de `status` van de notitie |
| GitHub, GitLab, Gitea, Forgejo | **Open**, dan de labels die je koos, dan **Gesloten** | verwisselt het statuslabel; naar **Gesloten** sluit het het issue, naar een kolom opnieuw heropent het |
| Trello | de open lijsten van het board, van links naar rechts | verplaatst de kaart naar de lijst |
| Redmine | de issuestatussen | wijzigt de status van het issue (de workflowregels van Redmine gelden) |
| Jira (Cloud en Data Center) | de statussen van het project, to-do eerst, klaar laatst | voert de workflowovergang uit die naar de kolom leidt (`GET`, dan `POST .../transitions`); zonder zo'n overgang gaat de kaart terug en zegt de statusbalk `no transition from X to Y` |
| Azure DevOps (Services en Server) | de staten van de work-itemtypen van het project, per statuscategorie | stelt `System.State` in; een work-item waarvan het type de staat van de kolom mist gaat terug, met de reden (`a Bug has no state Done`) |

Op een GitHub- of GitLab-project vraagt **Kolommen** (of `c`, of **Bord: Kolomlabels instellen...**) om de labels die de
kolommen zijn, op volgorde, bijvoorbeeld `status::todo, status::doing, status::review`. Het stelt de labels van het project voor die
op statussen lijken. De keuze wordt voor de repository bewaard. Zonder labels is het bord Open / Gesloten. Een kaart verplaatsen
raakt alleen de statuslabels die je noemde: zijn andere labels blijven. Een hostbord toont de open issues en de 60 meest
recent gesloten.

Een Jira-overgang die om verplichte velden vraagt (bijvoorbeeld een resolutie) wordt door Jira geweigerd en de kaart gaat
terug met de eigen melding van Jira: doe die op de site.

Hetzelfde bord is beschikbaar voor scripts en agents: `gitpoulpi hosting board` drukt de kolommen met hun kaarten af en
`gitpoulpi hosting board move <card> <column>` verplaatst er een (zie [de CLI-referentie](cli-reference.md)).

Een kaart toont het nummer, de titel, de toegewezenen als initialen, de labels en de mijlpaal. Sleep hem naar een andere kolom,
of `Shift+Left` / `Shift+Right`, of **Bord: Kaart verplaatsen naar...** (`m`). Hij verplaatst direct; na ongeveer 150 ms zegt een spinner
dat de host nog bezig is, en als de host weigert, gaat de kaart terug en zegt de statusbalk waarom. `Enter` opent
het [gesprek](conversations.md) van het issue in GitPoulpi, en het bord komt terug als je het sluit; `e` bewerkt toegewezenen, labels en mijlpaal; `n` start een nieuw issue in
de geselecteerde kolom; `a`, `l`, `M` en `/` filteren op toegewezene, label, mijlpaal en tekst, `x` wist de filters; `r`
leest de host opnieuw. Het bord leest opnieuw als de lijsten van de zijbalk vernieuwen. De Radar heeft geen bord.

## Een pull request maken

Open het formulier met de `+` in de kop PULL REQUESTS, `n` in de sectie, of
de opdracht *Pull requests: Pull request maken...*. Een branchmenu
heeft ook **Een pull request starten vanaf <branch>...**.

Het formulier heeft:

- Eén tabblad per verbonden profiel dat pull requests heeft (`Left` / `Right` op
  de tabbalk).
- Keuzevakken **Van repo / Branch** en **Naar repo / Branch**. Lange lijsten zijn
  doorzoekbaar. Je kunt een branch of repository typen die de lijsten niet tonen.
- **Titel**, ingevuld met het onderwerp van de tip-commit.
- **Beschrijving**, op meerdere regels (`Enter` voegt een nieuwe regel toe, `Tab` verlaat het
  veld).
- Knoppen **Toegewezen**, **Reviewers**, **Labels** en **Mijlpaal** (GitHub,
  GitLab, Gitea en Forgejo), die de kiezer openen.
- **Als concept indienen**.
- Met [AI](ai.md) aan: **Titel en beschrijving genereren**.

Het formulier opent op het profiel, de repository en de branches van de uitgecheckte
branch (`feature -> main`). Een andere bronrepository maakt een verzoek
vanuit een fork.

**Pull Request maken** (of `Ctrl+S`) maakt het verzoek op de host. Het
resultaat opent in je browser en de statusbalk noemt het.

## Een issue maken

Als de sectie ISSUES een provider **en** een project heeft, heeft de kop een
`+` (*Nieuw issue*). Ook: `n` in de sectie, of de opdracht
*Hosting: Nieuw issue...*.

Het formulier heeft **Titel** (verplicht), **Beschrijving** (meerdere regels) en
**Labels** (door komma's gescheiden, bijvoorbeeld `bug, ui`; op GitHub, GitLab, Gitea en Forgejo openen de
knoppen Labels, Toegewezen en Mijlpaal in plaats daarvan de kiezer). Jira en Azure DevOps
voegen een keuzelijst **Type** toe. Trello voegt een keuzelijst **Lijst** toe. Linear en YouTrack
voegen een keuzelijst **Status** toe (de workflowstatus, of de waarde van `State`, waarin het issue
begint). Redmine voegt een keuzelijst **Type** toe met de trackers van het project (Bug, Feature,
Support...); het heeft geen labels, dus er is geen knop Labels voor en een label
op de opdrachtregel wordt geweigerd. Het formulier vult deze
keuzelijsten vanaf de host als het opent.

**Issue maken** (of `Ctrl+S`) maakt het op de host. `Esc` of **Annuleren**
sluit het formulier. Als de host weigert (token geweigerd, ontbrekende scope, onbekend
project, onbekend label), toont het formulier de reden en behoudt het wat je typte.
Bij succes noemt de statusbalk het nieuwe issue en wordt de sectie ISSUES
vernieuwd.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Maak aan op de eigen webpagina van de host

Het formulier Nieuw issue en het formulier Pull Request maken hebben een
knop **Maak aan op <provider>**, rechtsboven. De toets is `Alt+W`. Het opent het
webformulier van de host in je browser, ingevuld met wat je tot nu toe hebt getypt. Het
dialoogvenster blijft open. Het heeft geen token nodig en doet geen API-aanroep, dus het werkt ook
na een API-fout.

- GitHub heeft geen URL-parameter voor *concept*, dus het conceptvakje wordt niet
  meegenomen. GitLab krijgt een titelvoorvoegsel `Draft: `.
- GitLab neemt labels alleen op id aan, dus ze worden toegevoegd als een snelle actie `/label ~"x"`
  aan het eind van de beschrijving.
- Een zeer lange beschrijving wordt afgekapt zodat de URL onder 6000 tekens blijft,
  met een opmerking `[...truncated: paste the rest]`.
- Bitbucket Data Center heeft geen adres voor een webformulier, dus het heeft geen
  knop **Maak aan op <provider>**.

CLI: `gitpoulpi hosting issue create ... --web` en `gitpoulpi hosting pr --web`
drukken dezelfde URL af.

## De chip in de werkbalk

Aan het uiteinde van de werkbalk (standaard rechts) toont een chip het verzoek van de uitgecheckte
branch: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) of
`BB #12` (Bitbucket). Zijn kleur toont de status: groen open, grijs concept,
paars merged, rood gesloten.

- Beweeg erover voor de titel, de status en `source → target`.
- Klik erop, of druk op `Ctrl+G g`, om het verzoek in de browser te openen.
- Rechtsklik erop voor **Openen in browser**, **URL kopiëren**, **Vernieuwen** en
  **Profiel wisselen**.

Een gepushte branch zonder verzoek toont een gedimde chip `+ PR`. Klik erop om een
pull request te starten. Er is geen chip als geen profiel overeenkomt, als de branch niet
is gepusht, of als de host niet bereikt kon worden. In een smal venster is de chip
verborgen om ruimte te houden voor de werkbalkknoppen.

## Wanneer de lijsten vernieuwen

De lijsten vernieuwen:

- als je een repository opent;
- als de uitgecheckte branch verandert;
- na een push of een fetch;
- elke *Vernieuwing pull requests & issues (minuten)*
  (`general.hosting_refresh_minutes`, standaard 5, `0` zet de timer uit;
  Instellingen > Integraties);
- als je Instellingen sluit;
- als je `r` indrukt, *Integraties: Pull requests en issues vernieuwen* uitvoert, of
  een filter wijzigt.

Verzoeken blokkeren het venster nooit.

Hosts beperken hoeveel aanroepen je kunt doen. GitPoulpi stuurt de `ETag` van het
laatste antwoord terug, dus een ongewijzigde lijst kost weinig (niets op GitHub). Als een
host meldt dat er weinig aanroepen over zijn, zegt de statusbalk dat. Als er geen
over zijn, stopt GitPoulpi met die host te vragen tot de limiet reset, en blijven de lijsten
zoals ze zijn.

## Terminologie

GitHub, Azure DevOps en Bitbucket zeggen "pull request". GitLab zegt "merge
request". *Terminologie voor reviewverzoeken* (`ui.pr_terminology`, op
Instellingen > Integraties) kiest het woord voor elk label, menu, palet-
opdracht en elke melding:

- **Auto (per host)**: volgt de gekoppelde provider van de repository ("Pull
  Request" zolang niets is gekoppeld).
- **Pull request** of **Merge request**: altijd dat woord.

De wijziging geldt meteen. De CLI en MCP zeggen altijd "pull/merge request".
Deze pagina zegt "pull request".

## Verwijzingen in commit-berichten en branchnamen

Als de issues of pull requests van de repository zijn gekoppeld, zijn verwijzingen in
het commit-bericht van de Commit-details onderstreept. Een klik opent de gekoppelde
pagina. Toetsenbord: de opdracht
*Integraties: Commit-verwijzing openen (#issue / !verzoek)...* (een menu als het
bericht meerdere verwijzingen heeft). Een repository zonder koppeling heeft geen
links.

Wat als verwijzing telt, hangt af van de provider van de sectie ISSUES:

| ISSUES gekoppeld aan | In een bericht | In een branchnaam |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` en `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, alleen als `ABC` de sleutel van het gekoppelde project is. Tekst als `UTF-8`, `SHA-256` of `ISO-8601` is nooit een link. | `ABC-123` overal in de naam (`feature/ABC-123-x`), in hoofdletters |
| Linear | `ENG-123`, alleen als `ENG` de sleutel van het gekoppelde team is (of van een team in de teamlijst van het profiel). `UTF-8` is nooit een link. | De eigen `branchName` van Linear als je de branch van een issue maakt; `ENG-123` overal in de naam wordt teruggelezen |
| YouTrack | `ABC-123`, alleen als `ABC` de korte naam van het gekoppelde project is (of van een project in de projectlijst van het profiel) | `ABC-123` overal in de naam |
| Redmine | `#123`, alleen als Redmine de gekoppelde ISSUES-bron is (een pull-requestbron houdt zijn eigen teken) | `123-fix-login` |
| Trello | een adres `https://trello.com/c/<shortLink>`; `#12` alleen als een gecachete kaart nummer 12 heeft | `12-fix-login` |

| PULL REQUESTS gekoppeld aan | In een bericht |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Als beide lijsten `#N` herkennen, wint het issue. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**In gesprekken.** De tekst van een issue of verzoek wordt gelezen met de grammatica van zijn host, die breder is dan de tabel
hierboven: `owner/repo#7`, `!7`, `GH-7`, `@login`, commit-id's en de andere vormen die staan in
[Kruisverwijzingen](conversations.md#kruisverwijzingen). De gekoppelde bronnen van de repository worden eerst gematcht, dus een Jira-sleutel in een
GitHub-opmerking opent nog steeds zijn issue in de app. `gitpoulpi hosting markdown --body TEXT --id I` toont wat de app in een
tekst vindt.

### Een issueverwijzing invoegen

In de commit-composer opent de knop `#▾` aan het rechteruiteinde van de Samenvatting
een filterbare lijst van de gekoppelde issues. Het issue dat de huidige branch noemt
staat eerst, gemarkeerd met *(van branch)*. `Enter` of een klik voegt de verwijzing
(`ABC-123`, `#12`, `AB#12`) in bij de cursor. Toetsenbord: `Alt+R` in de composer.
Palet: *Commit: Issue-verwijzing invoegen...*. Zonder gekoppelde issuelijst is de
knop grijs en zegt hij *koppel eerst een issuelijst in de zijbalk*.
GitPoulpi voegt nooit zelf een verwijzing toe.


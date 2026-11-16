[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · **Nederlands** · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Integraties met hosting en issuetrackers

GitPoulpi verbindt met deze producten:

- **Hosting** (pull requests, issues, klonen vanuit je account, SSH-sleutels):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea en Forgejo
  (inclusief Codeberg). **Gerrit Code Review** is hosting alleen voor review: zijn wijzigingen zijn de
  pull requests, het heeft geen issues.
- **Issuetrackers** (alleen issues): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud en Server) en Redmine.
- **Lokale Markdown-notities** (alleen issues, geen token, geen netwerk): een map met Markdown-notities, bijvoorbeeld een map in een Obsidian-vault, zie [Lokale issues en Obsidian](local-issues.md).

Elke integratie gebruikt een **persoonlijk token**. Er is geen GitPoulpi-account en
geen OAuth-aanmelding. Integraties zijn optioneel en **opt-in per repository**:
er wordt niets opgehaald tot je een repository koppelt. Je token wordt opgeslagen in je
credential helper, nooit in `config.toml`. Gewone `git`-bewerkingen (fetch, pull,
push) hebben geen integratie nodig: ze gebruiken de inloggegevens die je `git`
al heeft.

## 1. Maak een token op de host

Het verbindingsformulier heeft een link *Een token genereren op <host>*. Die opent de tokenpagina
van de host met de juiste scopes al geselecteerd. Om het met de hand te doen:

| Host | Waar | Scopes |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repositories, pull requests, issues), `read:user` (wie je bent), `admin:public_key` (alleen om SSH-sleutels toe te voegen of te verwijderen) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, op `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Maak het token **voor die organisatie**, niet voor "All accessible organizations": globale tokens werken niet meer vanaf 2026-12-01. |
| Azure DevOps Server | Dezelfde pagina op je server (`https://<server>/<collection>/_usersSettings/tokens`) | Dezelfde twee scopes |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Voor SSH-sleutels ook `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profielfoto > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, op `<host>/user/settings/applications` | Lees `user`; lees en schrijf `repository`, `issue` en `user` (de laatste voor SSH-sleutels). Gitea toont achteraf geen scopes, dus GitPoulpi kan niet waarschuwen voor een ontbrekende: een geweigerde aanroep zegt het. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. De sleutel handelt als jij. |
| YouTrack (Cloud en Server) | Je profiel > Account Security > New token, op `<host>/users/me?tab=account-security` | De scope `YouTrack`. Een permanent token handelt als jij. |
| Gerrit | Settings > HTTP Credentials > Generate new token (recente servers; oudere: Generate new password), op `<host>/settings/#HTTPCredentials` | Geen. Het token is het **HTTP-wachtwoord** en handelt als jij; de gebruikersnaam staat ernaast. |
| Redmine | My account > API access key > Show, op `<host>/my/account`. De beheerder moet **Administration > Settings > API > Enable REST web service** hebben aangezet. | Geen. De sleutel handelt als jij. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (het klassieke soort, zonder scopes) | Geen. Het token handelt als jij. |
| Jira Data Center | Profile > Personal Access Tokens | Geen. Het token handelt als jij. |
| Trello | Eerst een **API-sleutel** van `trello.com/apps/admin` (maak een Power-Up en genereer dan een API-sleutel). Dan een **token** van `trello.com/1/authorize?...&key=<your key>`. | `read`, `write` |

Geef het token een naam als "GitPoulpi" en een vervaldatum. Kopieer het token:
de host toont het maar één keer.

## 2. Verbind het

1. Druk op `Ctrl+,` en kies **Integraties**.
2. Kies links het product. Producten staan in twee groepen: **Hosting** en
   **Issuetrackers**.
3. Druk op **Verbinden**. Om een tweede account van hetzelfde product toe te voegen, gebruik je
   **+ Profiel toevoegen**.

   ![De pagina Integraties: de producten links en een knop Verbinden voor GitHub](../images/settings-integrations.png)

4. Vul het formulier in en plak het token. Het token wordt tijdens het typen in leesbare tekst getoond,
   dus zorg dat niemand meekijkt. Het formulier toont de scopes
   die het nodig heeft.

   ![Het verbindingsformulier: een veld Persoonlijk toegangstoken, de scopes repo, read:user en admin:public_key, een link om een token op github.com te genereren, en de knoppen Verbinden en Annuleren](../images/settings-connect.png)

5. Druk op **Verbinden**. GitPoulpi controleert het token op de achtergrond bij de host.
   De pagina toont dan een kaart met je naam en login,
   *Verbonden* in het groen, en het SSH-sleutelgedeelte. Een token zonder een van de
   gewenste scopes verbindt toch, en de ontbrekende scopes staan in het rood.

Op de kaart: **Bewerken** opent het formulier opnieuw. **Verbinding verbreken** verwijdert het token
en behoudt het profiel. **Verwijderen** verwijdert beide, na een bevestiging.

Toetsenbord op de pagina Integraties: `Tab` / `Shift+Tab` gaan tussen de
productlijst, de profielstrook en het formulier. `a` voegt een profiel toe, `e` bewerkt
het, `d` of `Delete` verwijdert het, `o` opent de tokenpagina, `Esc` gaat één
stap terug.

### Wat elk formulier vraagt

De link *Een token genereren* werkt pas nadat je de host of de
organisatie hebt ingevuld.

- **GitHub, GitLab:** alleen het token. **GitHub Enterprise** en
  **GitLab Self-Managed** vragen ook om het **Hostdomein**
  (`gitlab.example.com`; https tenzij je `http://` typt). Een naam zonder punt
  (een Docker-service of een machine op het lokale netwerk) heeft een poort of `http://` nodig:
  `gitea:3000`, `http://nas`. Een zelf gehoste server kan gewoon http
  gebruiken, bijvoorbeeld een lokale container op `http://localhost:8080`. Een token dat over gewoon
  http wordt verstuurd, gaat onversleuteld over het netwerk; gebruik http dus alleen
  op een lokaal of vertrouwd netwerk.
- **Azure DevOps:** de **Organisatie**, de naam in
  `dev.azure.com/<organization>`. Een adres `https://<org>.visualstudio.com` wordt
  ook geaccepteerd. Twee organisaties hebben twee profielen nodig. Een repository gebruikt het
  profiel van de organisatie die zijn remote noemt. **Azure DevOps Server**
  vraagt om de **Collectie-URL**, bijvoorbeeld
  `tfs.corp/tfs/DefaultCollection`. GitPoulpi gebruikt de credential helper
  van het systeem niet voor Azure DevOps: Git Credential Manager slaat daar een aanmeldtoken
  op, geen persoonlijk toegangstoken.
- **Bitbucket:** je **e-mailadres van het Atlassian-account** en een API-token. Als het
  e-mailadres leeg is, wordt het token als bearer-token verstuurd: gebruik dit voor een workspace-
  of repositorytoegangstoken. GitPoulpi vindt je workspaces via het token.
  **Bitbucket Data Center:** de **Server-URL** en een HTTP-toegangstoken.
- **Gitea, Forgejo:** het **Hostdomein** en het token. **Forgejo** begint op
  `https://codeberg.org`: vervang het voor een zelf gehoste server. Gitea heeft geen
  openbare standaard. Een adres `http://localhost:3000` werkt voor een lokale server.
  Beide soorten delen één client (Forgejo is een Gitea-fork met dezelfde
  `/api/v1`); het token wordt verstuurd als `Authorization: token <token>`. De
  credential helper van het systeem wordt er niet voor gebruikt.
- **Jira Cloud:** de **Site-URL** (`acme.atlassian.net`), je **e-mailadres van het Atlassian-
  account** en een klassiek API-token. Jira-tokens met scopes worden niet
  ondersteund. **Jira Data Center:** de **Server-URL** en een persoonlijk toegangs-
  token.
- **Trello:** de **API-sleutel** en het **token**. De API-sleutel is openbaar, dus hij
  wordt in `config.toml` bewaard (als `account`). Het token is geheim en gaat naar de
  credential helper. Het formulier heeft twee links: *API-sleutel ophalen bij Trello* en,
  nadat je de sleutel hebt getypt, *Token genereren*. De sleutel en het token worden in een
  header verstuurd, nooit in een URL.

- **Linear:** alleen de **API-sleutel**: Linear heeft één openbare host, dus er is
  geen adres te typen. De sleutel wordt zoals hij is in de header `Authorization` verstuurd
  (Linear neemt geen `Bearer` voor persoonlijke sleutels). Om de `ENG-12`-verwijzingen
  in commit-berichten de juiste pagina te laten openen, bewaar je de profiel-URL met je
  workspace (`https://linear.app/acme`, vanuit de CLI: `--url`); de issuerijen
  zelf dragen altijd het exacte adres.
- **YouTrack:** de **Site-URL** (`acme.youtrack.cloud` voor Cloud, anders je eigen
  server, met zijn contextpad als hij er een heeft, bijvoorbeeld
  `host/youtrack`) en een permanent token, verstuurd als `Authorization: Bearer`. Dezelfde
  client bedient Cloud en Server (`<url>/api`).
- **Gerrit:** de **Server-URL** (met zijn contextpad als hij er een heeft), je **Gebruikersnaam** en het HTTP-
  wachtwoord als token. Ze reizen als HTTP basic authentication; GitPoulpi praat met de `/a/` REST-API van de
  server en negeert de regel `)]}'` waarmee elk antwoord begint. De gebruikersnaam is niet geheim en wordt in
  `config.toml` bewaard (als `account`), het wachtwoord gaat naar de credential helper. Een repository wordt aan een Gerrit-
  *project* gekoppeld onder PULL REQUESTS (zijn wijzigingen staan daar in de lijst); Gerrit heeft geen issues, en een wijziging wordt gemaakt door
  te pushen (zie *Pushen voor review* in [Reviewen](review.md#gerrit)).
- **Redmine:** de **Site-URL** van je Redmine (met zijn subpad als hij er een heeft,
  bijvoorbeeld `host/redmine`) en je API-sleutel, verstuurd als `X-Redmine-API-Key`. Als
  Redmine met 403 of een HTML-pagina antwoordt, staat de REST-webservice uit (vraag het aan de
  beheerder, of zet hem aan in **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack en Redmine zijn alleen issuetrackers: geen pull requests, geen repositories om te
klonen en geen SSH-sleutels. Bitbucket heeft geen issues (Atlassian heeft Bitbucket
Issues stopgezet). Koppel in plaats daarvan de sectie ISSUES van een Bitbucket-repository aan een Jira-project.

## 3. Waar het token wordt opgeslagen

Het token wordt opgeslagen in de **credential helper** van je systeem (Git Credential
Manager op Windows, libsecret of `pass` op Linux, Keychain op macOS). Het gebruikt een
privé-item dat niet kan worden verward met de inloggegevens van een echte host. Het
wordt nooit naar `config.toml` geschreven en nooit afgedrukt.

Als er geen credential helper is geconfigureerd, of als hij het token weigert, houdt GitPoulpi
het token **alleen voor deze sessie in het geheugen** en laat het je weten. Verbind
opnieuw na een herstart.

GitPoulpi zoekt het token van een profiel in deze volgorde:

1. De omgevingsvariabele `GITPOULPI_TOKEN_<PROFILE ID>`. Gebruik die om een
   token te geven zonder het op te slaan, bijvoorbeeld in CI.
2. Het voor het profiel opgeslagen token (`token_ref`).
3. Alleen voor GitHub- en GitLab-profielen: de credential helper van het systeem voor de
   host, als geen ander ingeschakeld profiel dezelfde host gebruikt.

Elk profiel is een `[[integrations]]`-item in `config.toml`. Zie de
[configuratienaslag](config-reference.md#integrations-hosting-en-issuetrackers).

## De beveiligingscontrole

Een opgeslagen token is maar zo veilig als het adres waarheen het wordt gestuurd. GitPoulpi onthoudt daarom per integratie waarmee het was verbonden en stopt als dat verandert:

- **Het adres.** De eerste keer dat een integratie wordt gebruikt (of verbonden), wordt haar adres vastgelegd in `trust.toml`, in de privégegevensmap die alleen jij kunt lezen (`<data>/private/`), niet in `config.toml`: het bewerken van de configuratie kan nooit verbreden wat wordt vertrouwd. Als het adres in de configuratie niet meer het vastgelegde is, wordt er niets naartoe gestuurd en wordt het token niet eens gelezen. Spellingen van één adres (`HTTPS://Git.Example.com:443/`) zijn hetzelfde adres; een host die erop lijkt, niet.
- **Het account.** Nadat je een token hebt verbonden, en daarna ongeveer eenmaal per dag (en eenmaal per run voor een token dat uit `GITPOULPI_TOKEN_*` of de credential helper komt), vraagt GitPoulpi aan de host van wie het token is en vergelijkt de account-**id**, niet de login. Een token dat nu van iemand anders is, wordt gestopt.
- **Het certificaat.** Servers waarvan het certificaat niet naar een bekende autoriteit leidt (een privé-CA, een zelfondertekende server) worden alleen geaccepteerd voor de exacte sleutel die je hebt geaccepteerd, en de TLS-handshake bewijst nog steeds dat de server die sleutel heeft. Met strikt pinnen aan moet de sleutel overeenkomen, ook als het certificaat naar een openbare root leidt; de prijs is dat elke sleutelrotatie opnieuw vraagt, daarom is het opt-in. De openbare hosts (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) houden geen certificaatregistratie bij.
- **Gewone http.** Een `http://`-adres dat geen loopback is, wordt geweigerd tenzij je het voor die integratie hebt toegestaan.

Elk verzoek wordt ook gebonden aan het eigen adres van de integratie (en, voor een openbare host, aan zijn API-host zoals `api.github.com`): een verzoek ergens anders heen wordt geweigerd voordat het wordt verstuurd, en een redirect naar een ander adres wordt niet gevolgd.

**Integraties die zijn verbonden voordat deze controle bestond** krijgen hun registratie stilletjes, uit de configuratie zoals die was, de eerste keer dat ze na de upgrade worden gebruikt: ze behouden het huidige vertrouwensniveau en er wordt niets gevraagd.

### Vanaf de opdrachtregel

Een gestopte opdracht mislukt met afsluitcode 2 en drukt af wat er is veranderd (zie de [CLI-naslag](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` toont wat wordt vertrouwd; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` en `--allow-insecure-http on|off` accepteren elk één ding, met de exacte waarde. Voer na het accepteren van een nieuw adres het token opnieuw in. Verbinding verbreken (`hosting logout`) of een integratie verwijderen verwijdert haar token, haar vertrouwensregistratie en wat ervan in de cache stond (de opgeslagen repositorylijsten van het dialoogvenster Klonen, de accountlijsten van de Radar en de externe reviewconcepten van die host).

Een agent (MCP) kan niets accepteren: hem wordt gezegd dat de beveiligingscontrole jou nodig heeft.

### In de terminal-UI

Een gestopt verzoek opent het dialoogvenster **Beveiligingscontrole van integratie** boven waar je mee bezig was. Het zegt wat er is veranderd (het oude en het nieuwe adres, het oude en het nieuwe account met hun id's, of het oude en het nieuwe certificaat als een kleine tabel met de SHA-256-vingerafdruk in groepen van vier, onderwerp, uitgever en geldigheid) en dat het token niet is verstuurd. De antwoorden zijn genummerd:

1. **Verbinding verbreken** verwijdert het opgeslagen token, de vertrouwensregistratie en wat van de integratie in de cache stond.
2. **Het nieuwe adres vertrouwen**, **Het nieuwe account vertrouwen** of **Het nieuwe certificaat vertrouwen** (een niet-versleuteld adres: **Gewone http toestaan (niet versleuteld)**).
3. **Annuleren** wijzigt niets.

Geen enkel antwoord is gemarkeerd tot je een pijltoets of een cijfer indrukt, dus een verdwaalde Enter accepteert niets. Esc en het sluitkruisje annuleren; een klik buiten het dialoogvenster doet niets. Een lange tekst scrolt met het wiel, Page Up/Page Down of de schuifbalk. Meerdere controles komen in een wachtrij (één per integratie en soort wijziging) en worden na elkaar gevraagd. Een geannuleerde controle verschijnt niet opnieuw zolang hij onveranderd is; **Integraties: Beveiligingscontrole bekijken** in het opdrachtpalet brengt hem terug.

Nadat je een nieuw **adres** hebt vertrouwd, opent het formulier van de integratie met het tokenveld gefocust en de hint "Het token opnieuw invoeren wordt aanbevolen": het opgeslagen token is daar nooit naartoe gestuurd. In het formulier wordt een bewust gewijzigd adres geaccepteerd samen met een token dat ervoor is getypt; zonder token vraagt het formulier erom.

Het formulier heeft ook twee schakelaars. **Strikte certificaatpinning** (voor een https-adres) pint de certificaatsleutel die de server aanbiedt wanneer je verbindt. **Gewone http toestaan (niet versleuteld)** wordt alleen aangeboden voor een `http://`-adres dat niet deze computer is, met een rode waarschuwing dat het token als leesbare tekst reist. De verbonden kaart toont de vastgepinde vingerafdruk.

### Wat het token mag doen, en tot wanneer

Verbinden vraagt de host ook wat het token kan doen. GitHub (klassieke tokens), GitLab en Bitbucket (wanneer die de scopes meestuurt) melden hun scopes: de kaart toont een gele regel "Dit token heeft meer rechten dan GitPoulpi nodig heeft" met de scopes die verder gaan dan wat GitPoulpi gebruikt, de gevoelige (zoals `delete_repo`, `admin:org`, `sudo`) in rood, met het advies liever een fijnmazig token met een vervaldatum te gebruiken. Een klassiek GitHub-token (`ghp_...`) krijgt dat advies ook als zijn scopes precies kloppen. Een fijnmazig GitHub-token kan niet naar zijn rechten worden gevraagd, dus alleen zijn vervaldatum wordt getoond. De kaart toont de vervaldatum als de host die meldt. Een token zonder vervaldatum toont geen herinnering, en de andere producten melden geen scopes en geven geen waarschuwing. Dezelfde velden staan in de JSON van `hosting integrations login` en `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Ze worden gelezen wanneer je verbindt, dus de kaart toont ze voor de sessie die verbond.

## 4. Koppel een repository

Koppelen is expliciet en per repository. Als een profiel is verbonden, toont de Refs-
zijbalk na TAGS een sectie **PULL REQUESTS** en een sectie **ISSUES**.
Elke sectie heeft twee keuzelijsten: de provider en het project.

- De providerkeuzelijst **stelt** de accounts **voor** die overeenkomen met de remotes van de
  repository (`origin` eerst), als rijen zoals `owner/name · origin · GitHub`. Eén
  keuze stelt zowel de provider als het project in.
- Er wordt niets automatisch gekoppeld. Jira, Trello, Linear, YouTrack en Redmine worden nooit voorgesteld, omdat
  geen remote ze noemt: kies ze zelf.
- **Geen (ontkoppel deze repository)**, bovenaan de keuzelijst, ontkoppelt de
  sectie. Niets koppelt hem uit zichzelf opnieuw.
- De koppeling wordt opgeslagen in het bestand `.gitpoulpi/repo-state.toml` van de
  repository.

De twee secties worden **afzonderlijk** gekoppeld. Pull requests kunnen bijvoorbeeld
van Azure DevOps komen en issues van Jira.

| Provider | Project voor PULL REQUESTS | Project voor ISSUES |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | hetzelfde |
| Azure DevOps | `Project/Repo` (de organisatie komt uit het profiel) | `Project` (werkitems) |
| Gitea, Forgejo | `owner/name` | hetzelfde |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | geen: koppel Jira |
| Jira | geen | de projectsleutel, bijvoorbeeld `ABC` |
| Trello | geen | een board |
| Linear | geen | een teamsleutel, bijvoorbeeld `ENG` (de teams worden getoond) |
| YouTrack | geen | een korte projectnaam, bijvoorbeeld `ABC` |
| Gerrit | de naam van het Gerrit-project, bijvoorbeeld `team/tool` | geen |
| Redmine | geen | een projectidentifier, bijvoorbeeld `bench` (het laatste deel van het adres van het project) |

De projectlijst voor ISSUES is doorzoekbaar. Je kunt ook een project typen dat de
lijst niet toont.

**Extra query.** Een issuebron heeft een optionele filterrij onder de projectkeuzelijst.
Hij wordt met het project gecombineerd (AND):

- Jira: een **JQL**-fragment (`component = UI AND labels = backend`).
- Azure DevOps: een **WIQL**-voorwaarde (`[System.AssignedTo] = @Me`).
- YouTrack: een **YouTrack-query** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Hij wordt tussen haakjes gezet, dus een `or` kan het project niet verruimen. Een
  `sort by:` van jou vervangt de standaard `sort by: updated desc`.
- Redmine: geen (de filters voor status en "aan mij toegewezen" van de sectie volstaan).
- Linear: geen (de filters voor status en "aan mij toegewezen" van de sectie
  volstaan).
- Trello: een keuzelijst waarin je de **lijsten** aanvinkt die je wilt tonen. Zonder vinkje worden alle
  open lijsten getoond.
- GitHub en GitLab negeren hem.

`Enter` houdt een getypte query, `Esc` laat hem vallen. Palet:
*Issues: Query / lijsten bewerken...*.

Vanuit de CLI: `gitpoulpi hosting link pulls --id <profiel>` (of `issues`),
`--clear` om te ontkoppelen, `--filter` voor de extra query. Zie de
[CLI-naslag](cli-reference.md#opdrachten).

### Klonen vanaf een verbonden account

Een verbonden profiel (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) is ook een bron in het dialoogvenster **Klonen** (de knop **Klonen** op het tabblad Start, of daar `c`). Elk profiel staat onder **Klonen met URL** als een eigen bron (`GitHub · name`, `GitLab · name`); GitHub en GitLab worden nooit door elkaar gehaald. Kies er een en de repositories worden geladen, gegroepeerd per eigenaar: eerst je eigen account, daarna een sectie per organisatie of groep (`quarry/tools` voor een GitLab-subgroep), elk onder een kop met het aantal. Een slotje markeert een privé-repository.

![Het dialoogvenster Klonen met een verbonden GitHub-account: een sectie per eigenaar, een slotje bij een privé-repository](../images/clone-hosted.png)

Het zoekveld filtert over alle secties heen (naam, volledig pad en beschrijving); een sectie zonder treffer verdwijnt. `Enter` op een rij vult de kloon met de HTTPS-URL van die repository en een mapnaam.

![Het dialoogvenster Klonen, waarbij de zoekopdracht de lijst tot een organisatie beperkt](../images/clone-hosted-search.png)

De lijst van een profiel dat je eerder hebt geopend, verschijnt meteen uit een opgeslagen kopie terwijl de host opnieuw wordt gevraagd (*vernieuwen...* naast het label); zodra het antwoord binnen is, verschijnen nieuwe repositories en verdwijnen verdwenen, en blijven de gemarkeerde en gekozen repository waar ze zijn. Mislukt het vernieuwen, dan blijft de opgeslagen lijst staan met een opmerking, en de reden staat in het activiteitenlogboek.

Weigert de host het token, dan toont het dialoogvenster in plaats van de lijst *kon repositories niet weergeven* met de reden (en de opgeslagen kopie wordt verwijderd); de andere bronnen blijven werken.

## 5. Pull requests en issues in de zijbalk

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

### Acties op een rij

- `Enter` of een dubbelklik opent de rij: een pull request in de [review](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), een issue, en een verzoek op Azure DevOps of Bitbucket, als [gesprek](#5c-gesprekken)
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
    [Een pull request reviewen](review.md).
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

**Een pull request reviewen vanaf de opdrachtregel (en door een agent).** Op GitHub
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

## 5b. Het issuebord

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
het [gesprek](#5c-gesprekken) van het issue in GitPoulpi, en het bord komt terug als je het sluit; `e` bewerkt toegewezenen, labels en mijlpaal; `n` start een nieuw issue in
de geselecteerde kolom; `a`, `l`, `M` en `/` filteren op toegewezene, label, mijlpaal en tekst, `x` wist de filters; `r`
leest de host opnieuw. Het bord leest opnieuw als de lijsten van de zijbalk vernieuwen. De Radar heeft geen bord.

## 5c. Gesprekken

Een **gesprek** is alles wat over één issue, werkitem, kaart, wijziging of pull / merge request is gezegd: de
beschrijving, de opmerkingen, de reviews, de discussies bij code en de gebeurtenissen (labels, toewijzingen,
statuswijzigingen, pushes). GitPoulpi leest het van de host, toont het opgemaakt en laat je antwoorden zonder het venster te
verlaten.

### Een gesprek openen

| Vanuit | Hoe |
|---|---|
| Een rij ISSUES | `Enter` of een dubbelklik; **Gesprek openen** in het rijmenu; palet *Hosting: Issuegesprek openen...* (een lijst van de issues op het scherm). **Openen in browser** in het rijmenu houdt de browser. |
| Een rij PULL REQUESTS op Azure DevOps of Bitbucket | `Enter` of **Gesprek openen**. Op GitHub, GitLab, Gitea, Forgejo en Gerrit opent de rij de [review](review.md), waarvan de pagina Overzicht ditzelfde gesprek is (de tijdlijn, de toetsen en het opmerkingenvak eronder). |
| Een kaart van het [issuebord](#5b-het-issuebord) | `Enter` of een dubbelklik. |
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
| *Plaatsen als ci-bot (tokencommando van de repository)* | Het [tokencommando](#13d-optreden-als-github-app-bot) van de repository wordt voor de host-API gebruikt. |
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
[Optreden als GitHub App-bot](#13d-optreden-als-github-app-bot).

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

## 6. Een pull request maken

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

## 7. Een issue maken

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

## 8. Maak aan op de eigen webpagina van de host

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

## 9. De chip in de werkbalk

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

## 10. Wanneer de lijsten vernieuwen

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

## 11. Terminologie

GitHub, Azure DevOps en Bitbucket zeggen "pull request". GitLab zegt "merge
request". *Terminologie voor reviewverzoeken* (`ui.pr_terminology`, op
Instellingen > Integraties) kiest het woord voor elk label, menu, palet-
opdracht en elke melding:

- **Auto (per host)**: volgt de gekoppelde provider van de repository ("Pull
  Request" zolang niets is gekoppeld).
- **Pull request** of **Merge request**: altijd dat woord.

De wijziging geldt meteen. De CLI en MCP zeggen altijd "pull/merge request".
Deze pagina zegt "pull request".

## 12. Verwijzingen in commit-berichten en branchnamen

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
[Kruisverwijzingen](#kruisverwijzingen). De gekoppelde bronnen van de repository worden eerst gematcht, dus een Jira-sleutel in een
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

## 13. SSH-sleutels

### SSH-sleutel per profiel

Een verbonden profiel heeft onder zijn kaart een sectie **SSH-sleutel**. Kies zonder sleutel een van:

- **Standaardsleutel**: je ssh-agent of `~/.ssh`. Er verandert niets.
- **Nieuwe sleutel genereren**: een optionele titel (standaard
  `GitPoulpi <host> <datum>`). Met **De publieke sleutel aan <host> toevoegen** aangevinkt
  uploadt GitPoulpi hem, als de host hem nog niet heeft. De sleutel wordt
  zonder wachtwoordzin geschreven naar `~/.ssh/gitpoulpi_<profile id>_ed25519` en
  `.pub`. Er is geen `ssh-keygen` nodig. Een bestaand bestand wordt nooit overschreven.
- **Eigen sleutel**: typ een pad of **Bladeren** naar een bestand met een privésleutel, en druk dan op
  **Deze sleutel gebruiken**.

Als een sleutel is gekoppeld, toont de sectie `title (SHA256:...)` met:

- **Loskoppelen**: koppel de sleutel los, en verwijder hem op de host als GitPoulpi hem heeft toegevoegd.
- **Verwijderen**: hetzelfde, en verwijdert ook de sleutelbestanden, na een bevestiging
  die beide paden toont.

### SSH-sleutel voor één repository

Om één repository een bepaalde sleutel te laten gebruiken, open je het tabbladmenu (`Ctrl+T`, of
rechtsklik op het tabblad) en kies je **SSH & inloggegevens...**, of voer je de opdracht
*Repository: SSH & inloggegevens...* uit, en kies je onder SSH **Sleutelbestand** (of
**Accountsleutel** voor de sleutel van een integratieprofiel). De keuze wordt in de privémap van
GitPoulpi bewaard: er wordt niets naar de repository of naar de configuratie van git geschreven.
Zie [Instellingen](settings.md#repositoryspecifiek-ssh--inloggegevens).

Een repository die nog een eigen `core.sshCommand` heeft, van de vroegere pin
"SSH-sleutel voor deze repository", toont die op die pagina, alleen-lezen, met
**Naar GitPoulpi verplaatsen**.

CLI en MCP: `gitpoulpi repo-auth` (de keuze per repository) en, voor de pin `core.sshCommand`
zelf, `gitpoulpi ssh-command show | set --key <file> | clear`.

De globale sleutel op Instellingen > SSH & inloggegevens (*Pad van privésleutel*, of
*Sleutelpaar genereren*) geldt voor elke repository zonder eigen keuze.
Het sleutelbestand of de pin van een repository wint ervan.

## 13b. Branchbescherming, voor het herschrijven van de geschiedenis

Als je de geschiedenis herschrijft ([de werkbank](history-rewrite.md)), vraagt GitPoulpi de host
welke branches hij beschermt, zodat ze standaard buiten de herschrijving blijven. Dat gebeurt alleen
voor een repository die je hebt **gekoppeld** (het gedeelte met pull requests of issues in de
zijbalk, of een vastgepind profiel), met het token van dat profiel, en een token dat het niet kan
lezen geeft gewoon "onbekend": er mislukt niets. Wat elke provider antwoordt, en wat het token nodig
heeft:

| Host | Wat wordt gelezen | Het token heeft nodig |
|---|---|---|
| GitHub (en Enterprise Server) | de vlag `protected` van de branches, en de actieve rulesets voor branches (`~DEFAULT_BRANCH`, `~ALL` en patronen `refs/heads/...`) | een klassiek token met `repo` (`public_repo` voor een openbare repository), of een fijnmazig token met Metadata (lezen). Een ruleset die het token niet kan lezen, wordt gemeld; de klassieke regels tellen nog mee |
| GitLab (gitlab.com en zelf beheerd) | `protected_branches` (namen en patronen met `*`) | `read_api`, en een rol die de instellingen van de repository mag zien |
| Azure DevOps (Services en Server) | het ingeschakelde, blokkerende branchbeleid van de repository (bereiken `Exact`, `Prefix` en `DefaultBranch`) | Code (lezen). Het recht "force push" is een beveiligingsinstelling en wordt niet gelezen |
| Bitbucket Cloud | de branchbeperkingen die een push stoppen (`push`, `force`, `delete`) | `repository:admin`. Een beperking op het branchingmodel wordt niet uitgewerkt |
| Bitbucket Data Center | de branchrechten die een force push stoppen (`fast-forward-only`, `pull-request-only`, `read-only`) | beheerder van de repository |

De instellingenpagina van de bescherming (en van de standaardbranch, en van het verwijderen van de
repository) wordt voor GitHub, GitLab, Azure DevOps en Bitbucket opgebouwd uit de remote-URL van de
repository, ook als de repository niet is gekoppeld, en getoond met de drie uitwegen als de host een
push weigert.

## 13c. Accountbrede lijsten in de Radar

De Radar bewaakt normaal de repositories die je hebt gekloond. Een profiel kan ook tonen wat jou aangaat op de
hele host, gekloond of niet, met drie schakelaars in zijn tabel `[integrations.radar]` (standaard allemaal uit; zie de
[configuratienaslag](config-reference.md#integrationsradar-accountbrede-lijsten)):

- **reviewverzoeken**: openstaande verzoeken die om jouw review vragen;
- **openstaande verzoeken**: elk openstaand verzoek van je eigen namespace en van elke org of
  groep waarvan je lid bent, of van de namespaces die je noemt. Nooit de hele host;
  de **Namespaces** van het profiel (één per regel in Instellingen > Integraties > Radar, desgewenst
  als reguliere expressies) beperken alle drie de lijsten: een regel komt overeen met het volledige pad
  van een repository als hij eraan gelijk is of een padvoorvoegsel is bij een `/` (`ptm` houdt
  `ptm/produits/x`, niet `ptmx/y`), en wordt toegepast op de rijen die de host teruggeeft, plus aan
  serverzijde waar de zoekopdracht van de host het toelaat (de kwalificaties `user:` / `repo:` van GitHub);
- **toegewezen issues**: openstaande issues die aan jou zijn toegewezen.

GitHub, GitLab (ook self-managed), Gitea, Forgejo, Azure DevOps (Services
en Server), Bitbucket Data Center en Gerrit antwoorden. Hoe elk wordt gevraagd:

| Host | Verzoeken | Issues |
|---|---|---|
| Azure DevOps | actieve pull requests van de hele organisatie (`searchCriteria.reviewerId` voor degene die op jou wachten; een verzoek waarop je al stemde blijft weg); weigert de server de route op organisatieniveau, dan worden de eerste 20 projecten één voor één gevraagd. `namespaces` zijn projectnamen | werkitems aan jou toegewezen die niet Done, Closed of Removed zijn |
| Bitbucket Data Center | je dashboard: rol `REVIEWER` nog niet goedgekeurd voor reviewverzoeken, `AUTHOR` en `PARTICIPANT` voor openstaande verzoeken; `namespaces` zijn projectsleutels | geen (Data Center heeft geen issuetracker; een melding zegt het) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; openstaande verzoeken zijn `is:open projects:<prefix>` per namespace (een padvoorvoegsel van een project), of je eigen en gereviewde wijzigingen als er geen is genoemd; een work-in-progress-wijziging is een concept | geen |

Bitbucket Cloud heeft geen accountbrede rijen: het biedt geen endpoint dat verzoeken
over repositories heen toont, en elke repository afzonderlijk vragen zou te veel
aanroepen zijn. Zijn gekloonde repositories houden hun eigen lijsten, en `gitpoulpi hosting
radar --id <bitbucket>` zegt dat. Heeft Gerrit meer wijzigingen dan de limiet die het
teruggeeft, dan wordt de lijst met een melding als ingekort gemarkeerd.

Concepten blijven weg tenzij je ze houdt. Elke lijst is begrensd: 300 rijen per soort, met een
melding hoeveel er nog meer waren, en op GitHub een paar zoekaanroepen per
ronde, omdat zijn zoek-API er maar 30 per minuut toestaat. Een namespace die de token
niet kan lezen wordt een melding en de andere laden nog; een geweigerde token of een
snelheidslimiet laat alleen dat profiel mislukken. Antwoorden worden vijf minuten hergebruikt
(`radar-account.json` in de configuratiemap).

In Instellingen > Integraties heeft een verbonden codehostprofiel een blok **Radar**
onder zijn SSH-sleutel: één selectievakje per lijst (*Verzoeken die wachten op mijn review*,
*Elk openstaand verzoek*, *Concepten meenemen*, *Aan mij toegewezen issues*, de laatste
afwezig voor hosts zonder issues), een tekstgebied *Namespaces* (één item per regel: een naam die overeenkomt met die groep of eigenaar en alles
daaronder, of een reguliere expressie per regel als *Reguliere expressies* is aangevinkt; leeg betekent
jouw eigen namespace en elke waartoe je behoort) en de keuze *Verzoek van een
niet-gekloonde repository*: *Vragen*, *Klonen* of *Reviewen via de API*.
Alles wordt opgeslagen terwijl je wijzigt. Hosts zonder accountbrede query (Bitbucket
Cloud) tonen in plaats daarvan één regel die dat zegt. Het opdrachtenpalet bereikt het blok
met *Instellingen: Radar-opties van een profiel...*, en de zoekfunctie van Instellingen vindt elke
bediening op zijn label.

Vanaf de opdrachtregel: `gitpoulpi hosting integrations radar <id> --review-requests on`
zet een lijst aan, `gitpoulpi hosting radar --json` drukt de lijsten af (en
`--refresh` slaat de cache over). AI-agents hebben dezelfde twee opdrachten via MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 13d. Optreden als GitHub App-bot

Een project dat de naam van een persoon niet mag tonen, kan alles op de host doen
via een **GitHub App**: elke opmerking, review, sluiting of heropening en labelwijziging
verschijnt dan als `<app-slug>[bot]`. Dit werkt voor één repository tegelijk en laat je
andere repositories op je eigen token.

1. Maak een GitHub App voor de organisatie met de rechten **Issues** en **Pull
   requests** op *Read & write* (en *Contents: read*, zodat pull requests kunnen worden
   getoond en gereviewd), en installeer hem op de repository.
2. Schrijf een klein script dat een **installatietoken** op zijn standaarduitvoer afdrukt
   (de privésleutel van de App ondertekent een kortlevende JWT, die wordt ingewisseld voor een
   token dat een uur geldig is). GitPoulpi ziet de sleutel nooit.
3. Kies in **Instellingen > Repositoryspecifieke instellingen > SSH & inloggegevens** onder HTTPS
   **Tokencommando**, vul het script in als **Commando** en de host
   (`github.com`) onder **Hosts**, en vink **Gebruiken voor de host-API
   (issues, pull requests)** aan. Sla op en bevestig het commando (een persoon bevestigt het altijd,
   met zijn SHA-256). Vanaf de opdrachtregel:
   `gitpoulpi repo-auth set --https token-command --hosts github.com
   --use-for-hosting --yes --command ./app-token.sh`.

Wat er voor die repository verandert:

- Zijn issue- en pull-requestaanroepen (tonen, maken, opmerkingen plaatsen,
  reviewen, mergen, sluiten, labels) gebruiken het token van het commando, met dezelfde
  limiet van 30 seconden, geheugencache en maskering als de eigen aanmelding van git. Als
  de host met 401 antwoordt omdat het token verlopen is, wordt het commando nog eens
  gevraagd. Kan het commando niet draaien, dan wordt de aanroep geweigerd: je persoonlijke token is
  nooit de terugval.
- Er is geen "ik": een installatietoken kan niet vragen wie het is. De sectiekop noemt het profiel
  met **(App-bot)** en de filters die op "ik" gebouwd zijn
  (*Aan mij toegewezen*, *Mijn*, *Wacht op mijn review*) hebben niets om te matchen.
  Niets wordt als fout getoond.
- **Plaatsen als.** Elk bericht dat GitPoulpi plaatst, wordt getoond met de naam waarmee het wordt geplaatst
  (*Plaatsen als my-app[bot] (GitHub App-bot)* onder de kop van een
  [gesprek](#plaatsen-als) en in zijn opmerkingsvak; `posted_as` in het CLI-antwoord). Met een persoonlijk token is dat
  je login (`GET /user`). Met een App-token is het de naam **Plaatsen als**, bijvoorbeeld `my-app[bot]`: stel hem in voor
  elke repository in **Instellingen > Integraties** (`general.hosting_post_as`, ook
  `gitpoulpi config set general.hosting_post_as my-app[bot]`) of voor één repository in **Instellingen > Repositoryspecifieke
  instellingen > SSH & inloggegevens > Hosting**, wat voorgaat. Leeg gelaten vraagt GitPoulpi het aan GitHub (GraphQL
  `viewer { login }`) en leert anders de naam van de auteur van de eerste opmerking die het plaatst, en bewaart hem bij de
  repository. Die naam is ook hoe je eigen opmerkingen worden herkend, dus bewerken en verwijderen worden alleen daarop
  aangeboden (opmerkingen van een App moeten ook van een botaccount komen); zolang hij onbekend is, kunnen alleen de
  opmerkingen die deze installatie plaatste worden gewijzigd.
- Een App kan niet worden toegewezen. Combineer de opzet met het geblokkeerde toewijzen
  hieronder.
- **Gebruiken voor de host-API** aan- of uitzetten vraagt een persoon, net als een nieuw
  commando.

Alleen GitHub-profielen gebruiken het token van de repository zo; andere hosts houden het
eigen token van het profiel. (Een toegangstoken van een GitLab-project of -groep is ook een bot: het is het token van het profiel, getoond als
*projectbot*, en er hoeft niets te worden ingesteld.)

### Toewijzen blokkeren in een repository

**Instellingen > Repositoryspecifieke instellingen > SSH & inloggegevens** heeft een rij **Toewijzing**
(standaard **Toegestaan**, of **Geblokkeerd**; direct opgeslagen). Geblokkeerd is toewijzen
onmogelijk in de issues en pull requests van die repository, wie het ook vraagt:

- de toewijzingskiezer ontbreekt in de formulieren voor een nieuw issue en een nieuwe pull request
  (met een opmerking waarom), het paletrij *Hosting: Toegewezenen bewerken...* is
  grijs met de reden, en het rijmenu zegt het in plaats van te openen;
- elke client weigert een toewijzing voordat er iets wordt verstuurd, ook voor
  toegewezenen die bij het maken van een issue of een pull request worden meegegeven: de CLI antwoordt
  `{"refused": {"reason": "assignment_blocked", ...}}` met afsluitcode 2, en
  een agent via MCP krijgt dezelfde weigering;
- toegewezenen die er al zijn, blijven zichtbaar (alleen-lezen).

`gitpoulpi hosting assignment` toont de instelling, `--block` zet hem aan en
`--allow` zet hem weer uit; alleen een persoon kan weer toestaan (de MCP-server
kan het niet).

## 14. Vanaf de opdrachtregel

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

Zie de [CLI-naslag](cli-reference.md#opdrachten) voor elke opdracht. Een token
wordt nooit als argument geaccepteerd, en de MCP-server biedt `hosting login` niet
aan, dus een token gaat nooit door een agent heen. `hosting login` drukt
`stored: "os"` af als de credential store van het besturingssysteem het token bewaart, of
`stored: "session"` plus `store_error` als de store het weigerde en het token alleen voor
deze sessie in het geheugen leeft.

## Probleemoplossing

| Symptoom | Oplossing |
|---|---|
| *Verbinden* mislukt met een authenticatiefout | Het token is fout, verlopen of ingetrokken. Maak een nieuw token. |
| Een scope staat in het rood | Het token heeft hem niet. De verbinding werkt, maar die functie niet. Voeg de scope toe op de host, kies dan **Bewerken** en plak het nieuwe token. |
| De zijbalk heeft geen sectie PULL REQUESTS | Er is geen ingeschakeld profiel verbonden. Verbind er een en koppel dan de repository in de keuzelijst van de sectie. |
| Een rij *Opnieuw verbinden* in een lijst | Het token is kwijtgeraakt (een herstart zonder credential helper) of verlopen. Klik op de rij om het profiel te openen. |
| Een self-managed instance mislukt | Controleer dat het **Hostdomein** geen pad heeft, en dat je computer het certificaat van de instance vertrouwt. |

## Bekende beperkingen

- **Pull requests uit forks:** Azure DevOps en Bitbucket kunnen een verzoek uit een fork niet
  uitchecken (zie [Acties op een rij](#acties-op-een-rij)). Azure
  DevOps kan geen pull request maken vanuit een fork.
- **Trello** toont hooguit de eerste 1000 kaarten van een board. Gebruik het lijstfilter
  om het te beperken. **Jira**, **Linear** en **YouTrack** tonen hooguit 500
  issues per vernieuwing. Gebruik een JQL-fragment of een YouTrack-query om het te beperken.
- **Linear** neemt één toegewezene per issue: kies je er meerdere in de kiezer,
  dan wordt de eerste verstuurd. Zijn "mijlpaal" is de **cycle** van het team (projectmijlpalen
  worden niet gebruikt). Die van **YouTrack** is het veld `Fix versions`; een
  versie heeft geen nummer, dus de kiezer nummert de nog niet uitgebrachte op hun
  positie in het project. Geen van beide trackers heeft pull requests.
- **Redmine** neemt één toegewezene per issue (de eerste van de kiezer), heeft geen labels,
  en toont hooguit 500 issues per vernieuwing. Toegewezenen zijn de leden van het project
  (mensen en groepen) en worden onderscheiden op hun weergavenaam. De kolommen van het
  kanbanbord zijn de issuestatussen, en een kaart verplaatsen wijzigt de status; de standaard
  workflowregels van Redmine gelden (een rol die die overgang niet mag maken krijgt de
  weigering terug en de kaart springt terug). "Resolved" is geen gesloten status in de
  standaardgegevens van Redmine: de kolom Gesloten zijn de statussen "Closed" en "Rejected".
- **Bitbucket Data Center** heeft geen browserlinks voor nieuwe pull requests,
  branches of commits. De pagina's van pull requests openen normaal.
- **Azure DevOps** toont geen auteursavatars. De filters "mine" en "review requested"
  hebben een endpoint nodig dat persoonlijke toegangstokens kan weigeren. In dat
  geval verbindt het profiel zonder gebruikersnaam en zijn deze twee filters niet
  beschikbaar. Een ongeldig token krijgt een aanmeldpagina in plaats van een fout;
  GitPoulpi meldt dat als een geweigerd token.
- **Gitea en Forgejo:** een concept is een verzoek waarvan de titel met `WIP:` begint
  (de server heeft bij het maken geen conceptvlag). Een verzoek toont als reviewers
  iedereen die het al reviewde, en een reviewverzoek intrekken verwijdert alleen
  openstaande. Pull requests vanaf de lokale branch van een fork worden uitgecheckt
  via `refs/pull/<n>/head`, zoals bij GitHub. Vermeldingen worden gezocht met het
  filter `mentioned_by` van de server. De server toont geen tokenscopes, dus een
  ontbrekende wordt pas gevonden als een aanroep wordt geweigerd.
- **Het issuebord** verplaatst kaarten bij elke issuebron. Bij Jira voert een verplaatsing de workflowovergang uit die naar de kolom leidt;
  een overgang die om velden vraagt (bijvoorbeeld een resolutie) wordt door Jira geweigerd en de kaart springt terug met het bericht van Jira.
  Bij Azure DevOps springt een work item van een type zonder de doelstatus terug met de reden.
- **Bitbucket:** app-wachtwoorden bestaan niet meer; gebruik een API-token.
  Repositories worden workspace voor workspace getoond.
- **Jira Cloud:** alleen klassieke API-tokens werken (geen tokens met scopes, geen OAuth).
  Een nieuw Jira-issue krijgt een beschrijving in platte tekst.
- **Gesprekken:** de nieuwste 300 items laden eerst (**Oudere laden** voor de rest); een opmerking, een bewerking of een
  verwijdering die vanuit GitPoulpi is geplaatst, is uitgaand en kan er niet mee ongedaan worden gemaakt; wat elke host niet kan, staat in
  [de tabel](#wat-elke-host-niet-kan). Op Gerrit kan een bericht of opmerking niet worden bewerkt of verwijderd, en Gerrit-tekst
  is plat.
- Het maken van een pull request, werkitem, issue of kaart gebeurt op de host en
  **kan niet ongedaan worden gemaakt** vanuit GitPoulpi. Het uitchecken van een verzoek kan normaal
  ongedaan worden gemaakt.

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

**Waar nu heen.** Deze pagina is de installatie: tokens, verbinden, een repository koppelen, SSH-sleutels. Zodra een account is verbonden:

- de secties PULL REQUESTS en ISSUES van de zijbalk, het issuebord, een pull request of issue maken, de chip in de werkbalk en verwijzingen in commit-berichten:
  **[Pull requests en issues](pull-requests-and-issues.md)**;
- de weergave in de app van één issue of verzoek, met zijn opmerkingen, schrijven en plaatsen als:
  **[Gesprekken](conversations.md)**;
- een verzoek lezen en reviewen met opmerkingendiscussies en een openstaande review:
  **[Een pull of merge request reviewen](review.md)**;
- de pull requests en issues van een hele workspace in één lijst:
  **[De Radar](radar.md)**.

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

## 5. SSH-sleutels

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

## 6. Branchbescherming, voor het herschrijven van de geschiedenis

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

## 7. Accountbrede lijsten in de Radar

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

## 8. Optreden als GitHub App-bot

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
  [gesprek](conversations.md#plaatsen-als) en in zijn opmerkingsvak; `posted_as` in het CLI-antwoord). Met een persoonlijk token is dat
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

## 9. Vanaf de opdrachtregel

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
  uitchecken (zie [Acties op een rij](pull-requests-and-issues.md#acties-op-een-rij)). Azure
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
  [de tabel](conversations.md#wat-elke-host-niet-kan). Op Gerrit kan een bericht of opmerking niet worden bewerkt of verwijderd, en Gerrit-tekst
  is plat.
- Het maken van een pull request, werkitem, issue of kaart gebeurt op de host en
  **kan niet ongedaan worden gemaakt** vanuit GitPoulpi. Het uitchecken van een verzoek kan normaal
  ongedaan worden gemaakt.

[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · **Svenska** · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Värdtjänster och ärendehantering

GitPoulpi ansluter till de här produkterna:

- **Hosting** (pull requests, ärenden, kloning från ditt konto, SSH-nycklar):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea och Forgejo
  (inklusive Codeberg). **Gerrit Code Review** är hosting bara för granskning: dess ändringar är
  pull requests, det har inga ärenden.
- **Ärendehantering** (bara ärenden): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud och Server) och Redmine.
- **Lokala Markdown-anteckningar** (bara ärenden, ingen token, inget nätverk): en mapp med Markdown-anteckningar, till exempel en mapp i ett Obsidian-valv, se [Lokala ärenden och Obsidian](local-issues.md).

Varje integration använder en **personlig token**. Det finns inget GitPoulpi-konto
och ingen OAuth-inloggning. Integrationer är valfria och **opt-in per repository**:
ingenting hämtas förrän du länkar ett repository. Din token lagras i din
credential helper, aldrig i `config.toml`. Vanliga `git`-operationer (fetch, pull,
push) behöver ingen integration: de använder de uppgifter som ditt `git`
redan har.

## 1. Skapa en token hos värden

Anslutningsformuläret har en länk *Generera en token på <host>*. Den öppnar
värdens tokensida med rätt behörigheter redan valda. Så här gör du det för hand:

| Värd | Var | Behörigheter |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repositories, pull requests, ärenden), `read:user` (vem du är), `admin:public_key` (bara för att lägga till eller ta bort SSH-nycklar) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, på `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Skapa token **för just den organisationen**, inte för ”All accessible organizations”: globala tokens slutar fungera 2026-12-01. |
| Azure DevOps Server | Samma sida på din server (`https://<server>/<collection>/_usersSettings/tokens`) | Samma två behörigheter |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. För SSH-nycklar även `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profilbild > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, på `<host>/user/settings/applications` | Läs `user`; läs och skriv `repository`, `issue` och `user` (den sista för SSH-nycklar). Gitea visar inga behörigheter i efterhand, så GitPoulpi kan inte varna för en som saknas: ett anrop som vägras säger det. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. Nyckeln agerar som du. |
| YouTrack (Cloud och Server) | Din profil > Account Security > New token, på `<host>/users/me?tab=account-security` | Behörigheten `YouTrack`. En permanent token agerar som du. |
| Gerrit | Settings > HTTP Credentials > Generate new token (nyare servrar; äldre: Generate new password), på `<host>/settings/#HTTPCredentials` | Inga. Token är **HTTP-lösenordet** och agerar som du; användarnamnet anges bredvid. |
| Redmine | My account > API access key > Show, på `<host>/my/account`. Administratören måste ha slagit på **Administration > Settings > API > Enable REST web service**. | Inga. Nyckeln agerar som du. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (den klassiska sorten, utan behörigheter) | Inga. Token agerar som du. |
| Jira Data Center | Profile > Personal Access Tokens | Inga. Token agerar som du. |
| Trello | Först en **API-nyckel** från `trello.com/apps/admin` (skapa en Power-Up och generera sedan en API-nyckel). Sedan en **token** från `trello.com/1/authorize?...&key=<din nyckel>`. | `read`, `write` |

Ge token ett namn som ”GitPoulpi” och ett utgångsdatum. Kopiera token:
värden visar den bara en gång.

## 2. Anslut den

1. Tryck `Ctrl+,` och välj **Integrationer**.
2. Välj produkten till vänster. Produkterna är i två grupper: **Hosting** och
   **Ärendehantering**.
3. Tryck **Anslut**. För att lägga till ett andra konto för samma produkt, använd
   **+ Lägg till profil**.

   ![Sidan Integrationer: produkterna till vänster och en Anslut-knapp för GitHub](../images/settings-integrations.png)

4. Fyll i formuläret och klistra in token. Token visas i klartext medan du
   skriver, så se till att ingen tittar. Formuläret listar de behörigheter
   som behövs.

   ![Anslutningsformuläret: ett fält Personlig åtkomsttoken, behörigheterna repo, read:user och admin:public_key, en länk för att generera en token på github.com, och knapparna Anslut och Avbryt](../images/settings-connect.png)

5. Tryck **Anslut**. GitPoulpi kontrollerar token mot värden i
   bakgrunden. Sidan visar sedan ett kort med ditt namn och din inloggning,
   *Ansluten* i grönt, och SSH-nyckelsektionen. En token som saknar någon av de
   önskade behörigheterna ansluter ändå, och de saknade behörigheterna listas i rött.

På kortet: **Redigera** öppnar formuläret igen. **Koppla från** raderar token
och behåller profilen. **Ta bort** raderar båda, efter en bekräftelse.

Tangentbord på sidan Integrationer: `Tab` / `Shift+Tab` flyttar mellan
produktlistan, profilraden och formuläret. `a` lägger till en profil, `e` redigerar
den, `d` eller `Delete` tar bort den, `o` öppnar tokensidan, `Esc` går tillbaka ett
steg.

### Vad varje formulär frågar efter

Länken *Generera en token* fungerar först när du fyllt i värden eller
organisationen.

- **GitHub, GitLab:** bara token. **GitHub Enterprise** och
  **GitLab Self-Managed** frågar också efter **Värddomän**
  (`gitlab.example.com`; https om du inte skriver `http://`). Ett namn utan punkt
  (en Docker-tjänst eller en dator i det lokala nätverket) behöver en port eller `http://`:
  `gitea:3000`, `http://nas`. En egen server kan köras över
  vanlig http, till exempel en lokal container på `http://localhost:8080`. En token som skickas över
  vanlig http går okrypterad, så använd bara http i ett lokalt eller betrott
  nätverk.
- **Azure DevOps:** **Organisation**, namnet i
  `dev.azure.com/<organization>`. En adress `https://<org>.visualstudio.com`
  godtas också. Två organisationer kräver två profiler. Ett repository använder
  profilen för den organisation som dess remote nämner. **Azure DevOps Server**
  frågar efter **Samlings-URL**, till exempel
  `tfs.corp/tfs/DefaultCollection`. GitPoulpi använder inte systemets
  credential helper för Azure DevOps: Git Credential Manager lagrar där en
  inloggningstoken, inte en personlig åtkomsttoken.
- **Bitbucket:** din **Atlassian-kontots e-post** och en API-token. Om
  e-posten är tom skickas token som en bearer-token: använd det för en
  workspace- eller repository-åtkomsttoken. GitPoulpi hittar dina workspaces utifrån token.
  **Bitbucket Data Center:** **Server-URL** och en HTTP-åtkomsttoken.
- **Gitea, Forgejo:** **Värddomän** och token. **Forgejo** börjar på
  `https://codeberg.org`: ersätt den för en egen server. Gitea har ingen
  offentlig standard. En adress `http://localhost:3000` fungerar för en lokal server.
  Båda slagen delar en klient (Forgejo är en Gitea-fork med samma
  `/api/v1`); token skickas som `Authorization: token <token>`. Systemets
  credential helper används inte för dem.
- **Jira Cloud:** **Webbplats-URL** (`acme.atlassian.net`), din **Atlassian-kontots
  e-post** och en klassisk API-token. Jira-tokens med behörigheter stöds inte.
  **Jira Data Center:** **Server-URL** och en personlig åtkomsttoken.
- **Trello:** **API-nyckel** och **token**. API-nyckeln är offentlig, så den
  lagras i `config.toml` (som `account`). Token är hemlig och går till
  credential helpern. Formuläret har två länkar: *Hämta en API-nyckel på Trello* och,
  när du skrivit nyckeln, *Generera en token på Trello*. Nyckeln och token skickas i
  ett huvud, aldrig i en URL.

- **Linear:** bara **API-nyckeln**: Linear har en enda offentlig värd, så det
  finns ingen adress att skriva. Nyckeln skickas som den är i huvudet `Authorization`
  (Linear tar inget `Bearer` för personliga nycklar). För att `ENG-12`-referenserna
  i commit-meddelanden ska öppna rätt sida, lagra profilens URL med din
  arbetsyta (`https://linear.app/acme`, från CLI:t: `--url`); ärenderaderna
  bär alltid den exakta adressen själva.
- **YouTrack:** **Webbplats-URL** (`acme.youtrack.cloud` för Cloud, annars din egen
  server, med sin kontextsökväg om den har en, till exempel
  `host/youtrack`) och en permanent token, skickad som `Authorization: Bearer`. Samma
  klient betjänar Cloud och Server (`<url>/api`).
- **Gerrit:** **Server-URL** (med sin kontextsökväg om den har en), ditt **Användarnamn** och HTTP-
  lösenordet som token. De skickas som HTTP basic-autentisering; GitPoulpi talar med serverns REST-API
  under `/a/` och ignorerar raden `)]}'` som varje svar börjar med. Användarnamnet är inte hemligt och sparas i
  `config.toml` (som `account`), lösenordet går till credential helpern. Ett repository länkas till ett Gerrit-
  *projekt* under PULL REQUESTS (dess ändringar listas där); Gerrit har inga ärenden, och en ändring skapas genom
  att pusha (se *Push för granskning* i [Granska](review.md#gerrit)).
- **Redmine:** **Webbplats-URL** för din Redmine (med sin undersökväg om den har en,
  till exempel `host/redmine`) och din API-nyckel, skickad som `X-Redmine-API-Key`. Om
  Redmine svarar 403 eller en HTML-sida är REST-webbtjänsten av (be
  administratören, eller slå på den under **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack och Redmine är bara ärendehantering: inga pull requests, inga repositories att
klona och inga SSH-nycklar. Bitbucket har inga ärenden (Atlassian har lagt ned Bitbucket
Issues). Länka i stället sektionen ÄRENDEN för ett Bitbucket-repository till ett Jira-projekt.

## 3. Var token lagras

Token lagras i systemets **credential helper** (Git Credential
Manager på Windows, libsecret eller `pass` på Linux, Keychain på macOS). Den använder en
privat post som inte kan förväxlas med uppgifterna för en riktig värd. Den
skrivs aldrig till `config.toml` och skrivs aldrig ut.

Om ingen credential helper är konfigurerad, eller om den vägrar token, behåller GitPoulpi
token **i minnet bara för den här sessionen** och talar om det. Anslut
igen efter en omstart.

GitPoulpi letar efter en profils token i den här ordningen:

1. Miljövariabeln `GITPOULPI_TOKEN_<PROFILE ID>`. Använd den för att ange en
   token utan att lagra den, till exempel i CI.
2. Token som lagrats för profilen (`token_ref`).
3. Bara för GitHub- och GitLab-profiler: systemets credential helper för
   värden, när ingen annan aktiverad profil använder samma värd.

Varje profil är en `[[integrations]]`-post i `config.toml`. Se
[konfigurationsreferensen](config-reference.md#integrations-hosting-och-ärendehantering).

## Säkerhetskontrollen

En sparad token är bara så säker som adressen den skickas till. GitPoulpi kommer därför ihåg, för varje integration, vad den anslöts till, och stannar när det ändras:

- **Adressen.** Första gången en integration används (eller ansluts) registreras dess adress i `trust.toml`, i den privata datamappen som bara användaren kan läsa (`<data>/private/`), inte i `config.toml`: att redigera konfigurationen kan aldrig vidga vad som betros. Om adressen i konfigurationen inte längre är den registrerade skickas ingenting dit och token läses inte ens. Olika stavningar av en adress (`HTTPS://Git.Example.com:443/`) är samma adress; en värd som liknar den är det inte.
- **Kontot.** Efter att du har anslutit en token, och sedan ungefär en gång per dygn (och en gång per körning för en token som kommer från `GITPOULPI_TOKEN_*` eller credential helpern), frågar GitPoulpi värden vem token tillhör och jämför kontots **id**, inte inloggningsnamnet. En token som nu tillhör någon annan stoppas.
- **Certifikatet.** Servrar vars certifikat inte leder till en känd utfärdare (en privat CA, en självsignerad server) godtas bara för den exakta nyckel du godtog, och TLS-handskakningen bevisar fortfarande att servern har den nyckeln. Med strikt fästning på måste nyckeln matcha även när certifikatet leder till en offentlig rot; priset är att varje nyckelrotation frågar igen, vilket är anledningen till att det är frivilligt. De offentliga värdarna (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) har ingen certifikatpost.
- **Vanlig http.** En `http://`-adress som inte är loopback nekas om du inte har tillåtit den för den integrationen.

Varje begäran hålls också till integrationens egen adress (och, för en offentlig värd, dess API-värd som `api.github.com`): en begäran någon annanstans nekas innan den skickas, och en omdirigering till en annan adress följs inte.

**Integrationer som anslöts innan den här kontrollen fanns** får sin post tyst, från konfigurationen som den var, första gången de används efter uppgraderingen: de behåller dagens förtroendenivå och ingenting frågar.

### Från kommandoraden

Ett stoppat kommando misslyckas med slutkod 2 och skriver ut vad som ändrades (se [CLI-referensen](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` visar vad som är betrott; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` och `--allow-insecure-http on|off` godtar en sak var och namnger det exakta värdet. Efter att ha godtagit en ny adress anger du token igen. Att koppla från (`hosting logout`) eller ta bort en integration raderar dess token, dess förtroendepost och det som cachats från den (de sparade repositorylistorna i dialogen Klona, kontolistorna i Radar och utkasten till fjärrgranskningar från dess värd).

En agent (MCP) kan inte godta något: den får veta att säkerhetskontrollen behöver dig.

### I terminalgränssnittet

En stoppad begäran öppnar dialogen **Säkerhetskontroll av integration** över det du höll på med. Den säger vad som ändrades (den gamla och den nya adressen, det gamla och det nya kontot med sina id, eller det gamla och det nya certifikatet som en liten tabell med SHA-256-fingeravtrycket i grupper om fyra, ämne, utfärdare och giltighet) och att token inte skickades. Svaren är numrerade:

1. **Koppla från** raderar den sparade token, förtroendeposten och det som cachats från integrationen.
2. **Lita på** den nya adressen, det nya kontot eller det nya certifikatet (en okrypterad adress: **Tillåt vanlig http**).
3. **Avbryt** ändrar ingenting.

Inget svar är markerat förrän du trycker på en piltangent eller en siffra, så ett förlupet Enter godtar ingenting. Esc och stängningskorset avbryter; ett klick utanför dialogen gör ingenting. En lång text rullar med hjulet, Page Up/Page Down eller rullningslisten. Flera kontroller köas (en per integration och slag av ändring) och ställs en efter en. En avbruten kontroll dyker inte upp igen så länge den är oförändrad; **Integrationer: Granska säkerhetskontroll** i kommandopaletten tar tillbaka den.

Efter att du har litat på en ny **adress** öppnas integrationens formulär med tokenfältet fokuserat och tipset ”Du bör ange token igen.”: den sparade token skickades aldrig dit. I formuläret godtas en adress som ändrats med flit tillsammans med en token som skrivits för den; utan token ber formuläret om en.

Formuläret har också två reglage. **Strikt certifikatfästning** (för en https-adress) fäster certifikatnyckeln som servern presenterar när du ansluter. **Tillåt vanlig http (okrypterat)** erbjuds bara för en `http://`-adress som inte är den här datorn, med en röd varning om att token färdas i klartext. Det anslutna kortet visar det fästa fingeravtrycket.

### Vad token får göra, och till när

Att ansluta frågar också värden vad token kan göra. GitHub (klassiska tokens), GitLab och Bitbucket (när den skickar scopes) rapporterar sina scopes: kortet visar en gul rad ”Den här token har fler rättigheter än GitPoulpi behöver” som listar de scopes som går utöver det GitPoulpi använder, de känsliga (som `delete_repo`, `admin:org`, `sudo`) i rött, med rådet att föredra en finkornig token med utgångsdatum. En klassisk GitHub-token (`ghp_...`) får det rådet även när dess scopes är exakta. En finkornig GitHub-token kan inte tillfrågas om sina behörigheter, så bara dess utgångsdatum visas. Kortet visar utgångsdatumet när värden anger det. En token utan utgångsdatum visar ingen påminnelse, och de andra produkterna rapporterar inga scopes och ger ingen varning. Samma fält finns i JSON från `hosting integrations login` och `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). De läses när du ansluter, så kortet visar dem för sessionen som anslöt.

## 4. Länka ett repository

Länkning är explicit och per repository. När en profil är ansluten visar
Refs-sidopanelen en sektion **PULL REQUESTS** och en sektion **ÄRENDEN** efter TAGS.
Varje sektion har två rullgardinsmenyer: leverantören och projektet.

- Leverantörsmenyn **föreslår** de konton som matchar repositoryts
  remotes (`origin` först), som rader som `owner/name · origin · GitHub`. Ett
  val sätter både leverantör och projekt.
- Ingenting länkas automatiskt. Jira, Trello, Linear, YouTrack och Redmine föreslås aldrig, eftersom
  ingen remote nämner dem: välj dem själv.
- **Ingen (koppla bort det här repositoryt)**, överst i menyn, kopplar bort
  sektionen. Ingenting länkar den igen av sig självt.
- Länken sparas i filen `.gitpoulpi/repo-state.toml` i
  repositoryt.

De två sektionerna länkas **separat**. Till exempel kan pull requests komma
från Azure DevOps och ärenden från Jira.

| Leverantör | Projekt för PULL REQUESTS | Projekt för ÄRENDEN |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | samma |
| Azure DevOps | `Project/Repo` (organisationen kommer från profilen) | `Project` (arbetsobjekt) |
| Gitea, Forgejo | `owner/name` | samma |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | inget: länka Jira |
| Jira | inget | projektnyckeln, till exempel `ABC` |
| Trello | inget | en board |
| Linear | inget | en teamnyckel, till exempel `ENG` (teamen listas) |
| YouTrack | inget | ett projekts korta namn, till exempel `ABC` |
| Gerrit | Gerrit-projektets namn, till exempel `team/tool` | inget |
| Redmine | inget | en projektidentifierare, till exempel `bench` (sista delen av projektets adress) |

Projektlistan för ÄRENDEN är sökbar. Du kan också skriva ett projekt som
listan inte visar.

**Extra fråga.** En ärendekälla har en valfri filterrad under projektmenyn.
Den kombineras (AND) med projektet:

- Jira: ett **JQL**-fragment (`component = UI AND labels = backend`).
- Azure DevOps: ett **WIQL**-villkor (`[System.AssignedTo] = @Me`).
- YouTrack: en **YouTrack-fråga** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Den sätts inom parentes, så ett `or` kan inte vidga projektet. Ett eget
  `sort by:` ersätter standarden `sort by: updated desc`.
- Redmine: ingen (sektionens filter för tillstånd och ”tilldelade mig” räcker).
- Linear: ingen (sektionens filter för tillstånd och ”tilldelade mig”
  räcker).
- Trello: en rullgardinsmeny där du kryssar i de **listor** som ska visas. Utan kryss
  visas varje öppen lista.
- GitHub och GitLab ignorerar den.

`Enter` behåller en skriven fråga, `Esc` släpper den. Palett:
*Ärenden: Redigera fråga / listor...*.

Från CLI:t: `gitpoulpi hosting link pulls --id <profile>` (eller `issues`),
`--clear` för att koppla bort, `--filter` för den extra frågan. Se
[CLI-referens](cli-reference.md#kommandon).

### Klona från ett anslutet konto

En ansluten profil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) är också en källa i dialogen **Klona** (knappen **Klona** på fliken Hem, eller `c` där). Varje profil står under **Klona med URL** som en egen källa (`GitHub · name`, `GitLab · name`); GitHub och GitLab blandas aldrig. Välj en så läses dess repositories in, grupperade efter ägare: först ditt eget konto, sedan en sektion per organisation eller grupp (`quarry/tools` för en GitLab-undergrupp), var och en under en rubrik med antalet. Ett lås markerar ett privat repository.

![Dialogen Klona på ett anslutet GitHub-konto: en sektion per ägare och ett lås på ett privat repository](../images/clone-hosted.png)

Sökfältet filtrerar över alla sektioner (namn, fullständig sökväg och beskrivning); en sektion utan träff försvinner. `Enter` på en rad fyller klonen med det repositoryts HTTPS-URL och ett mappnamn.

![Dialogen Klona där sökningen begränsar listan till en organisation](../images/clone-hosted-search.png)

Listan för en profil som du har öppnat tidigare visas direkt från en sparad kopia medan värden tillfrågas på nytt (*uppdaterar...* bredvid etiketten); när svaret kommer visas nya repositories och borttagna försvinner, och det markerade och det valda repositoryt stannar där de är. Om uppdateringen misslyckas står den sparade listan kvar med en notis, och orsaken finns i Aktivitetsloggen.

Om värden vägrar token visar dialogen *kunde inte lista repositories* med orsaken i stället för listan (och den sparade kopian raderas); de andra källorna fungerar som vanligt.

## 5. Pull requests och ärenden i sidopanelen

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

### Åtgärder på en rad

- `Enter` eller ett dubbelklick öppnar raden i din webbläsare.
- Högerklick, eller `.`, öppnar radmenyn:
  - **Öppna i webbläsare**, **Kopiera URL**, **Kopiera nummer**.
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
| `Enter` / dubbelklick | öppna raden i webbläsaren |
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

## 5b. Ärendetavlan

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
ärendet hos värden (anteckningen, för lokala ärenden); `e` redigerar tilldelade, etiketter och milstolpe; `n` startar ett nytt ärende i
den valda kolumnen; `a`, `l`, `M` och `/` filtrerar på tilldelad, etikett, milstolpe och text, `x` rensar filtren; `r`
läser värden igen. Tavlan läses om när sidofältets listor uppdateras. Radar har ingen tavla.

## 6. Skapa en pull request

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

## 7. Skapa ett ärende

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

## 8. Skapa på värdens egen webbsida

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

## 9. Chippet i verktygsfältet

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

## 10. När listorna uppdateras

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

## 11. Terminologi

GitHub, Azure DevOps och Bitbucket säger ”pull request”. GitLab säger ”merge
request”. *Terminologi för granskningsbegäran* (`ui.pr_terminology`, under
Inställningar > Integrationer) väljer ordet för varje etikett, meny, palettkommando
och meddelande:

- **Auto (per värd)**: följer repositoryts länkade leverantör (”Pull
  Request” så länge inget är länkat).
- **Pull request** eller **Merge request**: alltid det ordet.

Ändringen gäller direkt. CLI:t och MCP säger alltid ”pull/merge request”.
Den här sidan säger ”pull request”.

## 12. Referenser i commit-meddelanden och grennamn

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

### Infoga en ärendereferens

I commit-composern öppnar knappen `#▾` längst till höger i Sammanfattning
en filtrerbar lista över de länkade ärendena. Ärendet som den aktuella grenen
nämner kommer först, markerat *(från grenen)*. `Enter` eller ett klick infogar dess referens
(`ABC-123`, `#12`, `AB#12`) vid markören. Tangentbord: `Alt+R` i composern.
Palett: *Commit: Infoga ärendereferens...*. Utan en länkad ärendelista är
knappen nedtonad och säger *länka först en ärendelista i sidofältet*.
GitPoulpi lägger aldrig till en referens av sig självt.

## 13. SSH-nycklar

### SSH-nyckel per profil

En ansluten profil har en sektion **SSH-nyckel** under sitt kort. Utan nyckel
väljer du ett av:

- **Standardnyckel**: din ssh-agent eller `~/.ssh`. Inget ändras.
- **Generera ny nyckel**: en valfri titel (standard
  `GitPoulpi <host> <date>`). Med **Lägg till den publika nyckeln på <host>** ikryssat
  laddar GitPoulpi upp den, om värden inte redan har den. Nyckeln
  skrivs utan lösenfras till `~/.ssh/gitpoulpi_<profile id>_ed25519` och
  `.pub`. Ingen `ssh-keygen` behövs. En befintlig fil skrivs aldrig över.
- **Egen nyckel**: skriv en sökväg eller **Bläddra** efter en fil med privat nyckel, och tryck sedan
  **Använd den här nyckeln**.

När en nyckel är kopplad visar sektionen `titel (SHA256:...)` med:

- **Ta bort**: koppla loss nyckeln, och radera den hos värden om GitPoulpi lade till den.
- **Radera**: samma sak, och radera även nyckelfilerna, efter en bekräftelse
  som listar båda sökvägarna.

### SSH-nyckel för ett repository

För att få ett repository att använda en viss nyckel, öppna flikmenyn (`Ctrl+T`,
eller högerklicka på fliken) och välj **SSH & inloggning...**, eller kör
palettkommandot *Repository: SSH & inloggning...*, och välj **Nyckelfil** (eller
**Kontonyckel** för nyckeln i en integrationsprofil) under SSH. Valet sparas i
GitPoulpis privata mapp: inget skrivs till repositoryt eller till gits config.
Se [Inställningar](settings.md#repo-specifikt-ssh--inloggning).

Ett repository som fortfarande har en egen `core.sshCommand`, från den tidigare
fastsättningen ”SSH-nyckel för det här repositoryt”, visar den på den sidan,
skrivskyddad, med **Flytta till GitPoulpi**.

CLI och MCP: `gitpoulpi repo-auth` (valet per repository) och, för själva
`core.sshCommand`-fastsättningen, `gitpoulpi ssh-command show | set --key <file> |
clear`.

Den globala nyckeln under Inställningar > SSH & inloggning (*Sökväg till privat nyckel*, eller
*Generera nyckelpar*) gäller för varje repository utan ett eget val.
Ett repositorys nyckelfil eller fastsättning går före den.

## 13b. Grenskydd, för en historikomskrivning

När du skriver om historiken ([omskrivningsvyn](history-rewrite.md)) frågar
GitPoulpi värden vilka grenar den skyddar, så att de lämnas utanför
omskrivningen som standard. Den frågar bara för ett repository du har **länkat**
(avsnittet med pull requests eller ärenden i sidopanelen, eller en fäst profil),
med den profilens token, och en token som inte kan läsa det ger bara ”okänt”:
ingenting misslyckas. Vad varje leverantör svarar, och vad token behöver:

| Värd | Vad som läses | Token behöver |
|---|---|---|
| GitHub (och Enterprise Server) | flaggan `protected` på grenarna, och de aktiva rulesets för grenar (mönstren `~DEFAULT_BRANCH`, `~ALL` och `refs/heads/...`) | en klassisk token med `repo` (`public_repo` för ett publikt repository), eller en finkornig token med Metadata (read). Ett ruleset som token inte kan läsa sägs ifrån, de klassiska reglerna räknas ändå |
| GitLab (gitlab.com och egen drift) | `protected_branches` (namn och `*`-mönster) | `read_api`, och en roll som får se repositoryts inställningar |
| Azure DevOps (Services och Server) | repositoryts aktiverade, blockerande grenpolicyer (omfången `Exact`, `Prefix` och `DefaultBranch`) | Code (read). Behörigheten ”force push” är en säkerhetsinställning och läses inte |
| Bitbucket Cloud | de grenbegränsningar som stoppar en push (`push`, `force`, `delete`) | `repository:admin`. En begränsning på grenmodellen löses inte upp |
| Bitbucket Data Center | de grenbehörigheter som stoppar en force push (`fast-forward-only`, `pull-request-only`, `read-only`) | repositoryadministratör |

Inställningssidan för skyddet (och för standardgrenen, och för borttagning av
repositoryt) byggs från repositoryts remote-URL för GitHub, GitLab, Azure DevOps
och Bitbucket, även när repositoryt inte är länkat, och visas tillsammans med de
tre vägarna förbi när värden avvisar en push.

## 13c. Kontoövergripande listor i Radar

Radar bevakar normalt de repositories du har klonat. En profil kan också
lista det som rör dig på hela värden, klonat eller inte, med tre reglage i
sin tabell `[integrations.radar]` (alla av som standard; se
[konfigurationsreferensen](config-reference.md#integrationsradar-kontoövergripande-listor)):

- **granskningsbegäranden**: öppna begäranden som ber om din granskning;
- **öppna begäranden**: alla öppna begäranden i din egen namnrymd och i varje organisation eller
  grupp du är medlem i, eller i de namnrymder du anger. Aldrig hela värden;
  profilens **Namnrymder** (en per rad i Inställningar > Integrationer > Radar, valfritt
  som reguljära uttryck) begränsar alla tre listorna: en rad matchar ett repositorys fulla
  sökväg när den är lika med den eller är ett sökvägsprefix vid ett `/` (`ptm` behåller
  `ptm/produits/x`, inte `ptmx/y`), och tillämpas på de rader värden returnerar, plus på
  serversidan där värdens sökning tillåter det (GitHubs kvalificerare `user:` / `repo:`);
- **tilldelade ärenden**: öppna ärenden tilldelade dig.

GitHub, GitLab (inklusive egen drift), Gitea, Forgejo, Azure DevOps (Services
och Server), Bitbucket Data Center och Gerrit svarar. Så frågas var och en:

| Värd | Begäranden | Ärenden |
|---|---|---|
| Azure DevOps | aktiva pull requests i hela organisationen (`searchCriteria.reviewerId` för dem som väntar på dig; en begäran du redan har röstat på lämnas utanför); när servern avvisar vägen på organisationsnivå frågas de första 20 projekten ett i taget. `namespaces` är projektnamn | arbetsobjekt tilldelade dig som inte är Done, Closed eller Removed |
| Bitbucket Data Center | din instrumentpanel: rollen `REVIEWER` som ännu inte godkänt för granskningsbegäranden, `AUTHOR` och `PARTICIPANT` för öppna begäranden; `namespaces` är projektnycklar | inga (Data Center har ingen ärendehantering; en notering säger det) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; öppna begäranden är `is:open projects:<prefix>` per namnrymd (ett prefix för projektsökvägen), eller dina egna och granskade ändringar när ingen anges; en ändring under arbete är ett utkast | inga |

Bitbucket Cloud har inga kontoövergripande rader: det erbjuder ingen slutpunkt som listar
begäranden över repositories, och att fråga varje repository skulle bli för många
anrop. Dess klonade repositories behåller sina egna listor, och `gitpoulpi hosting
radar --id <bitbucket>` säger det. När Gerrit har fler ändringar än gränsen den
returnerar markeras listan som avkortad med en notering.

Utkast lämnas utanför om du inte behåller dem. Varje lista är begränsad: 300 rader per slag, med en
notering som säger hur många fler det fanns, och på GitHub några få sökanrop per
omgång, eftersom dess sök-API tillåter bara 30 i minuten. En namnrymd som token
inte kan läsa blir en notering och de andra läses in ändå; en avvisad token eller en
hastighetsgräns får bara den profilen att misslyckas. Svaren återanvänds i fem minuter
(`radar-account.json` i konfigurationsmappen).

I Inställningar > Integrationer har en ansluten kodvärdsprofil ett **Radar**-block
under sin SSH-nyckel: en kryssruta per lista (*Begäranden som väntar på min granskning*,
*Alla öppna begäranden*, *Ta med utkast*, *Ärenden tilldelade mig*, den sista
saknas för värdar utan ärenden), ett fält *Namnrymder* (en post per rad, med en kryssruta *Reguljära uttryck*; tomt
betyder ditt eget och alla du tillhör) och valet *Begäran från ett
repository som inte är klonat*: *Fråga*, *Klona det* eller *Granska via API:et*.
Allt sparas när du ändrar det. Värdar utan någon kontoövergripande fråga (Bitbucket
Cloud) visar i stället en rad som säger det. Kommandopaletten når blocket
med *Inställningar: Radar-alternativ för en profil...*, och sökningen i Inställningar hittar varje
kontroll på dess etikett.

Från kommandoraden: `gitpoulpi hosting integrations radar <id> --review-requests on`
slår på en lista, `gitpoulpi hosting radar --json` skriver ut listorna (och
`--refresh` hoppar över cachen). AI-agenter har samma två kommandon över MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 14. Från kommandoraden

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
```

Se [CLI-referens](cli-reference.md#kommandon) för varje kommando. En token
godtas aldrig som argument, och MCP-servern erbjuder inte
`hosting login`, så en token passerar aldrig via en agent. `hosting login`
skriver ut `stored: "os"` när operativsystemets credential store behåller token,
eller `stored: "session"` plus `store_error` när lagret avvisade den och token
ligger i minnet bara under den här sessionen.

## Felsökning

| Symptom | Åtgärd |
|---|---|
| *Anslut* misslyckas med ett autentiseringsfel | Token är fel, har gått ut eller har återkallats. Skapa en ny. |
| En behörighet listas i rött | Token saknar den. Anslutningen fungerar, men den funktionen gör det inte. Lägg till behörigheten hos värden, välj sedan **Redigera** och klistra in den nya token. |
| Sidopanelen har ingen sektion PULL REQUESTS | Ingen aktiverad profil är ansluten. Anslut en och länka sedan repositoryt i sektionens rullgardinsmeny. |
| En rad *Anslut igen* i en lista | Token förlorades (en omstart utan credential helper) eller har gått ut. Klicka på raden för att öppna profilen. |
| En self-managed-instans misslyckas | Kontrollera att **Värddomän** saknar sökväg, och att din dator litar på instansens certifikat. |

## Kända begränsningar

- **Pull requests från forkar:** Azure DevOps och Bitbucket kan inte checka ut en
  begäran från en fork (se [Åtgärder på en rad](#åtgärder-på-en-rad)). Azure
  DevOps kan inte skapa en pull request från en fork.
- **Trello** visar högst de första 1000 korten på en board. Använd listfiltret
  för att begränsa. **Jira**, **Linear** och **YouTrack** listar högst 500
  ärenden per uppdatering. Använd ett JQL-fragment eller en YouTrack-fråga för att begränsa.
- **Linear** tar en tilldelad per ärende: om du väljer flera i väljaren
  skickas den första. Dess ”milstolpe” är teamets **cykel** (projektmilstolpar
  används inte). **YouTrack**s är fältet `Fix versions`; en
  version har inget nummer, så väljaren numrerar de ej släppta efter deras
  position i projektet. Ingen av tjänsterna har pull requests.
- **Redmine** tar en tilldelad per ärende (den första i väljaren), har inga etiketter,
  och listar högst 500 ärenden per uppdatering. Tilldelade är projektets medlemmar
  (personer och grupper) och skiljs åt på sitt visningsnamn. Kanban-tavlans
  kolumner är ärendestatusarna, och att flytta ett kort ändrar statusen; Redmines
  standardregler för arbetsflödet gäller (en roll som inte får göra den övergången får
  avslaget tillbaka och kortet hoppar tillbaka). ”Resolved” är ingen stängd status i Redmines
  standarddata: kolumnen Stängda är statusarna ”Closed” och ”Rejected”.
- **Bitbucket Data Center** har inga webbläsarlänkar för nya pull requests,
  grenar eller commits. Sidorna för pull requests öppnas som vanligt.
- **Azure DevOps** visar inga författaravatarer. Filtren ”mina” och ”granskning begärd”
  kräver en slutpunkt som kan avvisa personliga åtkomsttokens. I så fall
  ansluter profilen utan användarnamn och dessa två filter är inte
  tillgängliga. En ogiltig token får en inloggningssida i stället för ett fel;
  GitPoulpi rapporterar det som en avvisad token.
- **Gitea och Forgejo:** ett utkast är en begäran vars titel börjar med `WIP:`
  (servern har ingen utkastflagga vid skapande). En begäran listar som granskare
  alla som redan har granskat den, och att dra tillbaka en granskningsbegäran tar bara
  bort väntande. Pull requests från en forks lokala gren checkas
  ut via `refs/pull/<n>/head`, som GitHub. Omnämnanden söks med
  serverns filter `mentioned_by`. Servern visar inga token-behörigheter, så en
  som saknas upptäcks först när ett anrop vägras.
- **Ärendetavlan** flyttar kort på varje ärendekälla. På Jira kör en flytt arbetsflödesövergången som leder till
  kolumnen; en övergång som ber om fält (en lösning, till exempel) vägras av Jira och kortet går tillbaka med
  dess meddelande. På Azure DevOps går kortet tillbaka med orsaken när work item-typen saknar måltillståndet.
- **Bitbucket:** app-lösenord finns inte längre; använd en API-token.
  Repositories listas workspace för workspace.
- **Jira Cloud:** bara klassiska API-tokens fungerar (inga tokens med behörigheter, ingen OAuth).
  Ett nytt Jira-ärende får en beskrivning i ren text.
- Att skapa en pull request, ett arbetsobjekt, ett ärende eller ett kort sker hos värden och
  **kan inte ångras** från GitPoulpi. Att checka ut en begäran kan ångras
  som vanligt.

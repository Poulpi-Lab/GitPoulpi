[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · **Italiano** · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Integrazioni di hosting e issue tracker

GitPoulpi si collega a questi prodotti:

- **Hosting** (pull request, issue, clonazione dal tuo account, chiavi SSH):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea e Forgejo
  (compreso Codeberg). **Gerrit Code Review** è hosting solo per la revisione: le sue
  modifiche sono le pull request, non ha issue.
- **Issue tracker** (solo issue): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud e Server) e Redmine.
- **Note Markdown locali** (solo issue, nessun token, nessuna rete): una cartella di note Markdown, per esempio una cartella di un vault Obsidian, vedi [Issue locali e Obsidian](local-issues.md).

Ogni integrazione usa un **token personale**. Non esiste un account GitPoulpi e
nessun accesso OAuth. Le integrazioni sono facoltative e **si attivano per
repository**: non viene recuperato nulla finché non colleghi un repository. Il
tuo token è conservato nel tuo credential helper, mai in `config.toml`. Le
normali operazioni `git` (fetch, pull, push) non richiedono un'integrazione:
usano le credenziali che il tuo `git` ha già.

## 1. Crea un token sull'host

Il modulo di connessione ha un link *Genera un token su <host>*. Apre la pagina
dei token dell'host con gli ambiti (scope) giusti già selezionati. Per farlo a
mano:

| Host | Dove | Scope |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repository, pull request, issue), `read:user` (chi sei), `admin:public_key` (solo per aggiungere o rimuovere chiavi SSH) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, su `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Crea il token **per quella organizzazione**, non per "All accessible organizations": i token globali smettono di funzionare il 2026-12-01. |
| Azure DevOps Server | La stessa pagina sul tuo server (`https://<server>/<collection>/_usersSettings/tokens`) | Gli stessi due scope |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Per le chiavi SSH, anche `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Immagine del profilo > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, su `<host>/user/settings/applications` | Lettura di `user`; lettura e scrittura di `repository`, `issue` e `user` (l'ultimo per le chiavi SSH). Gitea non mostra gli scope a posteriori, quindi GitPoulpi non può avvisare di uno mancante: una chiamata rifiutata lo dice. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Lettura, scrittura, creazione di issue. La chiave agisce a tuo nome. |
| YouTrack (Cloud e Server) | Il tuo profilo > Account Security > New token, su `<host>/users/me?tab=account-security` | Lo scope `YouTrack`. Un token permanente agisce a tuo nome. |
| Gerrit | Settings > HTTP Credentials > Generate new token (server recenti; quelli più vecchi: Generate new password), su `<host>/settings/#HTTPCredentials` | Nessuno. Il token è la **password HTTP** e agisce a tuo nome; il nome utente va accanto. |
| Redmine | My account > API access key > Show, su `<host>/my/account`. L'amministratore deve aver attivato **Administration > Settings > API > Enable REST web service**. | Nessuno. La chiave agisce a tuo nome. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (quello classico, senza scope) | Nessuno. Il token agisce a tuo nome. |
| Jira Data Center | Profile > Personal Access Tokens | Nessuno. Il token agisce a tuo nome. |
| Trello | Prima una **chiave API** da `trello.com/apps/admin` (crea un Power-Up, poi genera una chiave API). Poi un **token** da `trello.com/1/authorize?...&key=<la tua chiave>`. | `read`, `write` |

Dai al token un nome come "GitPoulpi" e una data di scadenza. Copia il token:
l'host lo mostra una sola volta.

## 2. Collegalo

1. Premi `Ctrl+,` e scegli **Integrazioni**.
2. Scegli il prodotto a sinistra. I prodotti sono in due gruppi: **Hosting** e
   **Issue tracker**.
3. Premi **Connetti**. Per aggiungere un secondo account dello stesso prodotto,
   usa **+ Aggiungi profilo**.

   ![La pagina Integrazioni: i prodotti a sinistra e un pulsante Connetti per GitHub](../images/settings-integrations.png)

4. Compila il modulo e incolla il token. Il token è mostrato in chiaro mentre lo
   digiti, quindi assicurati che nessuno stia guardando. Il modulo elenca gli
   scope di cui ha bisogno.

   ![Il modulo di connessione: un campo Token di accesso personale, gli scope repo, read:user e admin:public_key, un link per generare un token su github.com, e i pulsanti Connetti e Annulla](../images/settings-connect.png)

5. Premi **Connetti**. GitPoulpi verifica il token con l'host in background. La
   pagina mostra poi una scheda con il tuo nome e login, *Connesso* in verde e la
   sezione delle chiavi SSH. Un token a cui manca uno degli scope richiesti si
   connette comunque, e gli scope mancanti sono elencati in rosso.

Sulla scheda: **Modifica** riapre il modulo. **Disconnetti** elimina il token e
mantiene il profilo. **Rimuovi** elimina entrambi, dopo una conferma.

Tastiera nella pagina Integrazioni: `Tab` / `Shift+Tab` si spostano tra l'elenco
dei prodotti, la fascia dei profili e il modulo. `a` aggiunge un profilo, `e` lo
modifica, `d` o `Delete` lo rimuove, `o` apre la pagina dei token, `Esc` torna
indietro di un passo.

### Cosa chiede ogni modulo

Il link *Genera un token* funziona solo dopo aver compilato l'host o
l'organizzazione.

- **GitHub, GitLab:** solo il token. **GitHub Enterprise** e
  **GitLab Self-Managed** chiedono anche il **Dominio host**
  (`gitlab.example.com`; https, a meno che tu non scriva `http://`). Un nome senza punto
  (un servizio Docker o una macchina della rete locale) richiede una porta o `http://`:
  `gitea:3000`, `http://nas`. Un server self-hosted può
  essere in semplice http, per esempio un container locale su `http://localhost:8080`. Un token
  inviato in semplice http viaggia non cifrato, quindi usa http solo su una rete
  locale o fidata.
- **Azure DevOps:** l'**Organizzazione**, il nome in
  `dev.azure.com/<organization>`. È accettato anche un indirizzo
  `https://<org>.visualstudio.com`. Due organizzazioni richiedono due profili.
  Un repository usa il profilo dell'organizzazione indicata dal suo remoto.
  **Azure DevOps Server** chiede l'**URL della raccolta**, ad esempio
  `tfs.corp/tfs/DefaultCollection`. GitPoulpi non usa il credential helper del
  sistema per Azure DevOps: Git Credential Manager vi conserva un token di
  accesso, non un token di accesso personale.
- **Bitbucket:** la tua **E-mail dell'account Atlassian** e un token API. Se
  l'e-mail è vuota, il token viene inviato come bearer token: usalo per un token
  di accesso a workspace o repository. GitPoulpi trova i tuoi workspace dal token.
  **Bitbucket Data Center:** l'**URL del server** e un token di accesso HTTP.
- **Gitea, Forgejo:** il **Dominio host** e il token. **Forgejo** parte da
  `https://codeberg.org`: sostituiscilo per un server self-hosted. Gitea non ha
  un predefinito pubblico. Un indirizzo `http://localhost:3000` funziona per un server
  locale. I due tipi condividono un solo client (Forgejo è un fork di Gitea con la
  stessa `/api/v1`); il token viene inviato come `Authorization: token <token>`. Il
  credential helper del sistema non viene usato per loro.
- **Jira Cloud:** l'**URL del sito** (`acme.atlassian.net`), la tua **E-mail
  dell'account Atlassian** e un token API classico. I token Jira con scope non
  sono supportati. **Jira Data Center:** l'**URL del server** e un token di
  accesso personale.
- **Trello:** la **chiave API** e il **token**. La chiave API è pubblica, quindi
  è conservata in `config.toml` (come `account`). Il token è segreto e va nel
  credential helper. Il modulo ha due link: *Ottieni una chiave API su Trello* e,
  dopo aver digitato la chiave, *Genera un token*. La chiave e il token vengono
  inviati in un'intestazione, mai in un URL.
- **Linear:** solo la **chiave API**: Linear ha un solo host pubblico, quindi non
  c'è nessun indirizzo da digitare. La chiave viene inviata così com'è
  nell'intestazione `Authorization` (Linear non vuole `Bearer` per le chiavi
  personali). Perché i riferimenti `ENG-12` dei messaggi di commit aprano la pagina
  giusta, salva l'URL del profilo con la tua area di lavoro
  (`https://linear.app/acme`, da CLI: `--url`); le righe delle issue portano
  comunque sempre l'indirizzo esatto.
- **YouTrack:** l'**URL del sito** (`acme.youtrack.cloud` per Cloud, altrimenti il
  tuo server, con il suo percorso di contesto se ne ha uno, ad esempio
  `host/youtrack`) e un token permanente, inviato come `Authorization: Bearer`. Lo
  stesso client serve Cloud e Server (`<url>/api`).
- **Gerrit:** l'**URL del server** (con il suo percorso di contesto se ne ha uno), il tuo **Nome utente** e la
  password HTTP come token. Viaggiano come autenticazione HTTP basic; GitPoulpi parla con la REST API `/a/` del
  server e ignora la riga `)]}'` con cui inizia ogni risposta. Il nome utente non è segreto e viene conservato in
  `config.toml` (come `account`), la password va nel credential helper. Un repository si collega a un *progetto*
  Gerrit sotto PULL REQUEST (lì sono elencate le sue modifiche); Gerrit non ha issue, e una modifica si crea
  facendo un push (vedi *Push per la revisione* in [Rivedere](review.md#gerrit)).
- **Redmine:** l'**URL del sito** del tuo Redmine (con il suo sottopercorso se ne ha uno,
  ad esempio `host/redmine`) e la tua chiave API, inviata come `X-Redmine-API-Key`. Se
  Redmine risponde 403 o con una pagina HTML, il servizio web REST è spento (chiedi
  all'amministratore, oppure attivalo in **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack e Redmine sono solo issue tracker: niente pull request, niente repository da
clonare e niente chiavi SSH. Bitbucket non ha issue (Atlassian ha dismesso
Bitbucket Issues). Collega invece la sezione ISSUE di un repository Bitbucket a
un progetto Jira.

## 3. Dove viene conservato il token

Il token è conservato nel **credential helper** del tuo sistema (Git Credential
Manager su Windows, libsecret o `pass` su Linux, Keychain su macOS). Usa una voce
privata che non può essere confusa con le credenziali di un host vero. Non viene
mai scritto in `config.toml` e mai stampato.

Se non è configurato nessun credential helper, o se rifiuta il token, GitPoulpi
tiene il token **in memoria solo per questa sessione** e te lo dice. Riconnettiti
dopo un riavvio.

GitPoulpi cerca il token di un profilo in quest'ordine:

1. La variabile d'ambiente `GITPOULPI_TOKEN_<PROFILE ID>`. Usala per fornire un
   token senza salvarlo, ad esempio in CI.
2. Il token salvato per il profilo (`token_ref`).
3. Solo per i profili GitHub e GitLab: il credential helper del sistema per
   l'host, quando nessun altro profilo abilitato usa lo stesso host.

Ogni profilo è una voce `[[integrations]]` in `config.toml`. Vedi il
[riferimento della configurazione](config-reference.md#integrations-hosting-e-issue-tracker).

## Il controllo di sicurezza

Un token salvato è sicuro quanto l'indirizzo a cui viene inviato. GitPoulpi quindi ricorda, per ogni integrazione, a cosa era collegata, e si ferma quando cambia:

- **L'indirizzo.** La prima volta che un'integrazione viene usata (o collegata), il suo indirizzo viene registrato in `trust.toml`, nella cartella dati privata riservata all'utente (`<data>/private/`), non in `config.toml`: modificare la configurazione non può mai ampliare ciò che è considerato attendibile. Se l'indirizzo nella configurazione non è più quello registrato, nulla gli viene inviato e il token non viene nemmeno letto. Le grafie di uno stesso indirizzo (`HTTPS://Git.Example.com:443/`) sono lo stesso indirizzo; un host che gli somiglia no.
- **L'account.** Dopo che hai collegato un token, e poi circa una volta al giorno (e una volta per esecuzione per un token che viene da `GITPOULPI_TOKEN_*` o dal credential helper), GitPoulpi chiede all'host a chi appartiene il token e confronta l'**id** dell'account, non il login. Un token che ora appartiene a qualcun altro viene fermato.
- **Il certificato.** I server il cui certificato non risale a un'autorità nota (una CA privata, un server autofirmato) vengono accettati solo per la chiave esatta che hai accettato, e l'handshake TLS dimostra comunque che il server possiede quella chiave. Con il pinning rigoroso attivo, la chiave deve corrispondere anche quando il certificato risale a una radice pubblica; il prezzo è che ogni rotazione della chiave chiede di nuovo, ed è per questo che è facoltativo. Gli host pubblici (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) non conservano alcun registro del certificato.
- **http in chiaro.** Un indirizzo `http://` non loopback viene rifiutato a meno che tu lo abbia consentito per quell'integrazione.

Ogni richiesta è inoltre vincolata all'indirizzo proprio dell'integrazione (e, per un host pubblico, al suo host API come `api.github.com`): una richiesta altrove viene rifiutata prima di essere inviata, e un reindirizzamento a un altro indirizzo non viene seguito.

**Le integrazioni collegate prima dell'introduzione di questo controllo** ricevono il loro registro in silenzio, dalla configurazione com'era, la prima volta che vengono usate dopo l'aggiornamento: mantengono il livello di fiducia di oggi e nulla chiede conferma.

### Da riga di comando

Un comando fermato fallisce con codice di uscita 2 e stampa cosa è cambiato (vedi il [riferimento della CLI](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` mostra ciò che è considerato attendibile; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` e `--allow-insecure-http on|off` accettano una cosa ciascuno, nominando il valore esatto. Dopo aver accettato un nuovo indirizzo, inserisci di nuovo il token. Disconnettere (`hosting logout`) o rimuovere un'integrazione elimina il suo token, il suo registro di fiducia e ciò che ne era stato messo in cache (gli elenchi di repository salvati della finestra Clona, gli elenchi di account della Radar e le bozze di revisione remote del suo host).

Un agente (MCP) non può accettare nulla: gli viene detto che il controllo di sicurezza richiede te.

### Nella TUI

Una richiesta fermata apre il dialogo **Controllo di sicurezza dell'integrazione** sopra qualsiasi cosa stessi facendo. Dice cosa è cambiato (il vecchio e il nuovo indirizzo, il vecchio e il nuovo account con i loro id, oppure il vecchio e il nuovo certificato come una piccola tabella con l'impronta SHA-256 in gruppi di quattro, soggetto, emittente e validità) e che il token non è stato inviato. Le risposte sono numerate:

1. **Disconnetti** elimina il token salvato, il registro di fiducia e ciò che era stato messo in cache dall'integrazione.
2. **Considera attendibile** il nuovo indirizzo, account o certificato (per un indirizzo non cifrato: **Consenti http in chiaro**).
3. **Annulla** non cambia nulla.

Nessuna risposta è evidenziata finché non premi una freccia o una cifra, quindi un Invio per errore non accetta nulla. Esc e la croce di chiusura annullano; un clic fuori dal dialogo non fa nulla. Un testo lungo scorre con la rotellina, PgSu/PgGiù o la barra di scorrimento. Più controlli si accodano (uno per integrazione e tipo di modifica) e vengono posti uno dopo l'altro. Un controllo annullato non riappare finché resta invariato; **Integrazioni: Rivedi il controllo di sicurezza** nella palette dei comandi lo riporta.

Dopo che hai considerato attendibile un nuovo **indirizzo**, il modulo dell'integrazione si apre con il campo del token attivo e il suggerimento «Si consiglia di inserire di nuovo il token»: il token salvato non è mai stato inviato lì. Nel modulo, un indirizzo cambiato di proposito viene accettato insieme a un token digitato per esso; senza un token il modulo ne chiede uno.

Il modulo ha anche due interruttori. **Pinning rigoroso del certificato** (per un indirizzo https) fissa la chiave del certificato che il server presenta quando ti connetti. **Consenti http in chiaro (non cifrato)** è offerto solo per un indirizzo `http://` che non è questa macchina, con un avviso rosso che il token viaggia in chiaro. La scheda collegata mostra l'impronta fissata.

### Cosa può fare il token, e fino a quando

Collegare chiede anche all'host cosa può fare il token. GitHub (token classici), GitLab e Bitbucket (quando invia gli ambiti) riportano i loro ambiti: la scheda mostra una riga gialla «Questo token ha più permessi di quelli di cui GitPoulpi ha bisogno» che elenca gli ambiti oltre a quelli che GitPoulpi usa, quelli sensibili (come `delete_repo`, `admin:org`, `sudo`) in rosso, con il consiglio di preferire un token a grana fine con una data di scadenza. Un token GitHub classico (`ghp_...`) riceve quel consiglio anche quando i suoi ambiti sono esatti. Di un token GitHub a grana fine non si possono chiedere i permessi, quindi ne viene mostrata solo la scadenza. La scheda mostra la data di scadenza quando l'host la indica. Un token senza scadenza non mostra alcun promemoria, e gli altri prodotti non riportano ambiti e non sollevano avvisi. Gli stessi campi sono nel JSON di `hosting integrations login` e `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Vengono letti quando ti connetti, quindi la scheda li mostra per la sessione che si è connessa.

## 4. Collega un repository

Il collegamento è esplicito e per repository. Quando un profilo è connesso, la
barra laterale Refs mostra una sezione **PULL REQUEST** e una sezione **ISSUE**
dopo TAG. Ogni sezione ha due menu a discesa: il provider e il progetto.

- Il menu del provider **propone** gli account che corrispondono ai remoti del
  repository (prima `origin`), come righe tipo `owner/name · origin · GitHub`. Una
  sola scelta imposta sia il provider sia il progetto.
- Nulla viene collegato in automatico. Jira, Trello, Linear, YouTrack e Redmine non vengono mai proposti,
  perché nessun remoto li nomina: sceglili tu.
- **Nessuno (scollega questo repository)**, in cima al menu, scollega la sezione.
  Niente la ricollega da solo.
- Il collegamento è salvato nel file `.gitpoulpi/repo-state.toml` del
  repository.

Le due sezioni sono collegate **separatamente**. Ad esempio, le pull request
possono arrivare da Azure DevOps e le issue da Jira.

| Provider | Progetto PULL REQUEST | Progetto ISSUE |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | lo stesso |
| Azure DevOps | `Project/Repo` (l'organizzazione viene dal profilo) | `Project` (work item) |
| Gitea, Forgejo | `owner/name` | lo stesso |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | nessuno: collega Jira |
| Jira | nessuno | la chiave del progetto, ad esempio `ABC` |
| Trello | nessuno | una board |
| Linear | nessuno | una chiave di team, ad esempio `ENG` (i team sono elencati) |
| YouTrack | nessuno | il nome breve di un progetto, ad esempio `ABC` |
| Gerrit | il nome del progetto Gerrit, ad esempio `team/tool` | nessuno |
| Redmine | nessuno | un identificatore di progetto, ad esempio `bench` (l'ultima parte dell'indirizzo del progetto) |

L'elenco dei progetti ISSUE è ricercabile. Puoi anche digitare un progetto che
l'elenco non mostra.

**Query aggiuntiva.** Un'origine di issue ha una riga di filtro facoltativa sotto
il menu del progetto. Viene combinata (AND) con il progetto:

- Jira: un frammento **JQL** (`component = UI AND labels = backend`).
- Azure DevOps: una condizione **WIQL** (`[System.AssignedTo] = @Me`).
- YouTrack: una **query YouTrack** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Viene racchiusa tra parentesi, quindi un `or` non può allargare il progetto. Un
  tuo `sort by:` sostituisce il predefinito `sort by: updated desc`.
- Redmine: nessuna (bastano i filtri di stato e «assegnate a me» della sezione).
- Linear: nessuna (bastano i filtri di stato e «assegnate a me» della sezione).
- Trello: un menu in cui spunti le **liste** da mostrare. Nessuna spunta mostra
  tutte le liste aperte.
- GitHub e GitLab la ignorano.

`Enter` mantiene la query digitata, `Esc` la scarta. Palette:
*Issue: Modifica query / elenchi...*.

Dalla CLI: `gitpoulpi hosting link pulls --id <profile>` (oppure `issues`),
`--clear` per scollegare, `--filter` per la query aggiuntiva. Vedi il
[riferimento della CLI](cli-reference.md#comandi).

### Clonare da un account connesso

Un profilo connesso (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) è anche una sorgente nella finestra **Clona** (il pulsante **Clona** della scheda Home, o `c` in quella scheda). Ogni profilo compare sotto **Clona con URL** come sorgente a sé (`GitHub · name`, `GitLab · name`); GitHub e GitLab non vengono mai mescolati. Scegline uno e i suoi repository si caricano, raggruppati per proprietario: prima il tuo account, poi una sezione per ogni organizzazione o gruppo (`quarry/tools` per un sottogruppo GitLab), ciascuna sotto un'intestazione con il conteggio. Un lucchetto indica un repository privato.

![La finestra Clona su un account GitHub connesso: una sezione per proprietario, un lucchetto su un repository privato](../images/clone-hosted.png)

Il campo di ricerca filtra in tutte le sezioni (nome, percorso completo e descrizione); una sezione senza corrispondenze scompare. `Enter` su una riga compila il clone con l'URL HTTPS di quel repository e un nome di cartella.

![La finestra Clona con la ricerca che restringe l'elenco a un'organizzazione](../images/clone-hosted-search.png)

L'elenco di un profilo già aperto in precedenza compare subito da una copia salvata mentre l'host viene interrogato di nuovo (*aggiornamento...* accanto all'etichetta); quando arriva la risposta, i repository nuovi compaiono e quelli scomparsi spariscono, mentre il repository evidenziato e quello scelto restano dove sono. Se l'aggiornamento fallisce, l'elenco salvato resta con una nota, e il motivo è nel Registro attività.

Se l'host rifiuta il token, la finestra mostra *impossibile elencare i repository* con il motivo al posto dell'elenco (e la copia salvata viene eliminata); le altre sorgenti continuano a funzionare.

## 5. Pull request e issue nella barra laterale

### PULL REQUEST

La sezione ha un'intestazione con il conteggio e un `+` (crea una pull request),
i menu del provider e del progetto, una casella di ricerca e un menu di stato
(*Aperte*, *Merged / chiuse*, *Tutte*). Seguono quattro gruppi, ciascuno con il
suo conteggio:

- **Le mie pull request**: ne sei l'autore.
- **Assegnate a me**.
- **In attesa della mia revisione**: sei un revisore richiesto.
- **Tutte le pull request**: compresso all'inizio.

La casella di ricerca filtra ogni gruppo (titolo, numero, autore, branch). Ogni
cambio del menu di stato interroga di nuovo l'host.

Una riga inizia con un glifo di stato: `●` aperta (verde), `○` bozza, `◆` merged
(viola), `✕` chiusa (rosso). La richiesta del branch attivo è in grassetto. Le
righe si leggono `#12  titolo` (GitHub, Bitbucket) o `!53  titolo` (GitLab,
Azure DevOps).

### ISSUE

Il titolo della sezione nomina il provider: `ISSUE DI GITHUB`, `ISSUE DI JIRA` e
così via. Per Azure DevOps diventa `ELEMENTI DI LAVORO DI ...` e per Trello
`SCHEDE DI ...`. Sotto i menu: una casella di ricerca, il menu di stato
(*Aperte*, *Assegnate a me*, *Chiuse*, *Tutte*) e le issue. Una riga Jira si
legge `ABC-12  titolo`, con il tipo e lo stato attenuati quando la barra laterale
è abbastanza larga.

### Azioni su una riga

- `Enter` o un doppio clic apre la riga nel browser.
- Il clic destro, o `.`, apre il menu della riga:
  - **Apri nel browser**, **Copia l'URL**, **Copia il numero**.
  - Per una pull request: **Modifica su <provider>**, **Checkout del branch di
    origine**, **Vedi i commit del branch di origine**. Per una richiesta da un
    fork, il checkout si legge **Checkout della pull request (fork)**, e **Vedi**
    è disattivato.
  - Per una issue: **Crea un branch dalla issue**. Apre la richiesta del nuovo
    branch con un nome come `12-fix-the-thing` (o `ABC-123-fix-the-thing` per
    Jira).
  - Su GitHub, GitLab, Gitea e Forgejo, una riga di pull request ha anche **Rivedi...**
    (anche **Enter** sulla riga): la revisione nell'app, vedi
    [Rivedere una pull request](review.md).
  - Su GitHub, GitLab, Gitea e Forgejo, entrambi i tipi di riga hanno anche **Modifica assegnatari...**,
    **Modifica etichette...** e **Imposta milestone...** (e **Modifica revisori...** su
    una pull request): vedi
    [Assegnatari, revisori, etichette e milestone](#assegnatari-revisori-etichette-e-milestone).
- **Modifica su <provider>** ha altri tre accessi: la matita `✎` all'estremità
  destra di una riga di pull request sotto il puntatore o selezionata, `e` sulla
  riga selezionata, e il comando della palette *Integrazioni: Modifica la pull
  request collegata sull'host* (per la richiesta del branch attivo).
- Se il branch di origine non è ancora nel repository, GitPoulpi lo dice. Fai
  prima un fetch (`Ctrl+G f`).

**Checkout di una pull request.** GitPoulpi recupera la richiesta in un branch
locale `pr/<n>` (`mr/<n>` su GitLab) che la traccia, e passa a quel branch. Su
GitHub, GitLab, Gitea e Forgejo funziona anche per i fork (GitPoulpi recupera il riferimento head
speciale dell'host). Azure DevOps e Bitbucket recuperano il branch di origine
della richiesta. Lì una richiesta da un fork viene rifiutata: aggiungi il fork
come remoto e fai tu il checkout del suo branch. Annulla torna al branch
precedente. CLI: `gitpoulpi hosting checkout-pr <n>`.

**Rivedere una pull request dalla riga di comando (e un agente).** Su GitHub
(compreso Enterprise), GitLab (compreso self-managed), Gitea e Forgejo una richiesta si può
leggere e rivedere senza uscire da GitPoulpi: `hosting review show <n>` (file,
stato del merge, controlli), `hosting review threads <n>` (discussioni di revisione e
conversazione), `hosting review comment`, `hosting review reply`, `hosting review resolve`
e `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. Il diff viene dal tuo repository, non
dall'host: `hosting review fetch <n>` recupera l'head della richiesta in
`refs/gitpoulpi/pr/<n>` e il suo branch di destinazione in `refs/gitpoulpi/pr/<n>-base`
(nessuno dei due è un branch, non viene fatto alcun checkout). I numeri di riga sono quelli di quel
diff: `new` è la riga all'head, `old` la riga alla base del merge. GitLab
non ha lo stato di revisione «richiedi modifiche»: lì viene rifiutato, senza inviare nulla.
Gitea e Forgejo accettano un commento su una sola riga (nessun intervallo); una discussione è l'insieme dei
commenti su una riga di un file. Gitea può risolvere una discussione, Forgejo 11 no
(l'host non ha un endpoint per questo: la chiamata lo dice). Una risposta su un server
senza l'endpoint delle risposte viene pubblicata come revisione di un solo commento sulla stessa riga.
Gitea rifiuta di approvare o respingere una tua richiesta.
Senza clone: aggiungi `--project owner/name` (GitLab: `group/project`) a `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` e `suggest` per rivedere un progetto che non è il repository corrente,
`hosting review diff <n> --project P` legge il diff (hunk per file) dall'API dell'host, e
`hosting file <path> --project P [--ref R]` legge un file a una revisione (GitHub, GitLab, Gitea, Forgejo e Gerrit; su Gerrit il diff di un
patch set viene letto file per file e il file viene dal branch o, per un id di commit, dal commit). Un file che l'host
non mostra nel diff (troppo grande) è marcato `omitted`.
Gli strumenti MCP sono gli stessi comandi (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Assegnatari, revisori, etichette e milestone

Su GitHub (compreso Enterprise), GitLab (compreso self-managed), Gitea e
Forgejo puoi cambiare a chi è assegnata una pull request o una issue, chi la rivede, le sue etichette
e la sua milestone senza uscire da GitPoulpi. Gli altri provider non mostrano nulla di tutto ciò.

- **Menu della riga**: **Modifica assegnatari...**, **Modifica revisori...** (solo
  pull request), **Modifica etichette...**, **Imposta milestone...**.
- **Palette**: *Hosting: Modifica assegnatari...*, *Hosting: Modifica revisori...*,
  *Hosting: Modifica etichette...*, *Hosting: Imposta milestone...* elencano prima
  le richieste e le issue della barra laterale tra cui scegliere. Nel
  [Radar](radar.md) gli stessi quattro agiscono sulla riga selezionata.
- Il selettore si apre subito e carica dall'host le persone o le etichette del progetto (e ciò
  che l'elemento ha adesso). Una riga con indicatore di attività compare dopo circa 150 ms.
  Digita per filtrare (una corrispondenza fuzzy), `Su` / `Giù` si spostano, `Space` spunta o
  toglie la spunta, `Enter` applica, `Esc` annulla, `Tab` raggiunge i pulsanti. Un clic
  su una riga la spunta, e la rotellina scorre. Le persone mostrano un badge con le iniziali,
  le etichette il loro colore.
- La milestone è un menu a discesa (con **Nessuna milestone** per toglierla).
- Dopo **Applica** l'host viene aggiornato, e gli elenchi e il Radar mostrano la
  modifica al loro prossimo aggiornamento, che parte subito.

I moduli di creazione (una pull request, una issue) hanno le stesse scelte come riga di
pulsanti: `Assegnatari: nessuno`, `Revisori: nessuno`, `Etichette: nessuna`,
`Milestone: nessuna`. Su GitHub, GitLab, Gitea e Forgejo il pulsante Etichette sostituisce il campo delle
etichette digitate del modulo della issue. Se un host accetta la nuova pull request ma
rifiuta alcune delle sue etichette o persone, la barra di stato lo dice.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, e `--assignee`, `--reviewer`, `--label`,
`--milestone` su `hosting pr` e `hosting issue create`; vedi il
[riferimento della CLI](cli-reference.md).

### Tasti nelle sezioni

| Tasto | Azione |
|---|---|
| `Enter` / doppio clic | apre la riga nel browser |
| `.` / clic destro | menu della riga |
| `e` | modifica sull'host la pull request selezionata |
| `n` | nuova pull request (PULL REQUEST) o nuova issue (ISSUE) |
| `v` | apre il menu di stato |
| `r` | aggiorna |
| `Enter` / `Space` su un'intestazione | comprime o espande la sezione |

### Riconnessione e modalità offline

Quando il token manca, è rifiutato o non ha uno scope, la prima riga della
sezione si legge `Riconnetti (token rifiutato)`. Cliccala, o premi `Enter`, per
aprire il profilo in Impostazioni > Integrazioni.

Quando l'host non è raggiungibile, le ultime liste restano a schermo e
l'intestazione mostra la loro età (`· di 5m fa`). L'ultima risposta è anche
memorizzata su disco (`.gitpoulpi/hosting-cache.json`), così le liste compaiono
prima che risponda la rete.

## 5b. La bacheca delle issue

Il pulsante **Bacheca** dell'intestazione ISSUE (palette: **Issue: Apri la bacheca**) mostra le issue della sorgente collegata come bacheca
Kanban sopra la finestra. Le colonne dipendono dalla sorgente:

| Sorgente | Colonne | Spostare una card |
|---|---|---|
| Note Markdown locali | gli stati (`statuses` del profilo, poi ogni altro in uso) | riscrive lo `status` della nota |
| GitHub, GitLab, Gitea, Forgejo | **Aperte**, poi le etichette che hai scelto, poi **Chiuse** | scambia l'etichetta di stato; verso **Chiuse** chiude la issue, verso una colonna di nuovo la riapre |
| Trello | le liste aperte della board, da sinistra a destra | sposta la card nella lista |
| Redmine | gli stati delle issue | cambia lo stato della issue (valgono le regole di workflow di Redmine) |
| Jira (Cloud e Data Center) | gli stati del progetto, quelli da fare per primi, quelli completati per ultimi | esegue la transizione di workflow che porta alla colonna (`GET` poi `POST .../transitions`); senza una transizione simile la card torna indietro e la barra di stato dice `no transition from X to Y` |
| Azure DevOps (Services e Server) | gli stati dei tipi di work item del progetto, per categoria di stato | imposta `System.State`; un work item il cui tipo non ha lo stato della colonna torna indietro, con il motivo (`a Bug has no state Done`) |

Su un progetto GitHub o GitLab, **Colonne** (o `c`, o **Bacheca: Imposta le etichette delle colonne...**) chiede le etichette che sono le
colonne, in ordine, ad esempio `status::todo, status::doing, status::review`. Propone le etichette del progetto che
sembrano stati. La scelta viene salvata per il repository. Senza etichette la bacheca è Aperte / Chiuse. Spostare una card
tocca solo le etichette di stato che hai elencato: le sue altre etichette restano. Una bacheca dell'host elenca le issue aperte e le 60 chiuse
più di recente.

Una transizione Jira che chiede campi obbligatori (una risoluzione, ad esempio) viene rifiutata da Jira e la card torna indietro con il
messaggio di Jira: fai quella dal sito.

La stessa bacheca è disponibile per script e agenti: `gitpoulpi hosting board` stampa le colonne con le loro card e
`gitpoulpi hosting board move <card> <column>` ne sposta una (vedi [il riferimento CLI](cli-reference.md)).

Una card mostra il numero, il titolo, gli assegnatari come iniziali, le etichette e la milestone. Trascinala su un'altra colonna,
oppure `Shift+Left` / `Shift+Right`, oppure **Bacheca: Sposta la card in...** (`m`). Si sposta subito; dopo circa 150 ms un indicatore di attività
dice che l'host sta ancora lavorando, e se l'host rifiuta la card torna indietro e la barra di stato dice perché. `Enter` apre
la issue sull'host (la nota, per le issue locali); `e` modifica assegnatari, etichette e milestone; `n` avvia una nuova issue nella
colonna selezionata; `a`, `l`, `M` e `/` filtrano per assegnatario, etichetta, milestone e testo, `x` azzera i filtri; `r`
rilegge l'host. La bacheca si rilegge quando gli elenchi della barra laterale si aggiornano. Il Radar non ha una bacheca.

## 6. Crea una pull request

Apri il modulo con il `+` nell'intestazione PULL REQUEST, con `n` nella sezione,
o con il comando della palette *Pull request: Crea pull request...*. Anche il
menu di un branch ha **Avvia una pull request da <branch>...**.

Il modulo contiene:

- Una scheda per ogni profilo connesso che ha pull request (`Sinistra` / `Destra`
  sulla fascia delle schede).
- I combobox **Repo di origine / Branch** e **Repo di destinazione / Branch**. Le
  liste lunghe sono ricercabili. Puoi digitare un branch o un repository che le
  liste non mostrano.
- **Titolo**, precompilato con l'oggetto del commit in punta.
- **Descrizione**, su più righe (`Enter` aggiunge una nuova riga, `Tab` esce dal
  campo).
- I pulsanti **Assegnatari**, **Revisori**, **Etichette** e **Milestone** (GitHub,
  GitLab, Gitea e Forgejo), che aprono il selettore.
- **Invia come bozza**.
- Con l'[IA](ai.md) attiva: **Genera titolo e descrizione**.

Il modulo si apre sul profilo, sul repository e sui branch del branch attivo
(`feature -> main`). Un repository di origine diverso crea una richiesta da un
fork.

**Crea pull request** (o `Ctrl+S`) crea la richiesta sull'host. Il risultato si
apre nel browser e la barra di stato lo nomina.

## 7. Crea una issue

Quando la sezione ISSUE ha un provider **e** un progetto, la sua intestazione ha
un `+` (*Nuova issue*). Inoltre: `n` nella sezione, o il comando della palette
*Hosting: Nuova issue...*.

Il modulo ha **Titolo** (obbligatorio), **Descrizione** (su più righe) ed
**Etichette** (separate da virgole, ad esempio `bug, ui`; su GitHub, GitLab, Gitea e Forgejo i pulsanti
Etichette, Assegnatari e Milestone aprono invece il selettore). Jira e Azure DevOps
aggiungono un menu **Tipo**. Trello aggiunge un menu **Lista**. Linear e YouTrack
aggiungono un menu **Stato** (lo stato del workflow, o il valore `State`, in cui parte la issue).
Redmine aggiunge un menu **Tipo** con i tracker del progetto (Bug, Feature,
Support...); non ha etichette, quindi non c'è un pulsante Etichette per lui e un'etichetta
data da riga di comando viene rifiutata. Il modulo riempie
questi menu con i dati dell'host quando si apre.

**Crea issue** (o `Ctrl+S`) la crea sull'host. `Esc` o **Annulla** chiude il
modulo. Se l'host rifiuta (token rifiutato, scope mancante, progetto sconosciuto,
etichetta sconosciuta), il modulo mostra il motivo e conserva ciò che hai
digitato. In caso di successo la barra di stato nomina la nuova issue e la sezione
ISSUE viene aggiornata.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## 8. Crea sulla pagina web dell'host

Il modulo Nuova issue e il modulo Crea pull request hanno un pulsante **Crea su
<provider>**, in alto a destra. Il tasto è `Alt+W`. Apre nel browser il modulo web
dell'host, compilato con quanto hai digitato finora. La finestra resta aperta.
Non richiede token e non fa nessuna chiamata API, quindi funziona anche dopo un
errore dell'API.

- GitHub non ha un parametro URL per *bozza*, quindi la casella della bozza non
  viene riportata. GitLab riceve un prefisso `Draft: ` nel titolo.
- GitLab accetta le etichette solo per id, quindi vengono aggiunte come azione
  rapida `/label ~"x"` alla fine della descrizione.
- Una descrizione molto lunga viene tagliata in modo che l'URL resti sotto i 6000
  caratteri, con una nota `[...truncated: paste the rest]`.
- Bitbucket Data Center non ha un indirizzo per il modulo web, quindi non ha il
  pulsante **Crea su <provider>**.

CLI: `gitpoulpi hosting issue create ... --web` e `gitpoulpi hosting pr --web`
stampano lo stesso URL.

## 9. Il chip della barra degli strumenti

All'estremità della barra degli strumenti (a destra per impostazione predefinita), un chip mostra la richiesta del
branch attivo: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) o
`BB #12` (Bitbucket). Il suo colore indica lo stato: verde aperta, grigio bozza,
viola merged, rosso chiusa.

- Passaci sopra per il titolo, lo stato e `origine → destinazione`.
- Cliccalo, o premi `Ctrl+G g`, per aprire la richiesta nel browser.
- Clic destro per **Apri nel browser**, **Copia l'URL**, **Aggiorna** e
  **Cambia profilo**.

Un branch su cui è stato fatto il push senza una richiesta mostra un chip
attenuato `+ PR`. Cliccalo per avviare una pull request. Non c'è nessun chip
quando nessun profilo corrisponde, quando non è stato fatto il push del branch o
quando l'host non è stato raggiungibile. In una finestra stretta il chip è
nascosto per lasciare spazio ai pulsanti della barra degli strumenti.

## 10. Quando le liste si aggiornano

Le liste si aggiornano:

- quando apri un repository;
- quando cambia il branch attivo;
- dopo un push o un fetch;
- ogni *Aggiornamento di pull request e issue (minuti)*
  (`general.hosting_refresh_minutes`, predefinito 5, `0` disattiva il timer;
  Impostazioni > Integrazioni);
- quando chiudi Impostazioni;
- quando premi `r`, esegui *Integrazioni: Aggiorna pull request e issue*, o
  cambi un filtro.

Le richieste non bloccano mai la finestra.

Gli host limitano il numero di chiamate che puoi fare. GitPoulpi rimanda l'`ETag`
dell'ultima risposta, così una lista invariata costa poco (nulla su GitHub).
Quando un host segnala che restano poche chiamate, la barra di stato lo dice.
Quando non ne restano, GitPoulpi smette di interrogare quell'host fino al
ripristino del limite, e le liste restano come sono.

## 11. Terminologia

GitHub, Azure DevOps e Bitbucket dicono "pull request". GitLab dice "merge
request". *Terminologia delle richieste di revisione* (`ui.pr_terminology`, in
Impostazioni > Integrazioni) sceglie il termine per ogni etichetta, menu, comando
della palette e messaggio:

- **Auto (in base all'host)**: segue il provider collegato del repository ("Pull
  Request" finché nulla è collegato).
- **Pull request** o **Merge request**: sempre quel termine.

La modifica si applica subito. La CLI e l'MCP dicono sempre "pull/merge request".
Questa pagina dice "pull request".

## 12. Riferimenti nei messaggi di commit e nei nomi dei branch

Quando le issue o le pull request del repository sono collegate, i riferimenti
nel messaggio di commit dei dettagli del commit sono sottolineati. Un clic apre
la pagina collegata. Da tastiera: il comando della palette *Integrazioni: Apri il
riferimento del commit (#issue / !richiesta)...* (un menu quando il messaggio ha
più riferimenti). Un repository senza nulla di collegato non ha link.

Cosa conta come riferimento dipende dal provider della sezione ISSUE:

| ISSUE collegate a | In un messaggio | In un nome di branch |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` e `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, solo quando `ABC` è la chiave del progetto collegato. Testo come `UTF-8`, `SHA-256` o `ISO-8601` non è mai un link. | `ABC-123` ovunque nel nome (`feature/ABC-123-x`), in maiuscolo |
| Linear | `ENG-123`, solo quando `ENG` è la chiave del team collegato (o di un team nell'elenco dei team del profilo). `UTF-8` non è mai un link. | Il `branchName` di Linear quando crei il branch da una issue; `ENG-123` ovunque nel nome viene riletto |
| YouTrack | `ABC-123`, solo quando `ABC` è il nome breve del progetto collegato (o di uno nell'elenco dei progetti del profilo) | `ABC-123` ovunque nel nome |
| Redmine | `#123`, solo quando Redmine è la sorgente ISSUE collegata (una sorgente di pull request mantiene il proprio sigillo) | `123-fix-login` |
| Trello | un indirizzo `https://trello.com/c/<shortLink>`; `#12` solo quando una scheda in cache ha il numero 12 | `12-fix-login` |

| PULL REQUEST collegate a | In un messaggio |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Quando entrambe le liste corrispondono a `#N`, vince la issue. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

### Inserire un riferimento a una issue

Nel compositore di commit, il pulsante `#▾` all'estremità destra del Riepilogo
apre un elenco filtrabile delle issue collegate. La issue indicata dal branch
corrente è la prima, contrassegnata *(dal branch)*. `Enter` o un clic inserisce il
suo riferimento (`ABC-123`, `#12`, `AB#12`) al cursore. Da tastiera: `Alt+R` nel
compositore. Palette: *Commit: Inserisci riferimento a una issue...*. Senza un
elenco di issue collegato, il pulsante è disattivato e dice *collega prima un
elenco di issue nella barra laterale*. GitPoulpi non aggiunge mai un riferimento
da solo.

## 13. Chiavi SSH

### Chiave SSH per profilo

Un profilo connesso ha una sezione **Chiave SSH** sotto la sua scheda. Senza una
chiave, scegli una di queste:

- **Chiave predefinita**: il tuo ssh-agent o `~/.ssh`. Non cambia nulla.
- **Genera una nuova chiave**: un titolo facoltativo (predefinito
  `GitPoulpi <host> <data>`). Con **Aggiungi la chiave pubblica a <host>**
  spuntato, GitPoulpi la carica, se l'host non ce l'ha già. La chiave viene
  scritta senza passphrase in `~/.ssh/gitpoulpi_<profile id>_ed25519` e `.pub`.
  Non serve `ssh-keygen`. Un file esistente non viene mai sovrascritto.
- **Chiave personalizzata**: digita un percorso o **Sfoglia** per un file di
  chiave privata, poi premi **Usa questa chiave**.

Quando una chiave è associata, la sezione mostra `titolo (SHA256:...)` con:

- **Rimuovi**: scollega la chiave, e la elimina sull'host se l'ha aggiunta
  GitPoulpi.
- **Elimina**: lo stesso, ed elimina anche i file della chiave, dopo una conferma
  che elenca entrambi i percorsi.

### Chiave SSH per un repository

Per far usare a un repository la chiave di un profilo, GitPoulpi imposta
`core.sshCommand` a `ssh -i "<key>" -o IdentitiesOnly=yes` in quel repository.
Non viene mai fatto in automatico. Apri il menu della scheda (`Ctrl+T`, o clic
destro sulla scheda) e scegli **Chiave SSH...**, oppure esegui il comando della
palette *Repository: Chiave SSH per questo repository...*. Il menu mostra il pin
attuale, ogni profilo che ha una chiave privata e **Rimuovi il pin**.

CLI e MCP: `gitpoulpi ssh-command show | set --key <file> | clear`.

La chiave globale in Impostazioni > SSH e credenziali (*Percorso della chiave
privata*, o *Genera coppia di chiavi*) vale per ogni repository senza pin. Il pin
di un repository ha la precedenza.

## 13b. Protezione dei branch, per una riscrittura della cronologia

Quando riscrivi la cronologia ([il pannello](history-rewrite.md)), GitPoulpi chiede all'host
quali branch protegge, così che vengano esclusi dalla riscrittura per impostazione predefinita.
Lo chiede solo per un repository che hai **collegato** (la sezione delle richieste o delle issue
della barra laterale, oppure un profilo fissato), con il token di quel profilo, e un token che
non riesce a leggerla produce semplicemente «sconosciuto»: nulla fallisce. Cosa risponde ogni
provider e cosa serve al token:

| Host | Cosa viene letto | Cosa serve al token |
|---|---|---|
| GitHub (ed Enterprise Server) | il flag `protected` dei branch e i ruleset attivi dei branch (pattern `~DEFAULT_BRANCH`, `~ALL` e `refs/heads/...`) | un token classico con `repo` (`public_repo` per un repository pubblico), oppure un token granulare con Metadata (lettura). Un ruleset che il token non può leggere viene segnalato, le regole classiche contano comunque |
| GitLab (gitlab.com e self-managed) | `protected_branches` (nomi e pattern con `*`) | `read_api` e un ruolo che può vedere le impostazioni del repository |
| Azure DevOps (Services e Server) | le policy dei branch attive e bloccanti del repository (ambiti `Exact`, `Prefix` e `DefaultBranch`) | Code (lettura). Il permesso di «force push» è un'impostazione di sicurezza e non viene letto |
| Bitbucket Cloud | le restrizioni dei branch che bloccano un push (`push`, `force`, `delete`) | `repository:admin`. Una restrizione sul modello di branching non viene risolta |
| Bitbucket Data Center | i permessi dei branch che bloccano un force push (`fast-forward-only`, `pull-request-only`, `read-only`) | amministratore del repository |

La pagina delle impostazioni della protezione (e del branch predefinito, e dell'eliminazione del
repository) viene costruita dall'URL del remote del repository per GitHub, GitLab, Azure DevOps e
Bitbucket, anche quando il repository non è collegato, e viene mostrata con le tre strade quando
l'host rifiuta un push.

## 13c. Elenchi a livello di account nel Radar

Il Radar di norma osserva i repository che hai clonato. Un profilo può anche elencare ciò che ti riguarda
sull'intero host, clonato o no, con tre interruttori nella sua tabella `[integrations.radar]` (tutti
disattivati per impostazione predefinita; vedi il
[riferimento della configurazione](config-reference.md#integrationsradar-elenchi-a-livello-di-account)):

- **richieste di revisione**: richieste aperte che chiedono la tua revisione;
- **richieste aperte**: ogni richiesta aperta del tuo namespace e di ogni organizzazione o
  gruppo di cui sei membro, o dei namespace che indichi. Mai l'intero host;
- **issue assegnate**: issue aperte assegnate a te.

GitHub, GitLab (anche self-managed), Gitea, Forgejo, Azure DevOps (Services
e Server), Bitbucket Data Center e Gerrit rispondono. Come viene interrogato ciascuno:

| Host | Richieste | Issue |
|---|---|---|
| Azure DevOps | pull request attive dell'intera organizzazione (`searchCriteria.reviewerId` per quelle che aspettano te; una richiesta su cui hai già votato è esclusa); quando il server rifiuta la rotta a livello di organizzazione, i primi 20 progetti vengono interrogati uno per uno. I `namespaces` sono nomi di progetto | work item assegnati a te che non sono Done, Closed o Removed |
| Bitbucket Data Center | la tua dashboard: ruolo `REVIEWER` non ancora approvato per le richieste di revisione, `AUTHOR` e `PARTICIPANT` per le richieste aperte; i `namespaces` sono chiavi di progetto | nessuna (Data Center non ha un issue tracker; una nota lo dice) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; le richieste aperte sono `is:open projects:<prefix>` per namespace (un prefisso di percorso di progetto), oppure le tue modifiche e quelle che hai rivisto quando non ne è indicato nessuno; una modifica work-in-progress è una bozza | nessuna |

Bitbucket Cloud non ha righe a livello di account: non offre alcun endpoint che elenchi le
richieste tra repository, e interrogare ogni repository sarebbe troppe
chiamate. I suoi repository clonati mantengono i propri elenchi, e `gitpoulpi hosting
radar --id <bitbucket>` lo dice. Quando Gerrit ha più modifiche del limite che
restituisce, l'elenco è marcato come tagliato con una nota.

Le bozze sono escluse a meno che tu le mantenga. Ogni elenco ha un limite: 300 righe per tipo, con una
nota che dice quante altre ce n'erano, e su GitHub poche chiamate di ricerca per
giro, perché la sua API di ricerca ne consente solo 30 al minuto. Un namespace che il token
non può leggere diventa una nota e gli altri si caricano comunque; un token rifiutato o un
rate limit fa fallire solo quel profilo. Le risposte vengono riusate per cinque minuti
(`radar-account.json` nella cartella di configurazione).

In Impostazioni > Integrazioni, un profilo di host di codice connesso ha un blocco **Radar**
sotto la sua chiave SSH: una casella per ogni elenco (*Richieste in attesa della mia revisione*,
*Ogni richiesta aperta*, *Includi le bozze*, *Issue assegnate a me*, l'ultima
assente per gli host senza issue), un campo *Namespace* (separati da virgole; vuoto
significa il tuo e tutti quelli di cui fai parte) e la scelta *Richiesta di un
repository non clonato*: *Chiedi*, *Clonalo* o *Rivedi tramite l'API*.
Tutto viene salvato man mano che cambi. Gli host senza query a livello di account (Bitbucket
Cloud) mostrano invece una riga che lo dice. La palette dei comandi raggiunge il blocco
con *Impostazioni: Opzioni del Radar di un profilo...*, e la ricerca nelle Impostazioni trova ogni
controllo dalla sua etichetta.

Dalla riga di comando: `gitpoulpi hosting integrations radar <id> --review-requests on`
attiva un elenco, `gitpoulpi hosting radar --json` stampa gli elenchi (e
`--refresh` salta la cache). Gli agenti IA hanno gli stessi due comandi via MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 14. Dalla riga di comando

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
```

Vedi il [riferimento della CLI](cli-reference.md#comandi) per ogni comando. Un
token non viene mai accettato come argomento, e il server MCP non offre
`hosting login`, quindi un token non passa mai da un agente.

## Risoluzione dei problemi

| Sintomo | Soluzione |
|---|---|
| *Connetti* fallisce con un errore di autenticazione | Il token è sbagliato, scaduto o revocato. Creane uno nuovo. |
| Uno scope è elencato in rosso | Il token non lo ha. La connessione funziona, ma quella funzione no. Aggiungi lo scope sull'host, poi **Modifica** e incolla il nuovo token. |
| La barra laterale non ha la sezione PULL REQUEST | Nessun profilo abilitato è connesso. Connettine uno, poi collega il repository nel menu della sezione. |
| Una riga *Riconnetti* in una lista | Il token è andato perso (un riavvio senza credential helper) o è scaduto. Clicca la riga per aprire il profilo. |
| Un'istanza self-managed fallisce | Controlla che il **Dominio host** non abbia un percorso, e che il tuo computer si fidi del certificato dell'istanza. |

## Limiti noti

- **Pull request da fork:** Azure DevOps e Bitbucket non possono fare il checkout
  di una richiesta da un fork (vedi [Azioni su una riga](#azioni-su-una-riga)).
  Azure DevOps non può creare una pull request da un fork.
- **Trello** mostra al massimo le prime 1000 schede di una board. Usa il filtro
  delle liste per restringere. **Jira**, **Linear** e **YouTrack** elencano al massimo 500
  issue per aggiornamento. Usa un frammento JQL o una query YouTrack per restringere.
- **Linear** accetta un solo assegnatario per issue: se ne scegli più d'uno nel selettore,
  viene inviato il primo. La sua «milestone» è il **ciclo** del team (le milestone di
  progetto non vengono usate). Quella di **YouTrack** è il campo `Fix versions`; una
  versione non ha un numero, quindi il selettore numera quelle non rilasciate in base
  alla loro posizione nel progetto. Nessuno dei due tracker ha pull request.
- **Redmine** accetta un solo assegnatario per issue (il primo del selettore), non ha etichette,
  ed elenca al massimo 500 issue per aggiornamento. Gli assegnatari sono i membri del progetto
  (persone e gruppi) e si distinguono dal nome visualizzato. Le colonne della bacheca Kanban
  sono gli stati delle issue, e spostare una card cambia lo stato; valgono le regole di
  workflow predefinite di Redmine (un ruolo che non può fare quella transizione riceve il
  rifiuto e la card torna al suo posto). «Resolved» non è uno stato chiuso nei dati
  predefiniti di Redmine: la colonna Chiuse è formata dagli stati «Closed» e «Rejected».
- **Bitbucket Data Center** non ha link per il browser per nuove pull request,
  branch o commit. Le pagine delle pull request si aprono normalmente.
- **Azure DevOps** non mostra avatar degli autori. I filtri "mie" e "revisione
  richiesta" richiedono un endpoint che può rifiutare i token di accesso
  personali. In quel caso il profilo si connette senza un nome utente e questi due
  filtri non sono disponibili. Un token non valido riceve una pagina di accesso
  invece di un errore; GitPoulpi lo segnala come token rifiutato.
- **Gitea e Forgejo:** una bozza è una richiesta il cui titolo inizia con `WIP:`
  (il server non ha un flag di bozza alla creazione). Una richiesta elenca come revisori
  tutti quelli che l'hanno già rivista, e ritirare una richiesta di revisione rimuove
  solo quelle in sospeso. Le pull request dal branch locale di un fork fanno il checkout
  tramite `refs/pull/<n>/head`, come GitHub. Le menzioni si cercano con il filtro
  `mentioned_by` del server. Il server non mostra gli scope del token, quindi uno
  mancante si scopre solo quando una chiamata viene rifiutata.
- **La bacheca delle issue** sposta le card su ogni origine di issue. Su Jira uno spostamento esegue la transizione di workflow che
  porta alla colonna; una transizione che chiede dei campi (una resolution, per esempio) viene rifiutata da Jira e la card torna al
  suo posto con il suo messaggio. Su Azure DevOps un work item il cui tipo non ha lo stato di destinazione torna al suo posto con il motivo.
- **Bitbucket:** le password per le app non esistono più; usa un token API. I
  repository sono elencati workspace per workspace.
- **Jira Cloud:** funzionano solo i token API classici (niente token con scope,
  niente OAuth). Una nuova issue Jira riceve una descrizione in testo semplice.
- Creare una pull request, un work item, una issue o una scheda avviene
  sull'host e **non può essere annullato** da GitPoulpi. Il checkout di una
  richiesta si può annullare normalmente.

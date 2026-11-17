[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · **Italiano** · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Conversazioni

La vista di conversazione di una issue, un work item, una card o una pull / merge request. Richiede un account e un repository collegato: vedi [Integrazioni di hosting e issue tracker](hosting-integration.md); gli elenchi che la aprono sono in [Pull request e issue](pull-requests-and-issues.md); rivedere il codice è in [Rivedere una pull o merge request](review.md).

Una **conversazione** è tutto ciò che è stato detto su una issue, un work item, una card, una modifica o una pull / merge request: la
descrizione, i commenti, le revisioni, le discussioni sul codice e gli eventi (etichette, assegnazioni, cambi di stato,
push). GitPoulpi la legge dall'host, la mostra formattata e ti lascia rispondere senza uscire dalla finestra.

### Aprire una conversazione

| Da | Come |
|---|---|
| Una riga ISSUE | `Enter` o un doppio clic; **Apri conversazione** del menu della riga; palette *Hosting: Apri la conversazione di una issue...* (un elenco delle issue a schermo). **Apri nel browser** del menu della riga mantiene il browser. |
| Una riga PULL REQUEST su Azure DevOps o Bitbucket | `Enter` o **Apri conversazione**. Su GitHub, GitLab, Gitea, Forgejo e Gerrit la riga apre la [revisione](review.md), la cui pagina Panoramica è questa stessa conversazione (la cronologia, i tasti e la casella dei commenti qui sotto). |
| Una card della [bacheca delle issue](pull-requests-and-issues.md#la-bacheca-delle-issue) | `Enter` o un doppio clic. |
| Il [Radar](radar.md) | `i` su una riga di issue o di richiesta, **Apri conversazione** del menu della riga, o *Radar: Apri la conversazione della issue*. |
| Un riferimento in un'altra conversazione | Cliccalo (oppure `l`, poi sceglilo): `#7`, `!12`, `ABC-12` di una sorgente collegata si aprono nell'app e finiscono su una pila di Indietro. |
| Un agente | `gitpoulpi ui show issue <n> --view conversation` (anche `ui show pr`), vedi [il riferimento della CLI](cli-reference.md). |

**Repository non clonati.** Anche una riga del Radar di un repository di cui non hai un clone ne apre la conversazione:
la vista ha bisogno solo del profilo di integrazione e del progetto, quindi il Radar la ospita nella sua finestra, e `Esc`
ridà il Radar. Ciò che richiede un clone viene escluso: un riferimento a un commit si apre sull'host invece che nel Log, e
*L'IA può leggere il repository* è disattivato con il suo motivo (vedi [il capitolo sull'IA](ai.md#aiuto-alla-scrittura-su-un-host)). Una
richiesta di un repository clonato ha invece la sua revisione (`e`), e **Apri conversazione** del Radar dice che per essa non c'è
nessuna conversazione da leggere qui; una issue di un repository clonato la apre nella scheda di quel repository.

Una issue di un host che non ne ha da leggere (Bitbucket e Gerrit non hanno issue) apre il browser come prima, e una build
senza la funzionalità `conversation` fa lo stesso ovunque ([build personalizzate](custom-builds.md)).

### La vista

Una vista a finestra intera: l'intestazione (stato, chi l'ha aperta e quando, le etichette, il numero di commenti, **Mostra eventi**,
**Indietro**, **Aggiorna**, **Apri nel browser**), la riga **Pubblichi come**, la cronologia, e sotto la casella dei commenti.

![La conversazione della issue #7: la descrizione con un link a #9, due commenti, un evento di etichetta mostrato da Mostra eventi, e la casella Nuovo commento vuota con Scrivi, Anteprima e Affianca](../images/issue-conversation.png)

![La pagina Panoramica della merge request !58: la descrizione, una discussione sul codice non risolta e una risolta con i loro link a file e riga, i commenti, Rispondi e Cita sulla discussione selezionata, e la casella dei commenti](../images/pr-conversation.png)

- La cronologia si legge dal più vecchio al più recente e parte dalla finestra più recente (al massimo 300 voci). **Carica precedenti** in
  alto (`m`, palette *Conversazione: Carica le voci più vecchie*) legge la finestra precedente, con il numero ancora da arrivare quando
  l'host lo dice.
- Un commento mostra il suo autore, la data, un chip `modificato`, un chip `bot` per i bot, `interno` per le note private, e i
  pulsanti **Rispondi**, **Cita**, **Modifica**, **Elimina** e **Link** che la voce selezionata consente. Una discussione sul codice
  mostra `percorso:riga`, `risolta` o `non risolta`, `obsoleta`, e i suoi commenti. Una revisione mostra il suo esito.
- Gli eventi sono una riga attenuata ciascuno (*carol ha aggiunto l'etichetta lfs*). **Mostra eventi** (`v`) li nasconde; la scelta viene conservata
  per repository. Gli host senza cronologia degli eventi (YouTrack, work item di Azure DevOps, note locali) non hanno quella casella.
- Il testo è Markdown reso: titoli, elenchi annidati e di attività, citazioni, avvisi in stile GitHub, codice evidenziato,
  tabelle, link, immagini come `[image: alt]`. Il testo arrivato nel markup proprio dell'host viene prima convertito (vedi
  [Conversione del markup](#conversione-del-markup)). Un blocco di codice `mermaid` ha un pulsante **Apri diagramma** che mostra il
  sorgente del diagramma in un visualizzatore scorrevole (`Enter` lo copia); la palette ha *Conversazione: Apri il diagramma della voce*.
  Richiede una build con entrambe le funzionalità `conversation` e `mermaid`.
- Nella Panoramica della revisione il `percorso:riga` di una discussione sul codice è un link al codice (`G`, *Conversazione: Apri il codice
  della discussione*; vedi [Revisione](review.md)); in una conversazione aperta da sola (un'issue, una riga del Radar) non c'è
  nessuna pagina di codice a cui andare.
- La vista rilegge ogni 60 secondi e con `F5` (**Aggiorna**).

**Tasti.** Ciascuno ha un pulsante, una riga di menu o una riga di palette (*Conversazione: ...*), e `Ctrl+N` mette i badge ai pulsanti.

| Tasto | Azione |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Voce precedente / successiva |
| `PageUp`, `PageDown`, `Home`, `End`, la rotellina, la barra di scorrimento | Scorrono |
| `Tab`, `Shift+Tab` | Cronologia, casella dei commenti, poi i pulsanti dell'intestazione |
| `r` | Risponde: nella discussione della voce selezionata, o citando il suo testo dove l'host non ha discussioni |
| `Q` | Cita la voce selezionata in un nuovo commento |
| `c` | Nuovo commento |
| `e`, `d` | Modificano, eliminano un tuo commento (prima di un'eliminazione c'è una domanda) |
| `l`, `Enter` | I link della voce: `Enter` segue l'unico, o ne apre il menu |
| `y`, `o` | Copiano l'indirizzo della voce, la aprono nel browser |
| `n` | Prossima voce nuova |
| `v`, `m`, `F5` | Mostra o nasconde gli eventi, Carica precedenti, Aggiorna |
| `Alt+Left`, `Backspace` | Indietro lungo la pila di Indietro |
| `q`, `Esc` | Chiudono (chiedono prima se la casella contiene testo) |

### Scrivere

**Nuovo commento** (`c`, o `Tab` fino alla casella), **Rispondi** (`r`), **Cita** (`Q`) e **Modifica** (`e`) usano un solo compositore
Markdown: **Scrivi**, **Anteprima** e **Affianca** (`Alt+P` alterna Scrivi e Anteprima, `Alt+L` attiva Affianca e richiede una
finestra larga 100 colonne). `Ctrl+Enter` (anche `Ctrl+S` e `Alt+Enter`) o **Invia** pubblica, **Annulla** chiude la casella, e
`Alt+E` apre **Scrivi con l'IA** dove l'IA è attiva ([il capitolo sull'IA](ai.md#aiuto-alla-scrittura-su-un-host)). L'anteprima è
impaginata dallo stesso renderer della cronologia, con gli stessi riferimenti. Quando l'host non accetta Markdown il
piè di pagina nomina il markup in cui il testo viene inviato (*Inviato come Jira wiki markup*, *Inviato come Textile*).

![Una risposta a una issue scritta nella vista Affianca: la bozza Markdown a sinistra, la sua anteprima resa a destra, e Invia](../images/issue-reply-composer.png)

- Il commento viene aggiunto subito alla cronologia e prende il suo posto quando l'host risponde. Se l'host rifiuta, la
  voce sparisce e la casella si riapre con il tuo testo e il motivo.
- Un testo che non hai inviato viene conservato per elemento (`remote-conversations.toml` nella cartella di configurazione) e c'è la volta
  successiva che apri quella casella; inviarlo o scartarlo lo rimuove. Chiudere una casella che contiene testo chiede prima.
- **Rispondi** su una discussione (una discussione di revisione, una discussion di GitLab, un commento di Linear, una discussione di Bitbucket o Azure DevOps)
  pubblica in quella discussione. Dove l'host ha solo un elenco piatto, **Rispondi** avvia un nuovo commento con la voce selezionata
  citata (`> **autore**`), come **Cita**.
- **Nota interna** (casella, `Alt+I`): su GitLab e Redmine il commento è visibile solo ai membri del progetto. Viene
  mostrata solo sugli host che hanno tali note, e solo su un nuovo commento.
- Pubblicare è un'azione verso l'esterno: nulla viene registrato nel journal e **da GitPoulpi nulla si può annullare**. Senza una
  connessione la casella resta, **Invia** è disattivato con il motivo, e l'intestazione dice che viene mostrata la copia salvata.

**Chi può modificare o eliminare.** Solo i tuoi commenti hanno **Modifica** ed **Elimina**. Prima vale la parola dell'host
quando ne dà una (`viewerCanUpdate` e `viewerCanDelete` di GitHub, `can_edit` altrove), poi il tuo login confrontato con
l'autore del commento, senza distinguere maiuscole e minuscole; i commenti di un bot App devono anche provenire da un account bot. Finché il
login di un bot GitHub App non è noto, contano solo i commenti pubblicati da questa installazione (il repository conserva i loro id):
nulla viene dedotto da un nome. In una discussione **Modifica** ed **Elimina** agiscono sull'ultimo commento che è tuo. Un host
che non sa modificare o eliminare (vedi [la tabella](#cosa-non-sa-fare-ogni-host)) mostra i pulsanti disattivati con il motivo.

### Pubblicare come

La riga sotto l'intestazione dice come vengono pubblicati i tuoi messaggi, e la stessa riga è in ogni casella:

| Riga | Significato |
|---|---|
| *Pubblichi come ann (token personale)* | Il token del profilo: il tuo login (`GET /user`). |
| *Pubblichi come ci-bot (comando token del repository)* | Per l'API di hosting si usa il [comando del token](hosting-integration.md#8-agire-come-bot-di-una-github-app) del repository. |
| *Pubblichi come my-app[bot] (bot di una GitHub App)* | Un token di installazione. Il nome viene da **Pubblica come**, altrimenti dall'host. |
| *Pubblichi come il bot della GitHub App (nome ignoto: imposta Pubblica come nelle Impostazioni)* | Nessun nome configurato, nessuno ancora appreso. |
| *Pubblichi come ... (bot del progetto)* | Un token di accesso di progetto o di gruppo di GitLab. |
| *Scrivi come kai (nota locale)* | Una cartella di note Markdown: il `local.me` del profilo, altrimenti l'utente connesso. |
| *Nome di pubblicazione non ancora noto. Imposta Pubblica come nelle Impostazioni.* | Non si è potuto apprendere nulla. |

**Pubblica come** è un nome: per una GitHub App è `my-app[bot]`. Impostalo per tutti i repository in **Impostazioni > Integrazioni >
Pubblica come** (`general.hosting_post_as`), o per un repository in **Impostazioni > Repository > SSH e credenziali > Hosting**,
che lo sostituisce. Se vuoto, GitPoulpi lo chiede a GitHub (GraphQL `viewer { login }`, a cui un token di installazione può
rispondere o no) e, in mancanza, apprende il nome dall'autore del primo commento che pubblica e lo conserva con
il repository. Un token personale non ne ha mai bisogno. Vedi
[Agire come bot di una GitHub App](hosting-integration.md#8-agire-come-bot-di-una-github-app).

### Riferimenti incrociati

I riferimenti in una conversazione sono sottolineati e cliccabili. Le sorgenti ISSUE e PULL REQUEST collegate al repository
vengono confrontate per prime, quindi `ABC-12` di un progetto Jira collegato si apre nell'app anche dentro un commento di GitHub; ciò che la
grammatica propria dell'host aggiunge apre la sua pagina dopo la conferma del link. Un id di commit si apre nel Log quando il repository
è clonato e l'id ha almeno 7 caratteri, sull'host altrimenti. Nulla dentro uno span di codice, un blocco delimitato, un
link Markdown, un indirizzo e-mail o un URL è un riferimento, e `C#1`, `UTF-8`, `SHA-256` e `ISO-8601` non lo sono.

| Host | Riconosciuti |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, id di commit |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (un progetto fratello), `@user`, `~label` e `%milestone` come chip, id di commit |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, id di commit |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, menzioni `{uuid}`, id di commit |
| Gerrit | valori `Change-Id` (`I` e 40 cifre esadecimali), id di commit |
| Redmine | `#7`, revisioni `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, solo per un progetto o un team di una sorgente collegata |
| Trello | indirizzi di card (`trello.com/c/...`) |
| Note locali | `[[nota]]`, `[[nota\|alias]]` e link `.md` relativi a un'altra nota della cartella |

`gitpoulpi hosting markdown --body T --id I` stampa cosa fa l'app di un testo, riferimenti compresi.

### Conversione del markup

Gli host che non parlano Markdown vengono convertiti nei due sensi, così leggi e scrivi Markdown ovunque. Ciò che un
convertitore non capisce resta testo leggibile.

- **Jira** (Cloud e Data Center) conserva i commenti come wiki markup tramite l'API v2. Ciò che leggi viene convertito in
  Markdown; ciò che scrivi viene convertito in wiki markup, e il piè di pagina dice *Inviato come Jira wiki markup*.
- **Redmine** formatta il testo come Textile o Markdown, cosa che l'API non dice: la riga **Formato del testo del
  server** del profilo (Impostazioni > Integrazioni, `markup`, Textile per impostazione predefinita) lo dice a GitPoulpi. Il Textile viene convertito in
  Markdown per la lettura e di nuovo in Textile per la scrittura.
- **Azure DevOps Server** restituisce i commenti dei work item come HTML e li accetta come HTML: vengono mostrati come il Markdown
  convertito da esso e scritti come HTML. Azure DevOps Services e le discussioni delle pull request accettano Markdown.
- **Gerrit** è testo semplice: non viene reso come Markdown e ciò che scrivi viene inviato come l'hai digitato.
- Ogni importatore rimuove prima le sequenze di controllo e di escape, così un commento non può muovere il terminale.

### La copia salvata, Carica precedenti, non letti

- Ogni conversazione che apri viene salvata (`<cartella di stato>/conversations/<integrazione>/<progetto>/<tipo>-<n>.json`, riservata all'utente,
  senza token, al massimo 2 MB per file, vengono conservati i 50 file più recenti). Viene mostrata subito quando l'elemento si apre e viene
  aggiornata dietro di essa. Quando l'host non è raggiungibile la copia salvata resta a schermo sotto *Offline: mostro la
  copia salvata 3 min fa (motivo)* e Invia è disattivato.
- **Non letti:** ciò che hai visto per ultimo viene ricordato per conversazione (vengono conservate le ultime 200). Le voci di qualcun altro
  più recenti ricevono una barra di accento e un chip `nuovo`, e `n` salta alla successiva. Una conversazione che apri per la
  prima volta non ha nulla di nuovo.

### Cosa non sa fare ogni host

Ogni `no` qui sotto è un limite reale: il controllo è nascosto o disattivato con il motivo, e la CLI risponde con un errore che nomina
l'host prima che venga inviato qualcosa.

| Host | Elementi | Risposta in una discussione | Modifica ed elimina i propri | Risolve una discussione | Eventi | Note interne | Scritto come |
|---|---|---|---|---|---|---|---|
| GitHub | issue, pull request | pull request | entrambe | pull request | sì | no | Markdown di GitHub |
| GitLab | issue, merge request | sì (discussion) | entrambe | merge request | sì | sì | Markdown di GitLab |
| Gitea | issue, pull request | pull request | entrambe | pull request | sì | no | Markdown |
| Forgejo | issue, pull request | pull request | entrambe | no (l'host non ha un endpoint) | sì | no | Markdown |
| Azure DevOps | work item, pull request | pull request | entrambe | no (stato mostrato, non cambiato) | solo pull request | no | Markdown (HTML sui work item di Server) |
| Bitbucket (Cloud, Data Center) | solo pull request | sì | entrambe | no (mostrato, non cambiato) | sì | no | Markdown |
| Gerrit | solo change | sì | no (nessuna delle due si può modificare o eliminare) | sì | sì | no | testo semplice |
| Jira | issue | no (elenco piatto) | entrambe | no | sì (changelog) | no (serve un ruolo di progetto) | wiki markup |
| Linear | issue | sì (risposte a un commento) | entrambe | no | sì | no | Markdown |
| YouTrack | issue | no | entrambe | no | no | no | Markdown |
| Redmine | issue | no | solo modifica (Redmine 5 e successivi), mai eliminazione | no | sì (journal) | sì (note private) | Markdown o Textile |
| Trello | card | no | entrambe | no | sì (spostamenti) | no | Markdown |
| Note locali | note | no | solo quelli dell'autore locale | no | no | no | Markdown |

Inoltre: nessun host ti lascia avviare dalla conversazione una nuova discussione su una riga (fallo nella revisione), e le reazioni sono
mostrate solo su GitHub. Un testo più lungo di quanto l'host accetta viene rifiutato (GitHub 65.536 caratteri, GitLab 1.000.000, Azure
DevOps 150.000, Jira 32.767, Trello 16.384): la CLI controlla prima, nell'app viene mostrato il messaggio dell'host stesso e il tuo
testo viene conservato. Una nota locale conserva i suoi commenti in una sezione finale `## Comments` del file della nota stesso, una
voce `### autore - ora` ciascuno, così li vedono anche Obsidian e una shell.

Dalla riga di comando le stesse conversazioni sono `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit`
e `comment-delete`, e `hosting review conversation` (vedi [il riferimento della CLI](cli-reference.md)); per gli agenti sono
strumenti MCP con gli stessi nomi (vedi [MCP e agenti](mcp-and-agents.md)).

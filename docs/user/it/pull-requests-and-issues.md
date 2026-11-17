[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · **Italiano** · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull request e issue

Leggere, creare e gestire pull / merge request e issue dalla barra laterale, dalla bacheca e dalla barra degli strumenti. Prima collega un account e un repository: vedi [Integrazioni di hosting e issue tracker](hosting-integration.md). Tutto ciò che riguarda una singola issue o richiesta (commenti, revisioni, discussioni) è in [Conversazioni](conversations.md); leggere e rivedere una richiesta è in [Rivedere una pull o merge request](review.md).

## Pull request e issue nella barra laterale

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

![La sezione ISSUE della barra laterale con il menu del clic destro della issue #7 aperto, Apri conversazione per primo](../images/issues-sidebar.png)

### Azioni su una riga

- `Enter` o un doppio clic apre la riga: una pull request nella [revisione](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), una issue, e una richiesta su Azure DevOps o Bitbucket, come [conversazione](conversations.md)
  nella finestra, e tutto il resto nel browser. **Apri nel browser** del menu della riga apre sempre il browser.
- Il clic destro, o `.`, apre il menu della riga:
  - **Apri conversazione** (issue, e richieste che non hanno una vista di revisione), **Apri nel browser**, **Copia l'URL**,
    **Copia il numero**.
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
| `Enter` / doppio clic | apre la conversazione in GitPoulpi dove l'host ne ha una (una pull request su GitHub, GitLab, Gitea o Gerrit apre la revisione), altrimenti il browser |
| `o` | apre la riga nel browser |
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

## La bacheca delle issue

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
la [conversazione](conversations.md) della issue in GitPoulpi, e la bacheca ritorna quando la chiudi; `e` modifica assegnatari, etichette e milestone; `n` avvia una nuova issue nella
colonna selezionata; `a`, `l`, `M` e `/` filtrano per assegnatario, etichetta, milestone e testo, `x` azzera i filtri; `r`
rilegge l'host. La bacheca si rilegge quando gli elenchi della barra laterale si aggiornano. Il Radar non ha una bacheca.

## Crea una pull request

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

## Crea una issue

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

## Crea sulla pagina web dell'host

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

## Il chip della barra degli strumenti

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

## Quando le liste si aggiornano

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

## Terminologia

GitHub, Azure DevOps e Bitbucket dicono "pull request". GitLab dice "merge
request". *Terminologia delle richieste di revisione* (`ui.pr_terminology`, in
Impostazioni > Integrazioni) sceglie il termine per ogni etichetta, menu, comando
della palette e messaggio:

- **Auto (in base all'host)**: segue il provider collegato del repository ("Pull
  Request" finché nulla è collegato).
- **Pull request** o **Merge request**: sempre quel termine.

La modifica si applica subito. La CLI e l'MCP dicono sempre "pull/merge request".
Questa pagina dice "pull request".

## Riferimenti nei messaggi di commit e nei nomi dei branch

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

**Nelle conversazioni.** Il testo di una issue o di una richiesta viene letto con la grammatica del suo host, più ampia della
tabella qui sopra: `owner/repo#7`, `!7`, `GH-7`, `@login`, id di commit e le altre forme elencate in
[Riferimenti incrociati](conversations.md#riferimenti-incrociati). Le sorgenti collegate al repository vengono confrontate per prime, quindi una chiave Jira in un
commento di GitHub apre comunque la sua issue nell'app. `gitpoulpi hosting markdown --body TEXT --id I` mostra cosa trova l'app in
un testo.

### Inserire un riferimento a una issue

Nel compositore di commit, il pulsante `#▾` all'estremità destra del Riepilogo
apre un elenco filtrabile delle issue collegate. La issue indicata dal branch
corrente è la prima, contrassegnata *(dal branch)*. `Enter` o un clic inserisce il
suo riferimento (`ABC-123`, `#12`, `AB#12`) al cursore. Da tastiera: `Alt+R` nel
compositore. Palette: *Commit: Inserisci riferimento a una issue...*. Senza un
elenco di issue collegato, il pulsante è disattivato e dice *collega prima un
elenco di issue nella barra laterale*. GitPoulpi non aggiunge mai un riferimento
da solo.

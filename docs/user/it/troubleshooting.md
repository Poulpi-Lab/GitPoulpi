[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · **Italiano** · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Risoluzione dei problemi e FAQ

Parti dalle risposte rapide qui sotto; le sezioni più lunghe riguardano Git LFS e il
rendering del grafo. Se nulla di tutto ciò aiuta, apri una issue sul repository
GitHub con il tuo sistema operativo, il tuo terminale, `gitpoulpi --version` e
ciò che hai fatto.

## Quando qualcosa non funziona

Un'operazione fallita non mostra alcun pop-up né scheda con cornice: lo dice la **barra di stato**, in rosso, con il titolo e il motivo (*errore: Fetch non riuscito: impossibile raggiungere il remoto.*), e il messaggio resta finché un altro non lo sostituisce. Il pulsante Attività alla sua estremità destra conta gli errori che non hai ancora guardato. Clicca il messaggio rosso, o esegui `Registro attività: Apri l'ultimo errore` dalla palette, per aprire il Registro attività su quella voce. Ogni errore che hai provocato funziona così, non solo le operazioni git:

- **Operazioni git**: un fetch, un push, un pull, un checkout, un merge, un rebase, un commit, uno stash...
- **Una clonazione** (e la creazione di un nuovo repository): «Clonazione non riuscita».
- **Una chiamata a GitHub o GitLab**: gli elenchi di repository dei selettori e della finestra Clona, la creazione di una pull request o di una issue, la verifica di un token o di una chiave SSH nella pagina Integrazioni: «Richiesta al servizio di hosting non riuscita».
- **Un programma esterno**: l'editor, lo strumento di diff o di merge, il gestore file, il browser, il terminale o il launcher, un editor di immagini, i pulsanti *Prova* delle Impostazioni: «Strumento esterno non riuscito».
- **Una richiesta IA** (un messaggio di commit, una spiegazione, un suggerimento per un conflitto...): «Richiesta IA non riuscita».
- **Un file che non è stato possibile scrivere**: Salva dell'editor integrato, l'editor di merge, un'impostazione che non è stato possibile salvare: «Salvataggio non riuscito».

Il messaggio rosso sta sulla barra di stato qualunque vista sia davanti: una scheda di repository, Home, il Radar, le Aree di lavoro, le Impostazioni (sulla riga inferiore dello schermo), sotto il Registro attività o una finestra. Dove la barra di stato è coperta (Home, Radar, Aree di lavoro, Impostazioni) il messaggio è scritto sulla sua riga inferiore finché non hai guardato l'errore. Ti segue da una scheda all'altra. Un'operazione ancora in corso quando cambi scheda non viene seguita: apri il Registro attività per vedere come è finita.

Gli elenchi delle sezioni PULL REQUESTS e ISSUE che non si riescono ad aggiornare mantengono invece una riga propria nella sezione (si aggiornano da soli, un messaggio ogni pochi secondi sarebbe solo rumore). Anche una finestra ancora aperta (Clona, Crea pull request...) mantiene il proprio messaggio di una riga, così puoi correggere il campo e riprovare.

La voce del Registro attività contiene tutto ciò che mostrava il pop-up:

- **Titolo e motivo.** «Fetch non riuscito», poi una frase che dice il perché in parole semplici, poi cosa fare. Riconosciuti: il remoto ha commit che non hai (*prima pull*), credenziali rifiutate (*controlla Impostazioni > Integrazioni o la tua chiave SSH*), un arresto per conflitti, modifiche locali che verrebbero sovrascritte (*fai commit, stash o scartale*), remoto non raggiungibile e un altro processo git che tiene un lock sul repository. Qualsiasi altra cosa mostra la prima riga di git stesso. Una clonazione o una chiamata all'host che fallisce allo stesso modo (credenziali rifiutate, host non raggiungibile) è formulata allo stesso modo; gli altri errori ricevono una frase su cosa è fallito («Non è stato possibile clonare il repository.», «Non è stato possibile eseguire il programma.») e i dettagli riportano ciò che hanno risposto il sistema o l'host.
- **I dettagli**, mostrati quando la voce è aperta (`Enter`, o il clic che l'ha aperta): l'operazione, il comando git eseguito, il suo codice di uscita e le ultime righe di ciò che ha stampato (per la libreria git integrata, la sua classe e il suo codice di errore). Credenziali, token e intestazioni `Authorization` vengono oscurati prima che qualcosa sia mostrato, registrato o copiato.
- **Copia** (il pulsante, o `c`) mette negli appunti un blocco di testo fatto per essere incollato in una chat con qualcuno, o con un'IA, che possa spiegarlo: la versione di GitPoulpi, il sistema, il terminale, il backend, l'operazione fallita con i suoi dettagli e la versione di git.
- **Un pulsante per il passo successivo**, quando c'è, accanto a Copia (o `r`): *Prima Pull* (un push che il remoto rifiuterebbe), *Stash e riprova* (modifiche locali che ostacolano un checkout, merge, rebase o pull), *Riprova* (il remoto non era raggiungibile), *Apri Integrazioni* (credenziali). Esegue il passo e chiude la finestra. Per un errore di un repository diverso dalla scheda in primo piano viene proposto solo *Apri Integrazioni*: un nuovo tentativo girerebbe nel repository sbagliato.
- Senza il mouse, la palette dei comandi ha `Registro attività: Apri l'ultimo errore`, `Registro attività: Copia l'ultimo errore` e `Registro attività: Esegui l'azione suggerita` (il pulsante del rimedio della voce, per l'errore che mostra la barra di stato).

Un push che il remoto ha rifiutato perché è andato avanti mantiene la sua finestra (pull prima, force push con lease, annulla); i fatti restano comunque nel Registro attività.

### Il Registro attività

Il pulsante **Attività** all'estremità destra della barra di stato (un numero su di esso conta gli errori che non hai ancora guardato), `Ctrl+N` poi `Shift+A`, e `Vista: Registro attività` della palette aprono la finestra. `Ctrl+N` mostra un badge su ogni scheda, casella, campo, casella di controllo e pulsante della finestra (e sulla prima voce dell'elenco); premere il carattere fa ciò che farebbe un clic su di esso. Due schede, **Operazioni git** e **Applicazione**, che si cambiano con `Tab`, con un clic, con `g` (Operazioni git) o con `p` (Applicazione), e una casella **Ambito** sotto di esse che vale per entrambe (vedi sotto):

- **Applicazione**: le righe di registro del programma stesso, quelle della finestra e del daemon in background, le più recenti per prime. La casella **Livello** (un menu a tendina: Errore, Avviso, Info, Debug, Trace; `l` lo apre) dice quanto mostrare; chiedere più dettaglio di quello registrato alza anche il livello di registrazione (e lo salva come `log.level`). Sotto, **Mostra** ha una casella per livello (Errore, Avviso, Info, Debug, Trace, ciascuna con quante righe contiene) per nascondere o mostrare le righe di quel livello, per esempio per vedere solo gli errori, e **Tutti** / **Nessuno** spuntano o tolgono la spunta a ogni casella. I tasti da `1` a `5` attivano o disattivano i livelli in quell'ordine, `a` è Tutti, `n` è Nessuno; la palette ha `Registro attività: mostra o nascondi il livello ...`, `Registro attività: mostra tutti i livelli` e `Registro attività: non mostrare nessun livello`. I livelli oltre la scelta della casella Livello sono in grigio (per loro non viene registrato nulla; passa il puntatore su uno per vedere perché). Le spunte vengono ricordate finché non esci, e solo la scheda Applicazione le ha. **Filtro** (`/`) tiene le righe che contengono il tuo testo. **Copia tutto (filtrato)** copia ciò che è mostrato (spunte e filtro si applicano entrambi); **Apri la cartella dei registri** apre la cartella nel tuo gestore file. Una riga lunga non viene mai tagliata: vedi «Righe lunghe» più sotto.
![Il Registro attività, scheda Applicazione: la casella Livello, le caselle Mostra per livello con Tutti e Nessuno, il filtro e le righe di registro](../images/activity-log.png)

**Svuota il registro** (`x`, o il comando della palette `Registro attività: Svuota il registro dell'applicazione`) è nella scheda **Applicazione**: chiede prima conferma, poi svuota i file di registro di tutti i giorni conservati e l'elenco a schermo. Non si può annullare e non compare nella cronologia di annullamento.

- **Operazioni git** (la scheda su cui si apre la finestra): le operazioni, le più recenti per prime, ciascuna con l'ora in cui è stata eseguita e la durata (`Push - 123 ms`): apertura del repository, caricamento del grafo dei commit, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Qui sono elencati anche i fallimenti, i messaggi di stato, un clone fallito e il Fetch all di un'area di lavoro; `Enter` o un clic ne apre uno (il suo motivo, il passo successivo, il comando, il codice di uscita e l'output), `c` o il suo pulsante Copia lo copia, `r` o il suo pulsante esegue il passo suggerito. `Tab` (o `p`) passa alla scheda Applicazione; `Sinistra`/`Destra` scorrono di lato le righe lunghe.
![Il Registro attività, scheda Operazioni git: ogni operazione con l'ora e la durata, un fetch fallito in rosso](../images/activity-log-repository.png)

- **Righe lunghe** (entrambe le schede): un'intestazione o una riga di dettaglio più larga della finestra non viene mai tagliata. Sotto l'elenco compare una barra di scorrimento orizzontale; scorri lateralmente con `Sinistra` / `Destra` (riassegnabili come `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + la rotella o una rotella orizzontale, `Registro attività: Scorri a sinistra` / `Registro attività: Scorri a destra` della palette, o la barra (trascina il cursore, o premi la guida per portare il cursore sotto il puntatore). `Home` torna all'inizio, e cambiando scheda l'altra scheda riparte dal bordo sinistro. Il testo a capo (un riepilogo, un'azione suggerita) resta a capo.

- **Ambito** (un menu a tendina sotto le schede, `s` lo apre, oppure `Registro attività: Scegli l'ambito` della palette): di cosa parlano entrambe le schede. Le righe sono **Questo repository (nome)** (il repository della scheda da cui hai aperto il registro; non offerta dalla Home, che non ne ha), **Tutti i repository**, ogni **area di lavoro** e ogni repository che la finestra conosce. Da una scheda di repository il registro si apre su *Questo repository*, dalla Home (il pulsante Attività, `Ctrl+N` poi `Shift+A`) su *Tutti i repository*; `Ctrl+N` poi `Shift+G` e `Vista: Registro attività globale` della palette si aprono sempre su *Tutti i repository*, e lo stesso fa un clic sull'errore rosso di un fallimento il cui repository non è la scheda in primo piano (un clone fallito, per esempio). L'ultima scelta viene ricordata per la sessione, separatamente per i due modi di aprirlo. Nella scheda Applicazione una riga appartiene a un repository quando il programma l'ha scritta a suo riguardo (un push fallito, il tempo di un'operazione); sotto *Tutti i repository* ognuna di queste righe dice di chi è (`[nome]`). Una riga che non riguarda alcun repository (avvio, impostazioni, il daemon stesso) e una voce che non appartiene a nessuno (un salvataggio fallito) sono **a livello di app**: compaiono sotto ogni ambito, segnate `[a livello di app]` quando l'ambito è più stretto di *Tutti i repository*, così un filtro non nasconde mai un fallimento. La palette ha anche `Registro attività: Operazioni git` e `Registro attività: Registro dell'applicazione` per le schede.

- **Copia diagnostica** (un pulsante su entrambe le schede, `d`, sempre l'intero registro, qualunque sia l'ambito; e `Vista: Copia diagnostica` della palette) copia la versione, il sistema, il terminale e il backend, l'ultimo fallimento con i suoi dettagli e le ultime 200 righe di registro al livello scelto: il blocco da dare a un'IA quando qualcosa non va.

Le durate sono misurate dalla finestra (dalla richiesta alla risposta del daemon, andata e ritorno compresi), quindi ciò che fanno la CLI o MCP non viene cronometrato. `log.timings` (attivo per impostazione predefinita) disattiva le durate, `log.extended` (disattivo per impostazione predefinita) elenca anche il lavoro in background, l'aggiornamento periodico e l'auto-fetch; entrambi sono in Impostazioni > Generale accanto alle altre impostazioni del registro, e ogni operazione viene comunque scritta nel registro dell'applicazione (`info` per ciò che hai chiesto, `debug` per il background).

I file di registro sono `gitpoulpi-YYYY-MM-DD.log`, conservati per una settimana per impostazione predefinita e al massimo dieci anni (`log.retention_days`, da 1 a 3650), eliminati all'avvio di GitPoulpi e ruotati a `log.max_file_mb` (5 MiB). Dove si trovano: [riferimento della configurazione](config-reference.md#log-il-registro-dellapplicazione). Da uno script, l'errore `--json` di un comando fallito porta gli stessi fatti sotto `detail`; vedi il [riferimento della CLI](cli-reference.md).

Un clic sul messaggio rosso apre il registro su *Tutti i repository* con quella voce aperta. L'elenco delle Operazioni git è quello della sessione; la scheda Applicazione conserva il registro del programma tra un riavvio e l'altro.

**Attività in background.** Un fetch, push, pull, clone o trasferimento LFS, Fetch tutto / Pull tutto / Clona i mancanti di un'area di lavoro, una richiesta all'IA e una pull request o issue in invio sono attività: l'estremità destra della barra di stato mostra quella in corso con un `[x]` che la annulla, e un clic sull'etichetta (o `Ctrl+N` poi `Shift+J`, la voce `Attività: mostra le attività in background`) apre il pannello con un pulsante Annulla per attività (anche `c` o `Canc`). Un clone gira in background; una richiesta all'IA e un giro sull'area di lavoro sopravvivono al cambio di scheda. Un'attività oltre il punto in cui fermarla non è più sicuro è in grigio con il motivo; un'attività annullata lascia il repository com'era.

## Risposte rapide

**La barra degli strumenti e la barra laterale mostrano riquadri o punti
interrogativi al posto delle icone.** Usa un Nerd Font o un qualsiasi font con
copertura delle emoji nel tuo terminale. Il grafo e il testo funzionano anche
senza.

**Il grafo dei commit è fatto di caratteri, non di immagini.** È il ripiego
automatico: le immagini richiedono un terminale che parli il protocollo grafico di
kitty (kitty, Ghostty, herdr con i grafici kitty attivi) o il protocollo iTerm2
(WezTerm, iTerm2, mintty), oppure sixel (Windows Terminal, foot, Konsole). La console di Windows, tmux e zellij ottengono
caratteri. Vedi la sezione sul grafo più sotto per imporre l'uno o l'altro, e [Scegliere un terminale](terminals.md).

**Il mouse non fa nulla, o selezionare il testo con il mouse non funziona.**
GitPoulpi cattura il mouse perché i clic lo raggiungano. Per selezionare il testo
con il terminale stesso, tieni premuto `Shift` mentre trascini (nella maggior parte
dei terminali). Puoi anche selezionare dentro il diff (trascina, poi `y`). Per
disattivare la cattura del mouse, usa *Impostazioni > Tastiera e mouse > Mouse
attivo* (serve un riavvio). La tastiera raggiunge ogni comando.

**Una scorciatoia non funziona.** Il tuo terminale, tmux o uno script AutoHotkey
possono prendersi il tasto prima che GitPoulpi lo veda (`Ctrl+,` spesso arriva come
una semplice `,` nei terminali Linux e macOS; `Ctrl+Shift+N` sembra `Ctrl+N`). Usa
le alternative (`Ctrl+N` poi `,` apre le Impostazioni; `Ctrl+P` raggiunge ogni
comando), oppure riassegna la sequenza: vedi
[Impostazioni](settings.md#tastiera-e-mouse).

**Non ricordo una scorciatoia.** Premi `Ctrl+P` e scrivi ciò che vuoi; ogni
risultato mostra la sua scorciatoia. `?` elenca tutto.

**Un'opzione è grigia nelle Impostazioni.** Dipende da un'altra opzione. La riga
dice quale (per esempio, *Intensità della tinta di riga* richiede che *Sfondo delle
corsie del grafo* sia impostato su *Tinta di riga*).

**«Auto-stash in sospeso» sopra il Log.** GitPoulpi ha messo in stash le tue
modifiche per eseguire un'operazione e non è riuscito a rimetterle a posto. Usa i
pulsanti del banner **Pop ora**, **Mostra lo stash**, **Ricomincia** (annulla
l'operazione e rimette a posto le tue modifiche) o **Drop**; vedi
[Auto-stash](log-and-history.md#auto-stash).

**Un agente dice `no_viewer` o `deferred`.** `no_viewer`: nessuna finestra di
GitPoulpi è aperta per quel repository (oppure *Consenti agli agenti IA di pilotare
la vista* è disattivata). `deferred`: la finestra mostra un chip; `Ctrl+N Enter` lo
accetta. Vedi [MCP e
agenti](mcp-and-agents.md#mostrare-le-cose-allutente).

**Ho aperto GitPoulpi e vedo una finestra vuota o `errore del log: backend error`.**
Controlla di averlo avviato dentro un repository Git (altrimenti finisci sulla
scheda Home: usa **Sfoglia**, **Clona** o **Init**). Poi esegui `gitpoulpi daemon
status`; `gitpoulpi daemon restart` riavvia il processo in background per questo
repository. Premi `r` per aggiornare.

**Due finestre sullo stesso repository si comportano in modo strano.** Condividono
di proposito un solo daemon per repository, quindi una modifica in una compare
nell'altra. Se un daemon sembra bloccato, `gitpoulpi daemon stop --all` ferma il
daemon di ogni sessione; la finestra successiva ne avvia uno nuovo.

**Un commit, pull o push chiede una password, o non riesce ad autenticarsi.**
GitPoulpi usa per la rete il credential helper e l'ssh-agent del tuo `git`.
Assicurati prima che `git fetch` funzioni in un normale terminale. I token
GitHub/GitLab per le pull request sono separati e facoltativi: vedi [Hosting e
issue tracker](hosting-integration.md).

**Un fetch, pull o push non riesce con «L'helper delle credenziali configurato non è riuscito», «La chiave host SSH del remoto non è ancora nota» o un accesso rifiutato, e la sua voce del Registro attività ha un pulsante «SSH e credenziali...».**
Il fallimento dipende da come questo repository accede. Il pulsante apre
Impostazioni > SSH e credenziali, dove scegli un'altra fonte per questo
repository: gli helper propri di git con gli helper del repository ignorati, un
account di integrazione, un file di chiave (con un file known_hosts), oppure un
helper o un comando del token tuo; **Prova la connessione** poi controlla ogni
remoto. Per un helper che non è stato eseguito, `git config --show-origin
--get-all credential.helper` mostra dove è impostato.

**«GitPoulpi non ha inviato il token dell'account a ...».** Il remoto del
repository punta a un server che non è l'indirizzo a cui è legato l'account
scelto (spesso una riscrittura `url.<x>.insteadOf` nella config del repository
stesso), quindi il token è stato trattenuto. Correggi il remoto o scegli
un'altra fonte nella pagina.

**«Il comando del token di questo repository non è riuscito», «... è cambiato da quando lo hai confermato», «L'accesso di questo repository non può essere usato».**
Le scelte della pagina non possono essere eseguite: il comando del token non ha
stampato alcun token (le sue ultime righe di errore sono citate, senza i
segreti), uno script che esegue è cambiato da quando lo hai confermato (conferma
di nuovo il comando nella pagina), oppure la scelta indica un file di chiave o
un account che non esiste più. Ogni messaggio ha il pulsante **SSH e
credenziali...**.

**OpenSSH dice «bad permissions» per un file di chiave.** Altri account possono
leggere la chiave. In Impostazioni > SSH e credenziali la riga **File della
chiave privata** mostra un avviso con **Limita al mio account**.

**«stage di tutte le modifiche...» resta nella barra di stato.** Un aggiornamento
in background è ancora in corso; scompare quando l'elenco dei file si aggiorna. `r`
forza un aggiornamento.

**Un repository molto grande è lento.** Abbassa *Commit iniziali nel grafo* nella
pagina Impostazioni > Log e grafo (il Log ne carica altri mentre scorri se
*Caricamento progressivo* è attivo); nascondi la colonna *Modif.* del Log e
disattiva gli avatar; disattiva *Annulla attivo* se non ti serve annullare (nessuna
contabilità su stage e commit). `GITPOULPI_TIMING=1` scrive i tempi per richiesta in
`gitpoulpi-timing.log` nella cartella temporanea, utile da allegare a una
segnalazione di prestazioni.

**I pulsanti IA mancano o sono grigi.** L'IA è disattivata di base; vedi [Assistenza
IA](ai.md#risoluzione-dei-problemi).

**GitPoulpi si è chiuso da solo.** Ogni uscita viene registrata: cerca
`gitpoulpi-tui-exit.log` (e `gitpoulpi-tui-panic.txt` dopo un crash) nella cartella
di configurazione (`gitpoulpi config path` stampa il file delle impostazioni, che
sta lì), e allegalo a una issue.

**Dove sono le mie impostazioni?** `gitpoulpi config path`. Lo stato per repository
è in `.gitpoulpi/repo-state.toml` dentro il repository.

**GitPoulpi invia qualcosa da qualche parte?** Solo ciò che chiedi: operazioni git ai
tuoi remoti, richieste di hosting dopo che colleghi un account, richieste all'IA
dopo che premi un pulsante IA e, se attivi gli avatar, un hash delle e-mail degli
autori a Gravatar. Non c'è telemetria. Vedi [Assistenza IA](ai.md#privacy).

## Ogni scrittura fallisce: "Unable to create index.lock"

Un comando git morto (un crash, un terminale terminato) può lasciare
`.git/index.lock`, e da quel momento ogni modifica al repository fallisce. Quando il
file ha più di 10 secondi GitPoulpi mostra una striscia sopra il Log:
**Rimuovi index.lock** (palette: *Repository: Rimuovi index.lock obsoleto*; CLI:
`gitpoulpi index-lock remove`). Chiede prima ed elimina solo quel file. GitPoulpi non
può sapere con certezza se un altro programma git è ancora in esecuzione, quindi
assicurati che non ce ne siano (un IDE, un altro terminale) prima di confermare. In
un worktree collegato il blocco è quello del worktree stesso, in
`.git/worktrees/<name>`.

## "git am" si è fermato e Interrompi il rebase non fa nulla

Un `am` (applicazione di patch da una mailbox) non è un rebase. Un `git am` fermo ha
una propria striscia, **Applicazione delle patch (git am)**, con Risolvi i conflitti,
Continua, Salta e Interrompi (palette: *git am: Continua / Salta questa patch /
Interrompi*; CLI: `conflicts am-continue|am-skip|am-abort`). Interrompi chiede
prima, scarta le patch applicate finora e riporta il branch dov'era. Queste azioni
non sono annullabili, come i pulsanti del rebase.

## Annulla è grigio e dice "Impossibile annullare ..." (voce bloccata)

La voce più recente del journal è bloccata: il repository è andato avanti fuori da
GitPoulpi, oppure la voce è stata interrotta. Passa il mouse su Annulla per il
motivo; se è bloccata per sempre, il tooltip rimanda a **Modifica: Dimentica la
voce di annullamento bloccata** (palette) o a `gitpoulpi undo-forget`. Chiede prima e
toglie solo la voce dal journal; il repository non viene toccato. Vedi [Annulla e
ripeti](undo-redo.md).

## I file grandi appaiono come tre righe di testo ("I file LFS appaiono come puntatori")

Un file che dovrebbe essere un'immagine, un modello o un grosso binario contiene `version https://git-lfs.github.com/spec/v1` e una riga `oid sha256:...`: è un **puntatore** Git LFS, e il contenuto reale non è stato scaricato (oppure i filtri LFS non sono installati).

- Guarda il pannello Stato: un repository che usa LFS mentre `git-lfs` manca mostra un banner ambra. Installa il programma (https://git-lfs.com; Windows: incluso in Git for Windows oppure `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) e riapri o premi `r`.
- `gitpoulpi lfs status` dice se il programma è installato, se i filtri e l'hook sono configurati e quali pattern sono tracciati; `gitpoulpi lfs files` elenca i file puntatore e quali sono `local`.
- Con `git-lfs` installato, esegui **`LFS: Pull degli oggetti`** nella palette (o `gitpoulpi lfs pull`): scarica gli oggetti e sostituisce i puntatori. Prima installa i filtri LFS per questo repository se non ce ne sono.
- Se il download fallisce, il messaggio ne indica il motivo (credenziali per il remoto, un server che non ha più l'oggetto). GitPoulpi non ha bisogno di un account proprio: LFS usa le solite credenziali del tuo remoto (credential helper o ssh-agent).
- Il clone di un repository LFS non fallisce mai a causa di LFS; se il passo di download non ha potuto girare, il clone c'è, con i puntatori, e `LFS: Pull degli oggetti` lo completa più tardi. `gitpoulpi clone --no-lfs` salta il download di proposito.
- Puoi disattivare tutto ciò che riguarda LFS in Impostazioni > Git e codifica > Git LFS (o `[git] lfs_enabled = false`).

## "git-lfs not found" / "Git LFS non è installato"

Il messaggio `Git LFS non è installato. Questo repository conserva i file grandi con Git LFS; ...` (o un dialogo con un pulsante *Apri pagina di installazione*, o il `git-lfs: command not found` / `'lfs' is not a git command` di git stesso) significa che il repository è configurato per LFS - il suo `.gitattributes`, i filtri di `.git/config` o l'hook `pre-push` chiamano `git-lfs` - e il programma non è nel `PATH` visto da GitPoulpi. Installalo (link sopra). Su Windows il programma è incluso in Git for Windows; se `git lfs version` funziona in un terminale ma GitPoulpi dice ancora che manca, avvia GitPoulpi da un terminale che veda lo stesso `PATH`. Per nascondere LFS a GitPoulpi lasciando intatto il comportamento di git, disattiva `git.lfs_enabled`.

## Il grafo è disegnato con caratteri, non come immagine (herdr, tmux, altri terminali)

GitPoulpi disegna la colonna Grafo e gli avatar degli autori come pixel solo dove è sicuro che il terminale li mostrerà: immagini inline iTerm2 in WezTerm puro, iTerm2 e mintty; il protocollo grafico Kitty in kitty, Ghostty e **herdr quando la configurazione di herdr imposta `kitty_graphics = true`** (`[terminal]`, o `[experimental]` nei herdr più vecchi) e herdr gira dentro WezTerm, kitty o Ghostty. Le immagini sixel sono usate in Windows Terminal, foot, Konsole e mlterm. Ovunque altrove disegna caratteri, il che è sempre corretto.

- In herdr con la grafica Kitty attiva ma con una configurazione che GitPoulpi non trova (o che si affida al predefinito di herdr), imposta `ui.image_protocol = "kitty"` (Impostazioni > Log e grafo > Protocollo immagini) e riavvia.
- In un terminale che mostra riquadri o rumore dove dovrebbe stare il grafo, imposta `ui.image_protocol = "none"` o `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` disattiva le immagini a prescindere dalla configurazione.

**herdr remoto (`herdr --remote <host>`, oppure `ssh` e poi `herdr`): il grafo di testo è il predefinito.** Via SSH, GitPoulpi gira sull'host remoto e può leggere solo l'ambiente remoto e il `config.toml` di herdr. Se compare un'immagine Kitty lo decide il client herdr *locale* (la sua impostazione `kitty_graphics` e il suo terminale), che GitPoulpi non può vedere: quando il client non disegna le immagini, le celle restano vuote. GitPoulpi quindi disegna il grafo di testo ogni volta che rileva herdr via SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Se il tuo herdr locale ha `kitty_graphics = true` e un terminale che disegna immagini Kitty (kitty, Ghostty), attivalo con `ui.graph_renderer = "image"` (o `ui.image_protocol = "kitty"`) sul remoto. Le immagini vengono inviate inline (`t=d`), il che funziona su qualsiasi flusso di byte. Se la colonna è vuota, imposta `ui.graph_renderer = "text"`.

**Un rettangolo più chiaro dietro le corsie.** L'immagine è trasparente (PNG RGBA con alpha dritto) dove non c'è disegnato nulla, così lo sfondo del terminale e la fascia di selezione traspaiono. I terminali con protocollo Kitty e **WezTerm** compongono quell'alpha; GitPoulpi riserva allora normali celle vuote sotto l'immagine e la invia dopo di esse. Ogni altro terminale con protocollo iTerm2 (iTerm2 stesso, Rio, mintty) non è noto per fondere l'alpha dei PNG, quindi lì l'immagine porta uno sfondo opaco nel colore del tema, che può apparire come un rettangolo su un terminale il cui sfondo è diverso; `ui.graph_renderer = "text"` lo elimina. Con `ui.graph_lane_background = "off"` non viene dipinto nulla dietro le corsie sui percorsi trasparenti.

Vedi `ui.graph_renderer` e `ui.image_protocol` nel [riferimento della configurazione](config-reference.md).

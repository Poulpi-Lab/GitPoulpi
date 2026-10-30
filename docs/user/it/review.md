[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · **Italiano** · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Rivedere una pull o merge request

Rivedi una pull request (GitHub, Gitea, Forgejo), una merge request (GitLab) o una modifica Gerrit (vedi [Gerrit](#gerrit))
**senza uscire dalla finestra del repository**. La vista di revisione mostra la descrizione, la conversazione, i file
modificati con le discussioni sui commenti sotto le loro righe e i commit, ed è il posto in cui scrivi la revisione:
commenti, approvazione e richieste di modifiche. Richiede un profilo per uno di quegli host (vedi
[Integrazioni di hosting e issue tracker](hosting-integration.md)) e un repository collegato a esso; gli altri provider non
offrono revisioni. I limiti di un host vengono rispettati: GitLab non ha l'esito *Richiedi modifiche* e accetta una riga
per commento. Questa pagina le chiama tutte *richiesta*.

![La vista di revisione, pagina Panoramica: l'intestazione della richiesta con i suoi pulsanti dei metadati, la descrizione e la conversazione con un'approvazione inviata](../images/review-overview.png)

![La pagina File: i file modificati con le loro caselle «visto», il diff con una discussione sotto la sua riga, e una discussione risolta fuori dalle righe mostrate](../images/review-files.png)

## Aprire una revisione

Ogni accesso porta alla stessa vista:

- l'elenco **PULL REQUEST** della barra laterale: seleziona una riga e premi **Enter**, oppure fai clic destro su di essa
  e scegli **Rivedi...**;
- il **Radar**: il menu della riga, **Rivedi...**, o il tasto `e` (si apre il repository e con esso la revisione;
  per una richiesta di un repository che non hai clonato viene chiesto se clonarlo o rivederla tramite l'API,
  vedi [Rivedere senza un clone](#rivedere-senza-un-clone));
- la **ricerca nell'area di lavoro** (`Ctrl+Alt+F`): evidenzia una pull o merge request e premi **`Alt+R`**;
- la palette: **Hosting: Rivedi la richiesta...** elenca le richieste tra cui scegliere.

La vista riempie la finestra sopra il repository (è una modalità della scheda del repository, non una finestra separata).
**`Esc`** o la `✕` nel suo angolo la chiude. Mentre si apre, la richiesta viene letta dall'host e il suo codice viene
recuperato con git in background; compare un indicatore di attività se ci mette più di un momento.

## L'intestazione

Sempre visibile: il numero e il titolo, lo stato (Aperta, Bozza, Unita, Chiusa), i branch
(`main ← feature/moons`), l'autore, i controlli (`✓ controlli superati`, `✗ controlli falliti`, `● controlli in corso`),
se si unisce senza problemi o è in conflitto, la dimensione, e **`3 commenti in sospeso`** quando ne hai scritti alcuni.

I quattro pulsanti dei metadati - **Assegnatari**, **Revisori**, **Etichette**, **Milestone** - aprono gli stessi selettori
della barra laterale (vedi [Assegnatari, revisori, etichette e milestone](hosting-integration.md#assegnatari-revisori-etichette-e-milestone));
l'intestazione mostra il risultato appena l'host lo ha accettato. I pulsanti di azione: **Checkout** (il solito
checkout del branch della richiesta, che chiude anche la vista), **Apri nel browser**, **Aggiorna**
(**`F5`**) e **Termina revisione**.

## Tre pagine

Un controllo segmentato nel bordo superiore cambia pagina: **Panoramica**, **File**, **Commit**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, o un clic).

### Panoramica

La descrizione (il Markdown di base viene mostrato come tale: grassetto, corsivo, `codice`, link come il loro testo,
elenchi puntati e numerati, titoli, citazioni e blocchi di codice), poi la conversazione: i commenti che non sono sul codice
e le revisioni già inviate, dalla più vecchia, ciascuna con il suo esito (`approvata`, `modifiche richieste`, `commentata`,
`ignorata`). Su GitLab gli esiti sono le sue approvazioni. Sotto, la casella di risposta: cliccala o premi **`c`**
per scrivere un commento sulla conversazione. **Questo viene pubblicato subito**, e la casella lo dice.

### File

I file modificati a sinistra (lettera di stato, `+aggiunte -rimosse`, un conteggio `●n` delle discussioni aperte, un
conteggio `✎n` dei tuoi commenti in sospeso) e il diff del file selezionato a destra. Il diff viene da git: l'head della
richiesta viene recuperato in `refs/gitpoulpi/pr/<n>` (mai un branch, nessun checkout) e confrontato con la base del
merge, quindi i numeri di riga nel margine sono esattamente quelli che usano i commenti.

- **`[ ]` prima di un file**: segnalalo come *visto* (clic, o premi **`v`** / `Space` sull'elenco). Il titolo li conta.
  I segni di «visto» sono conservati per repository e sopravvivono a un riavvio.
- Le **discussioni** stanno subito sotto la loro riga: primo commento, risposte, e i pulsanti **Rispondi** e
  **Risolvi** / **Riapri**. Una discussione risolta si riduce a una riga; **`z`** (o `Enter` su di essa, o un clic sulla
  `▾`) comprime ed espande qualsiasi discussione.
- Le **discussioni obsolete**, e quelle su righe che il diff non mostra, sono elencate dopo gli hunk sotto
  *Commenti fuori dalle righe mostrate*, con un badge `obsoleta` quando il codice è cambiato sotto di esse.
- **`c`** commenta la riga su cui si trova il cursore (il piccolo `+` all'inizio della riga fa lo stesso con il
  mouse). **`Shift+Su` / `Shift+Giù`** (o un trascinamento sulle righe) selezionano prima più righe e il
  commento copre l'intervallo. GitLab accetta una riga per commento, e la vista lo dice invece di selezionare.
- **`r`** risponde alla discussione sotto il cursore, **`x`** la risolve o la riapre. Entrambi vengono inviati all'host
  **subito**.
- Tasti: `Su` / `Giù` / `PgUp` / `PgDn` / `Home` / `End` si spostano; `Tab`, `Destra` ed `Enter` vanno dall'elenco dei file
  al diff, `Sinistra` e `Tab` tornano indietro; `]` e `[` saltano al file successivo e precedente.

### Le opzioni di visualizzazione del diff

Il pannello del diff di **File** e **Commit** è quello proprio della [vista Diff](staging-and-committing.md), con la stessa barra sulla sua prima
riga e le stesse scelte ricordate (sono un'unica impostazione: ciò che scegli qui è ciò con cui si apre la vista Diff,
e viceversa):

- **Inline / Split**: una colonna, oppure prima e dopo affiancati. In Split una discussione, un commento in sospeso o un
  suggerimento sta sotto la metà a cui appartiene: un commento su una riga rimossa sotto la metà sinistra, un commento su una riga
  aggiunta o invariata sotto la metà destra. Clicca la metà che intendi prima di premere **`c`** su una riga che
  mostra una rimozione e un'aggiunta; la tastiera commenta sul lato nuovo.
- **Hunk**: solo le modifiche con il loro contesto, oppure l'intero file con le modifiche sul posto. Nel file intero una
  discussione su una riga fuori dalle modifiche sta sotto la sua riga invece che in *Commenti fuori dalle righe mostrate*.
  Il testo del file viene letto in background; gli hunk restano a schermo con una riga che lo dice finché non arriva (nella
  pagina Commit, fino a 40 file di un commit vengono letti così).
- **WS**: mostra o ignora le modifiche che differiscono solo per gli spazi.
- **A capo**: manda a capo le righe lunghe.
- **Frecce su / giù**: l'hunk precedente e successivo.
- **Mappa** (**`M`**): la minimappa accanto alla barra di scorrimento, un'unica impostazione con quella della vista Diff. Clicca o trascina su di
  essa per scorrere.
- **Auto** (**`E`**, o *Revisione: Scegli la codifica...*): la codifica con cui viene letto il file, come nella vista Diff:
  Rilevamento automatico o una delle codifiche dell'elenco, scelta per il file che il pannello mostra (nella pagina Commit, il
  file sotto il cursore). Entrambi i lati del file vengono riletti in quella codifica, in background, e
  la scelta è condivisa con la vista Diff per lo stesso percorso. Un file che non si può leggere così dice perché sopra
  il suo diff.
- **`⋯`**: quando il pannello è troppo stretto per ogni pulsante, quelli che non entrano stanno dietro un `⋯` in fondo alla
  barra; il suo elenco esegue ciascuno come se fosse cliccato. I loro tasti e le loro righe della palette funzionano come sempre.

Il righello della barra di scorrimento mostra dove sono le cose: le modifiche (solo nel file intero, come nella vista Diff), un
punto `●` per ogni discussione, commento in sospeso e suggerimento, e il cursore.

Le righe lunghe vengono tagliate al bordo del pannello (attiva **A capo** per vederle intere) e scorrono lateralmente: **`Destra`**
e **`Sinistra`** sul diff (`Sinistra` al bordo sinistro torna all'elenco dei file), la rotellina laterale o `Shift`
con la rotellina, la barra di scorrimento lungo il bordo inferiore, o *Revisione: Scorri il diff a sinistra / a destra*.

### Selezionare testo e righe

Il punto in cui inizia un trascinamento decide cosa seleziona. Un trascinamento che inizia **nel codice** seleziona testo, come nella vista Diff
(il doppio clic seleziona una parola); **`Ctrl+C`** (o *Revisione: Copia il testo selezionato o la riga*) copia esattamente quei
caratteri, senza numeri di riga né marcatori, e senza una selezione di testo copia la riga sotto il cursore. Un
trascinamento che inizia **nel margine** (i numeri di riga, il marcatore, il `+`) seleziona righe, per commentarle
come prima, e un doppio clic nel margine avvia ancora un commento su quella riga.

I tasti `n`, `p`, `m`, `w`, `s` della vista Diff sono propri della revisione, quindi nella revisione gli interruttori sono lettere
maiuscole: **`V`** layout, **`H`** hunk o file intero, **`S`** spazi, **`W`** a capo, **`J`** / **`K`** hunk successivo e
precedente, **`E`** la codifica, **`M`** la minimappa. Ognuno ha una riga della palette (*Revisione: Layout unificato o affiancato*,
...).

### Commit

Solo i commit della richiesta (non la cronologia che c'è dietro), dal più recente, e il diff di quello selezionato
(le sue modifiche rispetto al suo genitore), con tutti i suoi file uno sotto l'altro e le stesse opzioni di visualizzazione.

## Scrivere una revisione

Un commento sul codice è **in sospeso**: viene conservato sulla tua macchina (nel file di stato del repository, quindi sopravvive a un
riavvio) e contato nell'intestazione. Nulla viene inviato finché non termini la revisione. Un commento in sospeso compare
sotto la sua riga con il badge `In sospeso - riga 12` e due pulsanti: **`e`** (Modifica) e **`d`** (Elimina).

**Termina revisione** (**`f`**, il pulsante, o la palette) apre un piccolo modulo: un riepilogo (facoltativo) e un
elenco **Esito**: *Commenta*, *Approva*, *Richiedi modifiche*. Sono elencati solo gli esiti che l'host consente: GitLab non ha
*Richiedi modifiche*. **Invia la revisione** (`Ctrl+S`) invia il riepilogo, l'esito e ogni commento in sospeso in
un colpo solo. **Scarta la revisione...** butta via i commenti in sospeso dopo una conferma.

Ciò che viene inviato subito, senza aspettare la revisione: le risposte, il risolvere e il riaprire, e i commenti sulla
conversazione. La casella del commento dice di che tipo è (*Questo commento viene pubblicato subito* oppure *Questo commento viene aggiunto alla tua
revisione in sospeso*). Se l'host rifiuta una risposta, la casella si riapre con le tue parole.

Se la richiesta ha ricevuto nuovi commit dopo che hai scritto i commenti in sospeso, la vista avvisa che le loro righe potrebbero
essersi spostate (GitHub e GitLab collocano un commento sul commit che hai letto; una revisione di un head più vecchio viene rifiutata).

## Gerrit

Un server Gerrit si collega come gli altri host (vedi [Integrazioni di hosting e issue tracker](hosting-integration.md): URL del
server, nome utente, password HTTP) e si collega sotto **PULL REQUEST** a un *progetto*. Le sue modifiche sono elencate come pull
request (`#7`, chip `GR`) e rivedute nella stessa vista, con le differenze che Gerrit porta:

- **Patch set.** Una modifica ha patch set numerati (ogni caricamento del commit ne è uno). L'intestazione ha un
  elenco **`Patch set N`** sul bordo superiore, accanto alle schede delle pagine (**`P`**, o la riga della palette *Revisione: Scegli il
  patch set...*). Parte dall'ultimo; sceglierne un altro legge i file e il codice di quel patch set, e le
  discussioni scritte su altri patch set sono elencate come *obsolete*. La revisione che termini viene inviata sul patch set
  mostrato.
- **Confronta con un altro patch set.** Un secondo elenco a sinistra, **`Confronto: base`** (**`D`**, o *Revisione: Confronta i
  patch set...*), elenca la base e ogni patch set più vecchio di quello mostrato. Scegli il patch set 1 mentre è mostrato il
  patch set 3 e la pagina File elenca, e mostra come diff, solo ciò che il patch set 3 ha cambiato rispetto al patch set 1
  (la differenza del server stesso, quindi un rebase su una destinazione più recente non compare come modifica); la pagina
  Commit mostra quell'unica differenza. Entrambi i patch set vengono recuperati in `refs/gitpoulpi/pr/<n>` e
  `refs/gitpoulpi/pr/<n>-ps<k>`. Le discussioni scritte su patch set diversi da quello mostrato sono *obsolete*. Scegliere da
  leggere un patch set più vecchio di quello confrontato riporta il confronto alla base. Il gemello da CLI è
  `review show|fetch --compare-with K`.
- **Submit e abbandono.** Quando il server dice che la modifica si può sottoporre a submit (ci sono i voti e i controlli
  richiesti), un pulsante **Submit** si aggiunge ai pulsanti dell'intestazione; **Abbandona** c'è finché puoi abbandonare la
  modifica. Entrambi (tasti **`U`** e **`Q`**, palette *Revisione: Esegui il submit della richiesta...* / *Revisione:
  Abbandona la richiesta...*) chiedono prima, in una finestra che dice che l'azione avviene sul server e **non si può annullare
  da GitPoulpi** (una modifica unita resta unita; una abbandonata si può ripristinare nell'interfaccia web di Gerrit). Rifiutare
  non invia nulla; in caso di successo la vista rilegge la modifica. I gemelli da CLI sono
  `gitpoulpi hosting review submit-change <n>` e `hosting review abandon <n> [--message M]`.
- **Voti invece di un esito.** **Termina revisione** mostra un elenco per ogni etichetta su cui puoi votare - `Code-Review`
  (da -2 a +2) e `Verified` dove il progetto ce l'ha - che elenca solo i valori che il server consente **per questa
  modifica e per te** (uno sviluppatore senza il diritto non può scegliere +2). Parte dal tuo voto attuale; viene inviato solo un voto che hai
  cambiato. Il riepilogo e i commenti in sospeso vanno nella stessa chiamata. Le bozze che hai salvato nell'interfaccia web di Gerrit
  vengono pubblicate con la revisione.
- Le **discussioni** sono le catene di commenti di Gerrit. Una discussione è risolta quando il suo commento più recente dice `unresolved: false`;
  **risolvi** pubblica una risposta `Done` che lo fa, **riapri** una risposta `Reopened`. Le risposte e il risolvere vanno al server
  subito, come sugli altri host.
- **Push per la revisione.** Gerrit crea una modifica quando fai un push su `refs/for/<branch>`. Con la sezione PULL REQUEST
  collegata a un progetto Gerrit, il pulsante **`+`** della sezione e la riga della palette **Hosting: Push per la
  revisione...** chiedono il branch di destinazione e fanno il push di `HEAD` su `refs/for/<branch>` (un topic si può dare da riga di
  comando). Il push non si può annullare da GitPoulpi (abbandona la modifica in Gerrit); il messaggio lo dice. Il gemello da CLI
  è `gitpoulpi hosting review push`.
- I pulsanti dei metadati sono di sola lettura per Gerrit (i revisori vengono mostrati), non c'è un esito *Richiedi modifiche*
  (un voto di -1 o -2 è la stessa cosa) e il chip dei controlli è l'etichetta `Verified`.

## Bozza di revisione IA

![Due suggerimenti ✦ sotto le loro righe nella pagina File, ciascuno con Accetta, Modifica e Ignora, accanto a una discussione esistente](../images/review-ai.png)

Con l'IA attiva (*Impostazioni > IA*, vedi [IA](ai.md)) l'intestazione ha un pulsante **`✦ Bozza di revisione...`** (anche **`s`**,
e la riga della palette *Revisione: ✦ Bozza di revisione...*). Apre le [opzioni di revisione](#opzioni-di-revisione) e
**`Enter`** lì interroga il modello. È l'unica cosa che interroga il modello: aprire una revisione
non lo fa mai. Il modello legge il titolo e la descrizione della richiesta, i commenti che esistono già (così da
non ripeterli) e il diff, e risponde con una **bozza di revisione**: il testo da inviare, l'esito che raccomanda (Approva, Richiedi modifiche o Commenta), un riepilogo generale e un elenco di commenti suggeriti. Un
diff lungo viene tagliato per il modello - per file, e in totale - e la Panoramica dice quanti file sono stati accorciati
o esclusi. Mentre lavora la barra di stato mostra un indicatore di attività dopo un momento e **`Esc`** (o lo stesso pulsante)
lo annulla. Si applicano le istruzioni *Revisione del codice* dei [quattro livelli](ai.md#istruzioni), quindi un team o un'area di lavoro
può dire che aspetto ha una buona revisione.

### Opzioni di revisione

`s` apre la finestra **✦ Opzioni di revisione**, compilata con i valori predefiniti del repository (quelli globali dove il
repository non ne ha). `Enter` (o il pulsante **Avvia**) avvia l'esecuzione con ciò che la finestra mostra; `Esc`, **Annulla**
o la croce la chiude senza interrogare nulla.

| Controllo | Cosa fa |
|---|---|
| **Esegui l'agente nel repository** | *Impostazione del profilo* (decide la cartella di lavoro del profilo IA), *Sì* o *No*. Disattivato, con il motivo sotto, per un profilo HTTP (non ha una cartella di lavoro) e quando la richiesta viene rivista tramite l'API senza clone. |
| **CLAUDE.md**, **AGENTS.md** | Forniscono al modello quel file del repository, letto sul branch di destinazione della richiesta. Quando l'agente viene eseguito nel repository li legge da solo, e la finestra lo dice. |
| **File aggiuntivo** | Un file in più: un percorso nel repository, un percorso assoluto o un link a un file di un host di codice. **Sfoglia...** (`Ctrl+B`, o `Invio` sul pulsante) apre un selettore di file che parte dalla radice del repository (o accanto al file già indicato); un file dentro il repository viene scritto come percorso relativo alla radice, uno fuori come percorso assoluto. Scrivere nel campo funziona come prima. |
| **Nessun file in più** | Nessun file in più per questa esecuzione, anche se l'impostazione globale o il repository ne indica uno; scrivere un file o sceglierlo con **Sfoglia...** la deseleziona. Con **Salva come predefiniti di questo repository** diventa l'interruttore *Nessun file in più* del repository. Un campo vuoto qui conta come nessuno, perché la finestra parte da ciò che l'esecuzione erediterebbe. |
| **Salva come predefiniti di questo repository** | Conserva anche i valori per le prossime revisioni: nel file di stato del repository, o in `remote-reviews.toml` per un repository non clonato. Viene conservato solo un valore diverso da quello globale, così il resto continua a seguire *Impostazioni > IA*. |

Tutto è raggiungibile con `Tab`, `Space` attiva o disattiva una casella e il campo del percorso è un normale campo di testo. Le
stesse quattro impostazioni stanno in *Impostazioni > IA* (*Revisione del codice*) e in *Impostazioni specifiche del repository > IA*; vedi
[Contesto della revisione](ai.md#contesto-della-revisione). Ciò che non si è potuto dare al modello (un file in più mancante o
illeggibile) e ogni file accorciato per rientrare nei limiti sono elencati sulla scheda della bozza.

I suggerimenti **non sono commenti**. Compaiono in due posti:

- **Sotto la loro riga in File**, come commenti fantasma segnati con `✦` con una gravità (`info`, `avviso`, `problema`) e
  tre pulsanti: **Accetta**, **Modifica** e **Ignora**. Un suggerimento su una riga che il diff non mostra viene
  conservato, mai scartato: è elencato dopo gli hunk sotto *Suggerimenti IA sull'intero file* e, una volta
  accettato, finisce sulla prima riga modificata del file con le parole *(riguardo alla riga N)* davanti.
- **Nella Panoramica**, sotto la descrizione: il riepilogo, poi l'elenco con gli stessi pulsanti e **Accetta tutti** /
  **Ignora tutti**.

**Accetta** (`y`) trasforma un suggerimento in un normale **commento in sospeso** (puoi ancora modificarlo o eliminarlo);
**Modifica** (`w`) apre la casella del commento con prima le parole del modello e accetta ciò che lasci; **Ignora** (`n`)
lo scarta. **Accetta tutti** (`A`) riempie comunque solo l'elenco dei commenti in sospeso. Nella pagina File i tasti agiscono sul
suggerimento sotto il cursore, nella Panoramica sul primo dell'elenco. **Usa il riepilogo IA** (`i`, o il pulsante
nel modulo Termina revisione) mette il riepilogo del modello nel testo della revisione; non viene mai inserito da solo.
**Nulla viene inviato all'host finché non invii la revisione.**

La Panoramica mostra la bozza come una scheda sopra i suggerimenti: il chip dell'esito, il testo della revisione e tre
pulsanti.

- **✦ Usa la bozza...** (`u`) accetta ogni suggerimento aperto come commento in sospeso e apre *Termina la revisione* già
  compilato con il testo e l'esito della bozza. La leggi, la modifichi e la invii tu; annullando il modulo la bozza
  resta. I limiti dell'host vengono rispettati: GitLab non ha *Richiedi modifiche*, quindi il modulo lo dice e
  ripiega su *Commenta*; su Gerrit l'esito diventa un voto Code-Review di +1 o -1, solo quando il server
  consente quel valore per te (altrimenti nessun voto viene impostato e il modulo lo dice).
- **Modifica il testo...** (`t`) cambia il testo della revisione della bozza prima di usarla.
- **Scarta la bozza** (`N`) elimina il testo, l'esito e i suggerimenti rimasti.

La bozza viene conservata per richiesta, quindi c'è ancora quando chiudi e riapri la revisione. Quando
l'head della richiesta si è spostato dopo che il modello l'ha scritta, la scheda dice *La richiesta è cambiata da quando è stata scritta questa bozza.*
Inviare la revisione consuma la bozza.

## Ogni cosa ha un tasto e una riga della palette

| Azione | Tasto | Palette |
|---|---|---|
| Panoramica / File / Commit | `Alt+1` / `Alt+2` / `Alt+3` | *Revisione: Mostra panoramica*, *Mostra file*, *Mostra commit* |
| Commenta la riga o l'intervallo | `c` | *Revisione: Commenta la riga...* |
| Commenta la conversazione | `c` nella Panoramica, `C` | *Revisione: Commenta la conversazione...* |
| Rispondi alla discussione | `r` | *Revisione: Rispondi alla discussione...* |
| Risolvi o riapri | `x` | *Revisione: Risolvi o riapri la discussione* |
| Comprimi o espandi una discussione | `z` | *Revisione: Comprimi o espandi la discussione* |
| Modifica / elimina un commento in sospeso | `e` / `d` | *Revisione: Modifica il commento in sospeso...*, *Elimina il commento in sospeso* |
| Segna il file come visto | `v` | *Revisione: Segna il file come visto* |
| Inline / Split | `V` | *Revisione: Layout unificato o affiancato* |
| File intero / solo gli hunk | `H` | *Revisione: File intero o solo hunk* |
| Modifiche di soli spazi | `S` | *Revisione: Mostra o nascondi le modifiche di soli spazi* |
| A capo automatico | `W` | *Revisione: Attiva o disattiva l'a capo automatico* |
| Hunk successivo / precedente | `J` / `K` | *Revisione: Hunk successivo*, *Hunk precedente* |
| File successivo / precedente | `]` / `[` | *Revisione: File successivo*, *File precedente* |
| Termina / scarta la revisione | `f` | *Revisione: Termina la revisione...*, *Scarta la revisione in sospeso...* |
| Aggiorna | `F5`, `Ctrl+R` | *Revisione: Aggiorna* |
| Apri nel browser / checkout | `o` / `b` | *Revisione: Apri nel browser*, *Checkout del branch* |
| Assegnatari / revisori / etichette / milestone | `a` / `p` / `l` / `m` | *Revisione: Modifica assegnatari...* e le sue tre sorelle |
| Scegli il patch set (Gerrit) | `P` | *Revisione: Scegli il patch set...* |
| Confronta con un patch set più vecchio (Gerrit) | `D` | *Revisione: Confronta i patch set...* |
| Submit / abbandona la modifica (Gerrit) | `U` / `Q` | *Revisione: Esegui il submit della richiesta...*, *Revisione: Abbandona la richiesta...* |
| ✦ Bozza di revisione... (IA attiva; apre le opzioni di revisione) | `s` | *Revisione: ✦ Bozza di revisione...* |
| Accetta / modifica e accetta / ignora un suggerimento | `y` / `w` / `n` | *Revisione: Accetta il suggerimento*, *Modifica e accetta il suggerimento...*, *Ignora il suggerimento* |
| Accetta / ignora ogni suggerimento | `A` / `N` | *Revisione: Accetta tutti i suggerimenti*, *Ignora tutti i suggerimenti* |
| Usa il riepilogo IA nel testo della revisione | `i` | *Revisione: Usa il riepilogo IA nel testo della revisione* |
| Usa la bozza ✦ (compila Termina la revisione) | `u` | *Revisione: ✦ Usa la bozza...* |
| Modifica il testo della bozza | `t` | *Revisione: Modifica il testo della ✦ bozza...* |
| Scarta la bozza ✦ | `N` | *Revisione: Scarta la ✦ bozza* |
| Chiudi | `Esc` | *Revisione: Chiudi* |

Apri la palette (`Ctrl+P`) mentre la vista è aperta: le sue righe agiscono su di essa, e dicono perché una non fa nulla in questo
momento («porta prima il cursore su una riga di codice»).

## Rivedere senza un clone

Si può rivedere anche una richiesta di un repository che non è sulla tua macchina. Dal Radar, **Rivedi**
(`e`) sulla sua riga chiede come (vedi [Il Radar](radar.md#rivedere-una-richiesta)); **Rivedi tramite l'API** apre
questa stessa vista nella finestra Home, sopra il Radar. Tutto in questa pagina funziona allo stesso modo, con queste
differenze:

- Il codice (i file modificati, il diff di ogni file, i commit della richiesta, il diff di un commit, il file intero) viene
  letto dall'API dell'host, quindi la prima occhiata a una richiesta grande richiede un momento, e un file per cui l'host non
  invia un diff (un binario o uno molto grande) lo dice invece di mostrarne uno.
- I tuoi commenti in sospeso sono conservati in `remote-reviews.toml` nella cartella di configurazione, archiviati sotto l'host
  e il progetto (`host/project`), invece che nello stato di un repository. Ci sono ancora quando
  chiudi e riapri la revisione, e inviare la revisione li consuma.
- Non c'è la scelta della codifica (*Non disponibile senza un clone*) e nessun **Checkout** (dice di clonare prima il
  repository).
- `Esc` o la `✕` nell'angolo lascia la vista e restituisce il Radar. La palette (`Ctrl+P`) raggiunge le righe della
  vista come nella scheda di un repository.

Per tenere invece il clone, scegli **Clonalo, poi rivedi** (o *Radar: Clona e rivedi...* della palette):
la finestra Clona si apre sull'account collegato della richiesta con il suo repository selezionato, e la revisione si apre nella scheda del nuovo repository quando il clone è arrivato.

## Da riga di comando e per gli agenti

La stessa revisione è disponibile come `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(e gli strumenti MCP ricavati da essi): vedi il [riferimento della CLI](cli-reference.md). `threads` elenca anche gli
esiti già inviati (`reviews`). `suggest <n>` interroga il modello (IA attiva) e stampa il suo `verdict`, `body`, riepilogo e i suoi
suggerimenti come JSON senza pubblicare nulla, così uno script o un agente può scegliere i commenti da passare a
`submit --comment`.

Niente di tutto questo richiede un clone: `--project owner/name` (con `--id` quando sono configurate più integrazioni)
esegue `show`, `threads`, `comment`, `reply`, `resolve`, `submit` e `suggest` su un progetto di cui la cartella
corrente non sa nulla. `hosting review diff <n>` stampa il diff con i suoi hunk (`--api` lo legge dall'host anche
dentro un clone), e `hosting file <path> --project P [--ref R]` legge un file a una revisione, per
esempio il `CLAUDE.md` del progetto. Entrambi leggono l'API dell'host: GitHub, GitLab, Gitea, Forgejo e Gerrit.

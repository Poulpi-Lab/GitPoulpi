[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · [Français](../fr/staging-and-committing.md) · **Italiano** · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · [Português (Brasil)](../pt-BR/staging-and-committing.md) · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage e commit

Come vedere cosa hai cambiato, scegliere esattamente cosa entra in un commit, scrivere il
commit e guardare i file più a fondo (blame, cronologia, editor integrato).
Termini come *riga WIP* e *hunk* sono spiegati nel
[tour dell'interfaccia](interface-tour.md).

## Le modifiche in corso

Il lavoro non committato è la **prima riga del Log** (cerchio tratteggiato, `// WIP`, con
i contatori `✎ n` modificati e `+ n` aggiunti). Selezionala e il pannello destro mostra
le modifiche in corso:

![Le modifiche in corso: i file non in stage e in stage a destra, con il modulo di commit sotto](../images/hero.png)

- **File non in stage**: file modificati, aggiunti (non tracciati) ed eliminati non ancora nel
  prossimo commit.
- **File in stage**: ciò che conterrà il prossimo commit.
- **Percorso** o **Albero** (`m`) elenca i file in piano o in cartelle. **Comprimi tutto**
  (`z`) chiude le cartelle. **Vedi tutti i file** (`a`) mostra l'intero albero
  invece dei soli file modificati. Clic destro sullo spazio vuoto dell'elenco per
  **Espandi tutto** / **Comprimi tutto**, su una riga di cartella per **Espandi tutte le sottocartelle** /
  **Comprimi tutte le sottocartelle**; tasti `=`, `-`, `*`, `_` (vedi
  [Mouse e gesti](mouse-and-gestures.md)).
- Il titolo dice quanti file sono cambiati su quale branch.

### Stage, unstage, scarto

| Per | Fai |
|---|---|
| Mettere in stage un file | Passa sulla sua riga e fai clic su **Stage del file**, oppure selezionalo e premi `Ctrl+Space`, o `Ctrl+G s` |
| Mettere in stage una cartella | **Stage della cartella** sulla riga della cartella |
| Mettere in stage tutto | **Stage di tutto** nel titolo della sezione, oppure `Ctrl+G a` |
| Unstage | Gli stessi pulsanti e tasti nella sezione File in stage (`Ctrl+G u`) |
| Scartare le modifiche di un file | Clic destro sul file > **Scarta le modifiche...** (chiede prima conferma; `Ctrl+Z` le recupera) |

Clic destro su un file per il suo menu: stage o unstage, scarto, copia del percorso, cronologia
del file e blame, apertura nel tuo editor esterno, strumento diff o gestore file (impostati
in [Impostazioni > Diff ed editor](settings.md#diff-ed-editor)), e altro.

## Ignorare i file e trattarli come binari o come testo

Clic destro su un file > **Ignora** per ignorarlo o ignorarne l'estensione, o su una riga di cartella > **Ignora tutti i file nella cartella**. GitPoulpi chiede prima in quale file va la regola: il `.gitignore` della radice del repository (predefinito), il `.gitignore` di ogni cartella fino a quella del file, o `.git/info/exclude` (solo questo clone, mai versionato). I file non ancora esistenti sono segnati come *nuovo* e creati alla conferma. La riga è scritta relativa al file scelto: `src/a/b.txt` diventa `/a/b.txt` in `src/.gitignore`. Spostati con le frecce, scrivi per filtrare l'elenco, `Enter` scrive, `Esc` annulla. `Ctrl+Z` annulla una scrittura in un `.gitignore`; `.git/info/exclude` è fuori dalla directory di lavoro e non è annullabile.

![L'elenco che chiede in quale file va una regola di esclusione: il .gitignore del repository, il .gitignore della cartella del file o .git/info/exclude, ciascuno con la riga che scriverebbe](../images/ignore-picker.png)

Clic destro su un file > **Tratta come binario** (o **Tratta come testo** se git lo tratta già come binario) scrive una regola `.gitattributes`: `binary` o `text diff`. Scegli il file stesso o tutti i file con la sua estensione, poi il `.gitattributes` in cui scrivere, con la stessa lista di sopra (e `.git/info/attributes` solo per questo clone). Una regola già scritta per lo stesso pattern viene sostituita, mai contraddetta. Il diff del file si aggiorna subito e la barra di stato dice cosa è stato scritto e dove (e avvisa se un'altra regola prevale ancora). Una riga di cartella ha **Tratta i file della cartella come** > **Binario** o **Testo**. Le stesse regole sono nella palette dei comandi (**Modifiche in corso: Tratta il file selezionato come binario...**) e da riga di comando (`gitpoulpi treat binary|text <percorsi>`). Anche la vista File segue la regola: un file trattato come testo mostra le sue righe anche se contiene un byte NUL.

## Menu, selezione multipla e conversione nelle liste di file

Le liste di file del pannello destro (File non in stage, File in stage, i file modificati di un commit e **Vedi tutti i file**) hanno gli stessi menu e gli stessi gesti. Clicca con il tasto destro su un file, su una riga di cartella o sulla parte vuota di una lista, oppure premi `.` (`Shift+F10`, il tasto Menu) sulla riga con il focus, oppure esegui **File: apri il menu della riga o della selezione** dalla palette.

- **Un file di un commit** (e un file di **Vedi tutti i file**): **Apri diff**, **Apri file**, **Cronologia del file**, **Blame del file**, apri nel tuo editor, nel programma predefinito o in **Apri con...**, **Mostra nella cartella**, **Modifica il file**, **Copia il percorso del file**, **Copia il percorso completo del file**, **Ignora**, **Tratta come binario** o **Tratta come testo**, **Converti**, **Ripristina il file da questo commit** (chiede prima; come `git checkout <commit> -- <file>` fa anche lo stage del file, e `Ctrl+Z` riporta il file precedente) e **Elimina il file...**. Una riga che riguarda la directory di lavoro è in grigio, con il motivo, quando il file non è nella directory di lavoro. **Diff con la directory di lavoro** apre il diff della versione del commit contro il file su disco. **Crea una patch dalle modifiche del file...** scrive una patch della modifica di quel file nel commit (anche una selezione e una riga di cartella la offrono; in **Vedi tutti i file** sulle modifiche in corso crea la patch delle modifiche del file).
- **Una riga di cartella**: **Apri la cartella**, **Copia il percorso del file**, **Ignora tutti i file nella cartella**, **Tratta i file della cartella come**, **Converti**, **Crea un file in questa cartella**.
- **La parte vuota di una lista**: **Crea file** e **Apri la cartella del repository**. La lista File non in stage aggiunge **Stage di tutto** e **Scarta tutte le modifiche...**, la lista File in stage **Unstage di tutto**.

![Il menu contestuale di un file di un commit: aprire il diff, la cronologia e il blame del file, aprire in un editor o nel programma predefinito, copiare i percorsi, ignorare, convertire, ripristinare dal commit ed eliminare](../images/commit-file-menu.png)

### Selezionare più file

I gesti sono quelli del Log. **Ctrl+clic** aggiunge una riga alla selezione o la toglie. **Shift+clic** (**Alt+clic** nei terminali che tengono Shift+clic) prende l'intervallo dall'ancora. Un clic semplice ricomincia e apre il diff come prima. `Esc` cancella la selezione. Da tastiera, `Space` o `Insert` commuta la riga corrente in ogni elenco (in File non in stage e File in stage `Ctrl+Space` mette in stage la selezione, o la riga corrente) e `Shift+Su` / `Shift+Giù` estendono l'intervallo. Una riga di cartella seleziona tutti i file che contiene, chiusa o no. Le righe selezionate sono evidenziate e la barra di stato le conta. `Ctrl+A` seleziona tutte le righe dell'elenco attivo.

Clicca con il tasto destro su una riga selezionata (o premi `.`) per il menu dell'intera selezione, intitolato **N file**: stage o unstage, scarta, ignora, tratta come binario o testo, converti, copia i percorsi, ripristina dal commit, elimina. Stage, unstage e ripristino sono un'azione ciascuno; scarta ed elimina chiedono prima e dicono quanti file. Un clic destro su una riga fuori dalla selezione ricomincia la selezione con quella riga.

![Quattro file selezionati nell'elenco dei file non in stage, con il menu intitolato 4 file: aggiungere allo stage, scartare, ignorare, trattare come binario o testo, convertire, copiare i percorsi ed eliminare](../images/file-pane-multiselect.png)

### Convertire codifica, interruzioni di riga e rientro

**Converti** è nel menu di un file, di una cartella o di una selezione. `gitpoulpi convert` fa lo stesso da riga di comando.

- **Codifica**: UTF-8, UTF-8 BOM, UTF-16 LE e BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Interruzioni di riga**: LF, CRLF o CR.
- **Rientro**: spazi o tabulazioni a inizio riga, con la larghezza di una tabulazione.

I file della directory di lavoro vengono riscritti in background con le stesse funzioni della barra di stato dell'editor. Un file che non è UTF-8 valido e non ha il byte order mark viene letto come Windows-1252, come nell'editor. I file binari, mancanti o che la codifica di destinazione non può rappresentare vengono saltati e segnalati (*12 file convertiti, 3 ignorati (3 binari)*), mai danneggiati; un file che non si può scrivere genera una notifica di errore. L'intero lotto è una sola voce del registro di annullamento: un `Ctrl+Z` rimette a posto tutti i file. Un file aperto nell'editor blocca la conversione finché non viene chiuso.

## Leggere un diff

Fai clic sul nome di un file (o premi `d` su di esso) e il suo diff sostituisce il Log al
centro. `Esc` o `q` riporta il Log.

![Un diff con due hunk, ciascuno con i pulsanti Scarta l'hunk e Stage dell'hunk](../images/diff.png)

La barra sopra cambia la vista:

- **Inline** mostra il diff unificato; **Split** mette prima e dopo affiancati
  (`m` cicla i layout).
- **Hunk** (`Shift+H`) mostra solo gli hunk modificati con una riga di contesto.
  Disattivalo per vedere l'intero file.
- **WS** (`s`) nasconde le modifiche che differiscono solo negli spazi bianchi.
- **A capo** (`w`) manda a capo le righe lunghe invece di troncarle.
- **Mappa** (`Shift+M`) mostra una minimappa dell'intero file accanto alla barra di scorrimento.
  Fai clic per saltare. Le altre occorrenze di una parola selezionata vengono evidenziate e segnate
  sulla barra di scorrimento.
- `n` e `p` (o le frecce sulla barra) saltano da un hunk all'altro.
- **Auto** (l'ultimo pulsante, `Shift+C`) è la codifica con cui viene letto il file.
  Cliccalo (o premi `Shift+C`, o esegui *Diff: Scegli la codifica...* dalla palette) e
  scegline una dall'elenco: prima **Rilevamento automatico**, poi UTF-8, UTF-8 BOM,
  UTF-16 LE e BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251,
  KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 ed EUC-KR (digita per cercare nell'elenco). I due
  lati del file vengono riletti, come byte esatti, in quella codifica, così un file
  Latin-1 mostra i suoi accenti e un file UTF-16 (che git considera binario) mostra le
  sue righe. La scelta viene ricordata per il percorso di quel file fino all'uscita,
  in questa vista e nella [revisione](review.md). Finché è scelta una codifica gli hunk
  sono propri di questa vista, quindi stage e scarto sono disattivati (scegli
  **Rilevamento automatico** per tornare a fare stage). Una lettura lenta scrive
  *lettura come ...* nel titolo dopo un momento.

![Lo stesso diff nel layout split, prima e dopo affiancati](../images/diff-split.png)

Puoi selezionare il testo con il mouse (trascinando, o con doppio clic su una parola) e copiarlo
con `y` o `Ctrl+C`; viene copiato solo il testo del file, mai i numeri di riga né i marcatori `+`
e `-`.

### Mettere in stage un hunk o una singola riga

In un diff della directory di lavoro, l'intestazione di ogni hunk ha **Stage dell'hunk** e **Scarta
l'hunk** (dal lato dei file in stage, **Unstage dell'hunk**).

- `Space` mette in stage l'hunk sotto il cursore.
- Per mettere in stage alcune righe, premi `v` per iniziare una selezione, muoviti con `j`/`k` e
  premi `Enter`. Contano solo le righe *modificate*.
- `x` scarta l'hunk e `Shift+X` le righe selezionate. Entrambi chiedono prima conferma, e
  `Ctrl+Z` recupera le modifiche.
- Clic destro (o `.`) su un hunk per il suo menu: stage, unstage, scarto, cronologia del file,
  blame, copia.

Il diff di un commit è in sola lettura. Un'immagine modificata viene mostrata come immagine; vedi
[Immagini](image-view.md).

## Scrivere il commit

Il modulo di commit è in fondo al pannello destro, e il suo **Riepilogo** è anche
il campo nella riga delle modifiche in corso del Log: ciò che digiti in uno compare
nell'altro.

![Il modulo di commit compilato con un messaggio scritto dall'IA: riepilogo con contatore di caratteri, descrizione e il pulsante verde di commit](../images/ai-commit-message.png)

- **Riepilogo** (il contatore mostra quanti dei 72 caratteri restano) e
  **Descrizione**. `Enter` nel riepilogo passa alla descrizione, `Enter`
  lì aggiunge una riga.
- **Commit delle modifiche** (il pulsante verde) o `Ctrl+Enter` fa il commit delle modifiche
  in stage. Se non c'è nulla in stage, il pulsante lo dice e non fa nulla.
- **Modifica il commit precedente** (`Shift+A`) aggiunge le modifiche in stage e il nuovo messaggio
  all'ultimo commit. Un commit già inviato richiederà poi un force
  push (con lease).
- **Salta gli hook** (`Shift+H`, o **Commit: Attiva o disattiva «Salta gli hook»** nella palette)
  fa il commit senza eseguire gli hook `pre-commit` e `commit-msg` del repository. Vale per un
  singolo commit; **Impostazioni > Impostazioni specifiche del repository > Commit** lo fa partire
  selezionato. Compare quando la riga ha spazio.
- `Ctrl+G c` o `c` nel pannello Stato porta il cursore nel modulo. `Esc`
  lo lascia.
- `Ctrl+G Enter` (**Git: Esegui ora il commit delle modifiche in stage** nella palette) è il
  pulsante di commit da qualsiasi punto: fa il commit delle modifiche in stage con il messaggio
  del modulo, rispettando **Modifica il commit precedente** e **Salta gli hook**, senza spostare
  il focus. Se non c'è nulla in stage, o manca il riepilogo, lo dice e porta invece il cursore
  nel riepilogo. Finché una richiesta all'IA è ancora in corso non fa nessun commit e lo dice
  (`Esc` annulla la richiesta). `Ctrl+Z` annulla il commit
  come qualsiasi altro.
- Per ogni repository, le [Impostazioni](settings.md#specifico-del-repository-commit) possono
  fare il **push dopo ogni commit**, far partire il modulo da un **modello di commit** (togliendo
  dal messaggio i suoi commenti `#`) e rendere un merge uno **squash**: il modulo si apre
  allora con il messaggio di squash di git.
- Con un'integrazione di hosting collegata, `Alt+R` nel modulo inserisce un riferimento
  a una issue; vedi [Hosting e issue tracker](hosting-integration.md).

Con l'[IA](ai.md) attiva, il pulsante **`✦`** a destra del riepilogo scrive un messaggio
dalle modifiche in stage (`Ctrl+G i`), e **Componi commit con l'IA** divide
il tuo lavoro in più commit, qui sotto. Il testo va nel modulo perché tu lo
modifichi; non viene fatto alcun commit finché non premi il pulsante di commit.

## Il compositore di commit: dividere le modifiche in commit

*(Richiede l'IA attiva: vedi [Assistenza IA](ai.md).)* Premi **Componi commit con
l'IA** (o `Ctrl+G C`, o clic destro sulla riga delle modifiche in corso). Il modello propone
più commit, ciascuno con i suoi file; ogni file modificato sta in esattamente un
commit, e un piano non valido viene rimandato una volta al modello. La proposta si apre
come una revisione in cui non è stato ancora committato nulla:

![Il compositore di commit: quattro commit proposti, ciascuno espandibile ai suoi file, con i controlli su, giù, modifica e drop e i pulsanti Reimposta, Annulla e Crea 4 commit](../images/composer.png)

| Per | Fai |
|---|---|
| Espandere o comprimere un commit | `Sinistra` / `Destra`, oppure clic sul chevron; `*` li alterna tutti |
| Spostare un commit su o giù | I `▲` `▼` sulla sua riga, oppure `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Spostare un file in un altro commit | Trascinalo sul commit (dall'elenco, o dall'elenco delle modifiche in corso a destra); oppure `m` sul file e scegli il commit dal menu a discesa; oppure `Alt+Up` / `Alt+Down` sul file |
| Mettere un file in un commit a sé | `n` sul file, oppure il pulsante `+ commit a sé` |
| Riformulare un commit | La matita, `e`, `Enter` o un doppio clic: modifica il messaggio, `Enter` lo conserva, `Esc` lo scarta |
| Lasciare fuori un commit | `drop` (`d` o `Delete`); i suoi file restano non committati. `ripristina` lo riporta |
| Ricominciare | **Reimposta** (`r`) torna al piano del modello |
| Crearli | **Crea N commit** (`Ctrl+S`, `Ctrl+Enter` o `a`) |
| Uscire | **Annulla** o `Esc`: non viene fatto nulla |

Creare i commit mette tutto in unstage, poi per ogni commit mette in stage i suoi file
e fa il commit, così la tua directory di lavoro non viene mai toccata. Se un passaggio fallisce, si ferma
e dice quanti commit sono stati fatti. Vengono messi in stage file interi: un file che era
in stage solo in parte viene committato per intero. Scegli quale profilo e quali istruzioni
usa il compositore in *Impostazioni > IA*; vedi [Assistenza IA](ai.md#il-compositore-di-commit).

## Vista file, Blame e Cronologia del file

La barra delle viste ha **Vista file** (il contenuto del file), **Vista diff**, **Blame** e il
selettore `File │ Tutti` della [colonna dei commit](interface-tour.md#la-colonna-dei-commit) (il suo
elenco `File` è la Cronologia del file); le stesse sono nel menu del clic destro di un file. Mostrano il file
com'era al commit da cui sei arrivato, o non committato (`@ WIP`) quando lo hai aperto
dalle modifiche in corso.

- **Blame** e la **Cronologia del file** sono viste complete a sé: vedi [Blame](log-and-history.md#blame) e [Cronologia del file](log-and-history.md#cronologia-del-file) in *Lavorare con il Log*. `Enter` su un hunk del blame o su una riga della cronologia va a quel commit nel Log.
- `y` o `Ctrl+C` copia il testo selezionato (nel Blame, solo il codice).
- `Shift+G` salta all'ultima riga. `q` chiude la vista.

## L'editor integrato

**Modifica** (`e`) apre la *copia del file nella directory di lavoro* in un piccolo editor
sopra il pannello centrale: colori della sintassi, numeri di riga, annulla e ripeti, cerca e una
barra di stato (posizione, fine riga, rientro, linguaggio). Modifica sempre la
copia nella directory di lavoro, anche quando l'hai aperto dal diff di un vecchio commit.

![L'editor che mostra un file Rust con evidenziazione della sintassi e numeri di riga; la barra ha Salva, Annulla, Ripeti e Cerca](../images/editor.png)

| Tasto | Azione |
|---|---|
| `Ctrl+S` | Salva (il file viene scritto in un file temporaneo e rinominato, così non è mai scritto a metà; fine riga, BOM e a capo finale vengono preservati) |
| `Ctrl+Z`, `Ctrl+Y` | Annulla, ripeti |
| `Ctrl+F`, `F3` | Cerca, cerca successivo |
| `Tab`, `Shift+Tab` | Indenta, riduci il rientro (secondo lo stile del file stesso) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Seleziona tutto, copia, taglia, incolla |
| `Esc` | Chiudi. Con modifiche non salvate chiede **Salva** (`s`), **Scarta** (`d`) o **Annulla** (`c`) |

I file binari vengono rifiutati. Dimensione della tabulazione, fine
riga, evidenziazione della sintassi, numeri di riga e a capo automatico sono in *Impostazioni >
Diff ed editor*; vedi [Impostazioni](settings.md#diff-ed-editor).

**Segui la selezione.** Con **Segui la selezione** attivo (la casella nella barra dell'editor, anche *Editor: Attiva o disattiva segui la selezione* nella palette e *Impostazioni > Diff ed editor*), selezionare un altro file nel pannello destro con l'editor aperto apre la copia di lavoro di quel file nell'editor, invece del suo diff o della vista file. Con modifiche non salvate chiede prima **Salva**, **Scarta** o **Annulla**; Annulla ti lascia sul file attuale. Un file che non può essere modificato (binario, troppo grande, eliminato) si apre nella vista abituale e la riga di stato ne dice il motivo. Chiudendo l'editor si arriva alla vista del file selezionato. Attivo di base. Vale per le modifiche di lavoro: scegliere un file di un commit o di un confronto durante la modifica fa uscire dall'editor (chiedendo prima di salvare) e mostra il diff di quel file in quel commit, e con esso torna la colonna dei commit, che l'editor non ha.

La barra di stato, a destra, mostra la **codifica**, le **interruzioni di riga** e il
**rientro** del file, poi la posizione del cursore e la sintassi. I primi tre sono
pulsanti: ognuno apre un menu a discesa (frecce o rotellina, scrivi per filtrare, `Enter`
per scegliere, `Esc` per chiudere). La palette (`Ctrl+P`) apre gli stessi menu: *Editor: Cambia codifica*, *Editor: Riapri con codifica*, *Editor: Converti in codifica*, *Editor: Converti interruzioni di riga*, *Editor: Cambia rientro*.

![Il menu della codifica dell'editor aperto sotto il pulsante UTF-8 della barra: riaprire il file con un'altra codifica oppure convertirlo](../images/editor-encoding-menu.png)

![Il menu dei fine riga dell'editor aperto sotto il pulsante LF: convertire il file in LF, CRLF o CR](../images/editor-eol-menu.png)

- **Codifica.** Un file si apre nella codifica che dichiara (un BOM: UTF-8 o UTF-16),
  altrimenti in UTF-8, altrimenti in Windows-1252, e la barra dice quale. **Riapri con
  codifica** rilegge i byte del file con un'altra codifica (chiede prima se ci sono
  modifiche non salvate): usalo quando il rilevamento ha sbagliato. **Converti in
  codifica** mantiene il testo e cambia la codifica in cui il file viene salvato (UTF-8,
  UTF-8 BOM, UTF-16 LE e BE, Windows-1252, ISO-8859-1 e -15, Windows-1250, Windows-1251,
  KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR). Non viene scritto nulla finché non
  salvi, e Annulla lo ripristina. Se il testo contiene caratteri che la destinazione non
  può contenere, la barra di stato dice quanti e dov'è il primo, e non viene convertito
  nulla; il salvataggio rifiuta allo stesso modo, non scrive mai `?`.
- **Interruzioni di riga.** Mostra `LF`, `CRLF`, `CR` oppure `Misto` quando il file ne
  usa diverse. Il menu converte tutte le interruzioni di riga del file in LF, CRLF o CR
  in un solo passo di annullamento. Le nuove righe che digiti usano l'interruzione del
  file. La tua scelta prevale sull'impostazione *Interruzioni di riga*.
- **Rientro.** Mostra `Spazi: 4` o `Tabulazioni: 4`. Il menu sceglie cosa inseriscono `Tab` e il
  rientro automatico (spazi o tabulazioni), la dimensione (da 1 a 8: il passo di rientro
  e la larghezza di una tabulazione in questo file), e converte gli spazi iniziali di
  ogni riga in spazi o in tabulazioni, in un solo passo di annullamento.

## Stash, patch e Git LFS

- **Stash** e **Pop** sono nella barra degli strumenti; vedi
  [Lavorare con il Log](log-and-history.md#branch-tag-e-la-barra-laterale-refs).
- **Crea patch** e **Applica patch** sono nei menu dei commit e dei riferimenti; la CLI
  ha `gitpoulpi patch`.
- I file grandi memorizzati con Git LFS compaiono come piccoli file puntatore finché non scarichi
  il loro contenuto; vedi [Git LFS](lfs.md).

## Da riga di comando

Tutto ciò che c'è in questa pagina è automatizzabile:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

Vedi il [riferimento della CLI](cli-reference.md).

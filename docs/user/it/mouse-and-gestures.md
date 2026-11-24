[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · **Italiano** · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Mouse e gesti

Tutto ciò che puoi fare con il mouse ha anche un equivalente da tastiera. Vedi
[Scorciatoie da tastiera](keybindings.md). I nomi delle aree sono nel
[tour dell'interfaccia](interface-tour.md).

## Regole generali

- **Clic** su una riga per selezionarla. Vale in ogni pannello, elenco, menu e
  dialogo.
- **Doppio clic** su una riga per aprirla o eseguirla. Un doppio clic su un
  branch locale, un tag o una pillola di ref ne fa il checkout. Un doppio clic su
  una riga di repository apre il repository.
- **Clic destro** su una riga per il suo menu. `.`, `Shift+F10` o il tasto Menu
  aprono lo stesso menu per la riga selezionata.
- **La rotella** fa scorrere l'area sotto il puntatore. In un elenco sposta solo la
  vista. La selezione resta dov'è, anche quando esce dalla vista scorrendo. Solo un
  clic o un tasto sposta la selezione.
- **Shift+rotella** scorre lateralmente, nel diff, nella Vista file e nell'editor
  quando l'a capo automatico è disattivato.
- **Passare il mouse** su un pulsante, una riga o il cursore di una barra di
  scorrimento lo illumina. Molti terminali non sanno cambiare il puntatore del
  mouse, quindi questa luce è spesso l'unico segno che qualcosa è cliccabile.
  Fermare il puntatore su un pulsante o su una cella del Log mostra un tooltip.
- **Clic fuori** da un campo di testo per uscirne. Ciò che hai scritto viene
  mantenuto. Il filtro dei Refs applica ciò che hai scritto, come fa `Enter`. La
  barra di ricerca del Log si chiude, come fa `Esc`.

## Chiudere finestre e dialoghi

Ogni finestra e ogni dialogo con cornice ha una `✕` in alto a destra del bordo, e ovunque
valgono le stesse regole:

- **Clic su `✕`** per chiuderlo. `Esc` fa lo stesso e chiude prima ciò che sta più
  all'interno (un elenco a discesa aperto, un campo di testo, una conferma) e poi la finestra.
- **Clic all'esterno** di un dialogo, di un pannello di aiuto o di informazioni o di un popup
  per chiuderlo. Un clic all'esterno non chiude mai una vista a tutto schermo (Impostazioni,
  vista Diff, vista file, editor, Revisione, Board): non hanno un esterno.
- Un clic **dentro** un pannello non lo chiude mai.
- Una pressione del mouse (sinistro o destro) mentre un chord `Ctrl+G` o `Ctrl+N` attende il secondo tasto annulla il chord come `Esc`, poi agisce come al solito (un clic seleziona ancora, un clic destro apre ancora il suo menu).
- Un modulo che contiene testo digitato da te (una nuova issue, una pull o merge request)
  ignora un clic all'esterno, così un clic involontario non te lo fa perdere. `✕` e `Esc`
  chiedono prima **Scartare le modifiche?**; **Modifica ancora** torna indietro. Un modulo
  senza testo digitato si chiude subito, come un dialogo a riga singola.
- Anche la palette dei comandi, i selettori (schede, tema, lingua), i piccoli dialoghi di
  revisione, board, merge e compositore e la richiesta di conferma hanno la `✕`. I menu
  contestuali e gli elenchi a discesa no: si chiudono con `Esc` o con un clic all'esterno.

## Elenchi nelle Impostazioni

Gli elenchi delle Impostazioni (Profili, profili IA, Workflow, Apri in, Immagini) rispondono
al mouse così. **Clicca una sotto-scheda** (Generali o l'elenco) per passarci; `Alt+Left` /
`Alt+Right` fa lo stesso.

- **Clic** su una riga per selezionarla, **doppio clic** per passare ai suoi campi,
  **rotella** per scorrere senza selezionare.
- **Premi una riga e trascinala** in su o in giù per riordinare: una barra mostra dove
  finirà, rilasciando la rilasci lì, `Esc` annulla. L'equivalente da tastiera è `Alt+Su` /
  `Alt+Giù`.
- **Clic su `+ Aggiungi`, `Duplica`, `Rimuovi`, `↑`, `↓`** sotto l'elenco. Tasti:
  `Insert`, `Ctrl+D`, `Delete`, `Alt+Su` / `Alt+Giù`; vedi
  [Impostazioni](settings.md#elenchi).

## Elenchi nei popup

Queste regole valgono per ogni elenco in un popup che ha un campo filtro: la
palette dei comandi, i suoi selettori (tema, lingua, colori del grafo), il
selettore di schede, i menu a tendina ricercabili, l'elenco dei riferimenti alle
issue e l'aiuto.

- **Clic** su una riga per selezionarla.
- **Doppio clic** su una riga per eseguirla o sceglierla, come fa `Enter`.
- **La rotella** fa scorrere l'elenco. Non sposta la selezione.
- Quando l'elenco è più lungo del popup, sul suo bordo destro compare una **barra
  di scorrimento**. Trascina il cursore o clicca sulla guida.

Un breve menu a tendina senza campo filtro sceglie la riga al primo clic. Un menu
contestuale esegue la riga al primo clic. I tasti per questi elenchi sono in
[Scorciatoie da tastiera](keybindings.md#elenchi-popup-filtrabili).

## Barre di scorrimento

Ogni area che può scorrere mostra una sottile barra di scorrimento sul suo bordo
quando il contenuto non entra: il Log, le sezioni Refs, gli elenchi di file, il
diff, la Vista file, l'editor e gli elenchi dei popup.

- **Trascina il cursore** per scorrere.
- **Clicca sulla guida** per spostarti di una pagina verso il clic. Nel Log e negli
  elenchi di file, la vista salta al punto cliccato.
- **Tieni premuto il pulsante su una freccia di estremità** per continuare a
  scorrere. Parte dopo un breve ritardo. Sposta il puntatore fuori dalla freccia
  per mettere in pausa.
- Una barra di scorrimento sposta solo la vista. Non sposta mai la selezione.

Nel diff e nell'editor, la barra di scorrimento verticale mostra anche l'intero
file con piccoli segni: verde per le righe aggiunte, rosso per le righe rimosse,
ambra per le righe modificate, `◆` per la riga del cursore e `▌` per ogni altro
punto in cui compare il testo selezionato. L'editor di merge segna ogni conflitto
sulle sue barre di scorrimento. Clicca un segno per andarci.

## Errori e Registro attività

- **Clicca l'errore rosso** nella barra di stato per aprire il Registro attività
  sulla sua voce; **clicca Copia / il pulsante suggerito** (Riprova, Prima Pull,
  Stash e riprova, Apri Integrazioni) sulla voce.
- **Clicca Attività** all'estremità destra della barra di stato per aprire il
  Registro attività. Clicca una scheda per cambiare, clicca la casella di un
  livello (o **Tutti** / **Nessuno**) per mostrare o nascondere quelle righe,
  clicca una voce di Repository per aprirne i dettagli, clicca **Svuota il registro**
  nella scheda Applicazione (chiede prima conferma). Scorri lateralmente le
  righe lunghe con `Shift` + rotella o con la barra di scorrimento orizzontale.
  Vedi [Il Registro attività](troubleshooting.md#il-registro-attività).
- **Clic su `[x]`** dell'indicatore delle attività a destra della barra di stato per annullare quella in corso, **clic sulla sua etichetta** per aprire il pannello delle attività. Il pulsante **Attività** in basso a destra nella scheda Home (o `Ctrl+N Shift+G`) apre il registro attività su Tutti i repository (la sua casella Ambito, un menu a tendina sotto le schede, sceglie un repository o un'area di lavoro).

## Ridimensionamento

- **Trascina un divisore** tra la barra laterale Refs, il Log e il pannello destro.
  Trascinalo oltre la larghezza minima per comprimere quel pannello. La tastiera li sposta di un passo alla volta: `Ctrl+N Left` / `Ctrl+N Right` spostano il divisore Refs | Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` quello Log | pannello destro (il divisore va nella direzione della freccia). La palette li ha come `Navigazione: Allarga il pannello Refs`, `Navigazione: Restringi il pannello Refs`, `Navigazione: Allarga il pannello Stato` e `Navigazione: Restringi il pannello Stato`, più `Navigazione: Comprimi o ripristina il pannello Refs` / `Navigazione: Comprimi o ripristina il pannello Stato`.
- **Trascina il bordo inferiore** di una sezione Refs per cambiarne l'altezza.
- **Trascina il bordo inferiore** del blocco del messaggio di commit nel pannello
  destro.
- **Trascina il bordo tra due intestazioni del Log** (per impostazione predefinita lo spazio, dall'intestazione o da qualsiasi punto della tabella; Impostazioni > Log e grafo > Ridimensionamento delle colonne) per ridimensionare una colonna (da tastiera: `Shift+C` per scegliere una colonna, poi `]` / `[`).
- **Doppio clic sul bordo tra due intestazioni del Log** per adattare la colonna alla sua sinistra al contenuto (da tastiera: `Shift+C` per scegliere una colonna, poi `=`; `+` adatta ogni colonna). L'ultima colonna ha una maniglia sulla sua ultima cella: trascinala per ridimensionare quella colonna, e la colonna Messaggio prende la differenza.
- **Scorri il Log lateralmente** (la disposizione `scroll` delle colonne; Impostazioni > Log e grafo > Disposizione delle colonne, o la casella `Scorri lateralmente` del selettore): la rotella orizzontale o `Shift`+rotella sopra la tabella, la barra sul bordo inferiore (trascina il cursore, premi la traccia libera, clicca le frecce alle estremità), oppure `Shift+Sinistra` / `Shift+Destra`. Nella disposizione `fit` predefinita le colonne riempiono il pannello e non c'è nulla da scorrere.
- **Trascina un'intestazione del Log** per spostare la colonna: un fantasma del suo nome segue il puntatore, l'intestazione che ha lasciato è attenuata e una barra d'accento segna lo slot in cui finirebbe. Le colonne si spostano al rilascio; `Esc` annulla e il puntatore mostra la mano aperta mentre la porti (da tastiera: `Shift+C` per scegliere una colonna, poi `<` / `>`).
- **Scorri lateralmente un pannello a elenco** quando una riga è più larga del pannello: una sezione Refs (il nome lungo di un branch, di un tag o di uno stash), gli elenchi File non in stage e File in stage, i file modificati del commit (o di un confronto) e le righe di informazione dei dettagli del commit (id, autore, data di creazione, conteggi). La rotella orizzontale o `Shift`+rotella sopra il pannello lo fa scorrere; una barra compare sul bordo inferiore del pannello solo finché qualcosa trabocca (trascina il suo cursore, premi la guida vuota, clicca le frecce alle estremità), quindi non occupa mai una riga. Da tastiera, `Alt+Left` / `Alt+Right` fanno scorrere il pannello che ha il focus (il Log fa scorrere i dettagli del commit; dentro i file modificati, i file); la palette ha `Vista: Scorri il pannello a sinistra/destra`. Scorre l'intera riga (rientro, segno di stato, freccia e nome insieme); il titolo del pannello, il suo bordo, le barre di scorrimento e il pulsante Stage/Unstage di una riga di Stato sotto il puntatore restano fermi. Un nome non viene più tagliato con i puntini di sospensione: la sua coda è a uno scorrimento di distanza (un branch con altre destinazioni di push mantiene invece in vista il suo badge `⇉`, quindi è il suo nome a cedere). L'offset è per pannello, si ferma alla riga più larga in vista e viene dimenticato quando cambi repository. Il blocco del messaggio di commit manda a capo il testo, quindi non ha nulla da scorrere. Mentre una sezione è scorsa lateralmente il suo bordo inferiore porta la barra, quindi ridimensiona quella sezione con i suoi comandi da tastiera o da una sezione che ci sta tutta.

GitPoulpi ricorda ogni dimensione. Vedi
[Le dimensioni vengono ricordate](interface-tour.md#le-dimensioni-vengono-ricordate).

## Barra del titolo, barra delle schede e barra degli strumenti

- **Clicca <img src="../../../assets/logo/gitpoulpi.png" alt="Icona di GitPoulpi" height="16">** nella barra del titolo per aprire **Informazioni**. In
  Informazioni le pupille del polpo seguono il puntatore; clicca il logo per
  farlo girare sul colore successivo, clic destro per l'elenco dei colori, clicca
  un link per aprirlo.
- **Clicca Apri in ▾** nella barra del titolo per l'elenco dei programmi. Vedi
  [Apri in](open-in.md).
- **Clicca la versione** (`v1.0.0`, oppure `vX.Y.Z disponibile`) nella barra del titolo per aprire la scheda Note di rilascio.
- **Clicca la lente e `Ricerca globale`** nella barra del titolo, subito a sinistra dell'ingranaggio, per la
  [ricerca globale](search.md).
- **Clicca l'ingranaggio** nella barra del titolo, a sinistra della ×, per le
  Impostazioni.
- **Clicca ×** all'estremità destra della barra del titolo per uscire.
- **Clicca una scheda** per passare a essa. **Clicca la sua `x`** (mostrata sulla scheda
  attiva e su quella sotto il puntatore) per chiuderla. **Trascina una scheda** di lato per
  riordinarla. **Clic destro su una scheda** per il menu della scheda.
- **Clic con il tasto centrale su una scheda** per chiuderla, come la sua `x`.
- **Clicca `‹` o `›`** alle estremità delle schede (compaiono quando non ci stanno tutte) per far scorrere la riga di una scheda; tieni premuto il pulsante per continuare a scorrere, oppure usa la rotella sulla riga. Anche trascinare una scheda su una freccia fa scorrere la riga. La riga scorre da sola per mostrare la scheda attiva ogni volta che cambia.
- **Clicca il `+`** dopo l'ultima scheda per un menu: Nuova scheda (`Alt+T`) e, finché le loro schede sono chiuse, Radar e Aree di lavoro (`Alt+J`).
- **Clicca l'icona della casa** per la scheda Home.
- **Clicca la scheda Radar o Aree di lavoro** per aprirla; la sua `x`, un clic centrale o `Ctrl+W` la chiude. **Clicca il chip dell'area di lavoro** (`Alt+W`) per cambiare area di lavoro.
- **Clicca «Esci e cd qui»** (a destra delle schede, se hai avviato GitPoulpi tramite il wrapper `gp` della shell) per uscire e lasciare la shell nella cartella della scheda attiva. Vedi [Seguire la cartella della scheda attiva](following-the-tab-folder.md).
- **Clicca un pulsante della barra degli strumenti** per eseguirlo. **Clicca `▾`**
  accanto a Pull, Push o LFS per il suo elenco. **Clicca `⋯`** per i pulsanti che
  non entravano.
- **Clicca la combobox dei profili** all'estremità destra della barra degli
  strumenti per scegliere l'identità di commit per questo repository.

## Scheda Home

- **Clic** su una riga per selezionarla. **Doppio clic** per aprire il repository.
- **Clicca la stella** per aggiungere o rimuovere un preferito.
- **Clicca Apri, Vai a, Chiudi o Rimuovi** su una riga, oppure **`⋮`** per il menu
  della riga. **Clic destro** su una riga per lo stesso menu.
- **Clicca il titolo di una sezione** per comprimerla.
- **Clicca Sfoglia, Clona, Init, Comprimi tutto o Espandi tutto** in alto, e
  **Chiudi tutte le schede** o **Rimuovi tutto** sul titolo di una sezione.

Nel selettore di cartelle **Sfoglia**: clicca un luogo o una cartella per andarci,
doppio clic su una cartella per aprirla, clicca il campo del percorso per scrivere
un percorso. Spunta **Mostra le cartelle sorelle** (`Alt+S`, ricordato) e l'albero a sinistra elenca anche le altre cartelle di ogni livello aperto: le cartelle sul percorso aperto mostrano `▾`, le altre `▸`, e un clic su una di esse ci porta lì. In **Clona** e **Init**: clicca un campo per scriverci, e clicca un
pulsante o una casella di controllo per usarli.

## Barra laterale Refs

- **Clic** su una riga per selezionarla. **Doppio clic** su un branch locale o su un
  tag per fare il checkout. **Doppio clic** su un worktree o un sottomodulo per
  aprirlo in una scheda. Il **+** sull'intestazione WORKTREES crea un worktree; clic
  destro su un worktree per Apri in una nuova scheda, Apri in, Merge e Rimuovi
  ([Worktrees](worktrees.md)).
- **Passa il mouse** su una riga per mostrare l'icona alla sua sinistra. **Cliccala**
  per nascondere il ref nel Log, o per mostrarlo di nuovo. Mentre **Solo** è
  attivo, lo stesso punto mette il ref in solo. Su una riga di cartella, l'icona
  agisce su ogni ref della cartella.
- **Clicca Termina il solo** o **Mostra tutto** nel titolo della barra laterale per
  annullare ogni solo o ogni ref nascosto.
- **Clic destro** su una riga per il suo menu: checkout, merge, fast-forward,
  rebase, creare un worktree, rinominare, eliminare, nascondere, solo e altro. La
  riga di un remoto ha **Fetch**, **Modifica**, **Rimuovi** e altre azioni del
  remoto. Uno stash ha **Pop**, **Applica** ed **Elimina**. Una riga grigia dice
  perché non può essere eseguita.
- **Clicca `+`** sul titolo REMOTI per aggiungere un remoto.
- **Clicca una riga di cartella** per comprimerla o espanderla.

### Pull request e issue

| Gesto | Azione |
|---|---|
| Clic su una riga | La seleziona. |
| Doppio clic su una riga | La apre nel browser. |
| Clic destro su una riga | Il suo menu: apri, copia l'URL o il numero, fai il checkout del branch di origine, crea un branch da una issue. |
| Clic su `+` nel titolo della sezione | Apre il modulo **Crea pull request** o il modulo **Nuova issue**. |
| Clic sulla riga del filtro (`Aperte ▾`) | Sceglie quali richieste o issue elencare. |
| Clic sul titolo di un gruppo | Comprime o espande il gruppo. |
| Clic sulla riga del provider o del repository (ISSUE) | Sceglie la fonte delle issue. **Nessuno (scollega questo repository)** la rimuove. |
| Clic sulla riga della query (ISSUE) | Scrivi una query JQL o WIQL (Jira, Azure DevOps), oppure scegli le liste di Trello. |
| Clic sul chip della barra degli strumenti (`GH #12`, `GL !53`, ...) | Apre la pull request nel browser. Clic destro per **Apri nel browser**, **Copia l'URL**, **Aggiorna** e **Cambia profilo**. |
| Clic sul tenue chip `+ PR` | Avvia una pull request per il branch inviato. |

Vedi [Hosting e issue tracker](hosting-integration.md).

## Il Log

- **Clic** su un commit per selezionarlo. I suoi dettagli compaiono nel pannello
  destro.
- **Ctrl+clic** aggiunge un commit alla selezione o lo toglie. **Shift+clic**
  seleziona un intervallo (**Alt+clic** nei terminali che tengono Shift+clic per la
  propria selezione di testo, come kitty e Windows Terminal). Clic destro su un
  commit selezionato per fare il cherry-pick di tutti. La selezione può includere
  la riga WIP e gli stash; due o più righe trasformano il pannello destro in un
  confronto, vedi [Confrontare i commit](log-and-history.md#confrontare-i-commit).
- **Clicca una scheda** del pannello di confronto per portare il cursore del Log
  su quella riga; **clicca Inverti** per scambiare le estremità Da e A. Un semplice
  clic su una riga esce dal confronto.
- **Clicca la riga WIP** per le modifiche in corso. **Clicca il suo testo `// WIP`**
  per scrivere lì il riepilogo del commit.
- **Passa il mouse** su una cella per il testo completo: il messaggio, l'autore e
  l'e-mail, la data completa, l'hash completo.
- **Passa il mouse su un badge `+N`** per gli altri ref del commit. Entra nel popup
  per cliccare un ref. Doppio clic su un ref nel popup per farne il checkout.
- **Doppio clic su una pillola di ref** per farne il checkout.
- **Clic destro su un commit** per il menu del commit. Ha un sottomenu per ogni ref
  del commit.
- **Clic destro su una pillola di ref** per un unico menu con le azioni del ref e
  quelle del commit.
- **Clic destro sull'intestazione** per mostrare o nascondere le colonne: un elenco di caselle (`[x]` / `[ ]`) con una ✕ nel titolo; clicca una riga o usa le frecce e `Spazio` / `Enter`, poi chiudilo con la ✕, `Esc` o un clic fuori (la palette: `Vista: Log: Scegli le colonne da mostrare`). Sotto le caselle: `Adatta <colonna> al contenuto` (la colonna cliccata), `Adatta tutte le colonne al contenuto` e `Reimposta le larghezze delle colonne`. L'equivalente da tastiera del clic destro: `Shift+C` per scegliere una colonna, poi `.` (o `Shift+F10`) apre lo stesso menu su di essa.
- **Le colonne del Radar** funzionano allo stesso modo: trascina il separatore dopo la colonna del repository, `#`, autore o età per dimensionarla, **fai doppio clic su di esso** per adattare la colonna alla sua voce più larga (`C` sceglie una colonna, poi `=`; `+` adatta ogni colonna), e **clic destro sul titolo di una colonna** (o `C`, poi `.`) per `Adatta <colonna> al contenuto` e `Adatta tutte le colonne al contenuto`.
- **Clicca l'imbuto nell'intestazione Autore** (o `Shift+A` nel Log, o il comando della palette `Log: Filtra per autore...`) per scegliere i commit di quali autori restano accesi; gli altri si attenuano come con una ricerca. `Enter` o un clic su una riga attiva/disattiva un autore, la riga **Rimuovi il filtro** (✕) toglie il filtro, `Esc` svuota il testo di ricerca e poi chiude il popup, mantenendo il filtro. `Esc` nel Log toglie un filtro acceso. **Clic destro sull'imbuto acceso** per toglierlo (la palette: `Log: Rimuovi il filtro per autore`).
- **Clicca `↑`, `↓` o `✕`** nella barra di ricerca per andare al risultato
  precedente o successivo, o per chiudere la barra.
- **Clicca `#12`, `ABC-12` o `AB#12`** in un messaggio di commit, nel pannello
  destro, per aprire quella issue o pull request nel browser.

### Sezioni del pannello sinistro

Le **cartelle** (branch e tag chiamati `a/b/c`, i file del pannello destro nella vista ad albero) si espandono e si
comprimono tutte insieme: **clic destro sull'intestazione o sulla parte vuota di una sezione** che ha delle
cartelle per **Espandi tutto** / **Comprimi tutto** (e **Mostra/nascondi sezioni...**, che porta alla lista di
controllo qui sotto), e **clic destro su una riga di cartella** per **Espandi tutte le sottocartelle** / **Comprimi
tutte le sottocartelle** (la cartella e tutto ciò che sta sotto). Tasti, nel pannello Refs, negli elenchi Non in
stage e In stage e nei file modificati di un commit: `=` espande tutto, `-` comprime tutto, `*` (o `Shift+Right`)
espande la cartella sotto il cursore e tutto ciò che sta sotto, `_` (o `Shift+Left`) la comprime. Palette:
*Refs: Espandi tutte le cartelle*, *Refs: Comprimi tutte le cartelle*, *Refs: Espandi la cartella selezionata e le
sue sottocartelle*, *Refs: Comprimi la cartella selezionata e le sue sottocartelle* e le quattro gemelle *File:*
per l'elenco di file con il focus. Mentre un filtro sul nome è attivo (il filtro dei Refs, *Filtra i file* di un
commit), ogni cartella che contiene una corrispondenza viene mostrata aperta qualunque cosa avessi compresso, e le
tue compressioni non vengono né cambiate né salvate: cancella il filtro e tornano. Una compressione fatta durante il
filtro agisce su ciò che è mostrato e viene dimenticata insieme al filtro.

**Clic destro sull'intestazione di una sezione** (una sezione senza cartelle, o la parte vuota del pannello sinistro) per una
lista di controllo delle sue sezioni: Locali, Remoti, Worktree, Stash, Tag, Pull request, Issue e
Sottomoduli. Un clic spunta o toglie la spunta a una sezione; il menu resta aperto. Una sezione nascosta
non viene disegnata e la tastiera la salta. L'ultima sezione visibile non può essere tolta. La scelta è per repository e si conserva tra un riavvio e l'altro; un repository senza scelta propria segue il valore predefinito (il `ui.sidebar_sections_hidden` globale). *Usa per tutti i repository* rende la scelta attuale il valore predefinito, e *Ripristina predefinite* scarta la scelta propria di questo repository (palette: *Barra laterale: Usa queste sezioni per tutti i repository*). Palette:
*Barra laterale: Mostra o nascondi le sezioni...*, un comando *Mostra o nascondi* per ogni sezione,
*Ripristina le sezioni predefinite*.

### Trascinare i ref

Premi su un branch, un branch remoto o un tag nella barra laterale Refs, o su una
pillola di ref nel Log, e trascinalo su un altro ref. La barra di stato dice cosa
stai portando. Un bersaglio che può accettare il rilascio si illumina. Rilascia per
ottenere un menu. `Esc` annulla il trascinamento. Il Log resta dov'è mentre porti:
premere un ref fa scorrere il Log al suo commit solo quando si rivela un clic (il
salto avviene quando il pulsante viene rilasciato), così puoi prima scorrere fino al
bersaglio del rilascio e poi prendere il ref.

| Trascinamento | Menu |
|---|---|
| Branch locale X su branch locale Y | **Merge di X in Y**, **Rebase di X su Y**, **Rebase interattivo di X su Y...**, **Fast-forward di Y a X** |
| Branch locale su un branch remoto | **Push di X** verso quel remoto |
| Branch remoto su branch locale Y | **Merge**, **Rebase di Y su di esso**, **Fast-forward di Y a esso**, e **Pull** quando è l'upstream di Y |
| Tag T su qualsiasi branch, branch remoto o tag | **Sposta il tag T qui**; marcato *(non è un fast-forward: T non sarebbe più un antenato)* quando la destinazione non discende dal commit del tag (a tua scelta); disattivato *già lì* quando è già su quel commit |
| Tag T su un branch locale Y | anche **Merge del tag T in Y**, e sotto una linea il più raro **Merge di Y nel tag T e sposta il tag** (un commit di merge del commit del tag e di Y, detached, poi il tag si sposta su di esso) |

Quando l'azione cambia un branch di cui non è stato fatto il checkout, GitPoulpi ne
fa prima il checkout. Se git rifiuta il checkout, non cambia nulla. Solo i tag
possono essere rilasciati su un tag, e un branch non può essere rilasciato su uno.

Spostare un tag ne mantiene il tipo: un tag leggero resta leggero, un tag annotato
viene ricreato annotato con il suo messaggio e la riga dell'autore del tag riportati
invariati (un tag firmato viene rifiutato, la sua firma non corrisponderebbe più). Si sposta solo
il tuo tag locale: un remote che ha il tag mantiene la vecchia posizione, il tag
diventa *differs* e la barra di stato lo dice; fai tu il push quando lo vuoi. Lo spostamento e
il merge con spostamento sono ciascuno un passo di annullamento (l'annullamento rimette il vecchio
oggetto tag). Un merge in un tag che va in conflitto non cambia nulla. Palette:
`Refs: Sposta il Tag selezionato sul Commit selezionato...` sposta il tag del pannello Refs sul
commit selezionato nel Log (chiede prima quando non è un
fast-forward); ogni altra destinazione passa da segna poi rilascia, più sotto.

Da tastiera: «segna poi rilascia» fa ciò che fa un trascinamento, per qualsiasi
coppia di ref. Nel pannello Refs premi `m` su un branch (palette: `Refs: Segna come
origine del rilascio`). Il ref mostra un `◆` ed è sottolineato, anche sulle sue
pillole nel Log, e la barra di stato dice dove andare. Seleziona un altro ref, in
qualsiasi sezione, e premi `Shift+M` (palette: `Refs: Rilascia qui il ref
segnato...`): compare lo stesso menu di un trascinamento, e le sue voci si
eseguono esattamente come dopo un trascinamento. Il segno si esaurisce quando il
menu è aperto; `Esc`, o `m` sul ref segnato, lo toglie prima. Il comando della
palette `Refs: Rilascia il ref selezionato sul branch attivo...` apre il menu per
il ref selezionato e il branch corrente.

## Il pannello destro

### Modifiche in corso

- **Clicca** un file per aprirne il diff. **Clicca di nuovo lo stesso file** per
  tornare al Log. Se il file è aperto nell'editor con modifiche non salvate,
  GitPoulpi chiede prima.
- **Passa il mouse** su una riga per mostrare il suo pulsante **Stage del file** o
  **Unstage del file**. Una riga di cartella ha **Stage della cartella** o
  **Unstage della cartella**.
- **Clicca Stage di tutto** o **Unstage di tutto** sul titolo di una sezione.
- **Clicca il titolo di una sezione** (`File non in stage`, `File in stage`) per
  comprimerla.
- **Clicca Percorso o Albero** per scegliere un elenco piatto o cartelle. **Clicca
  Comprimi tutto** o **Espandi tutto** per comprimere ogni cartella.
- **Clicca `✗`** a sinistra del titolo per scartare tutte le modifiche (chiede
  prima). **Clicca `✦`** a destra per spiegare le modifiche con l'IA.
- **Clic destro su un file** per il suo menu: stage, scarta, ignora, LFS, stash del
  file, cronologia, blame, apri in un programma esterno, copia il percorso, crea
  una patch, modifica, elimina. Un file in conflitto inizia con le righe **Risolvi
  con ...**.
- **Clic destro su una riga di cartella** per il menu della cartella: stage,
  scarta, ignora, stash, crea una patch, apri la cartella, crea un file al suo
  interno.
- **Clic destro nella parte vuota** di File non in stage per **Crea file**.
- **Ignora**, **Tratta come binario** e **Tratta come testo** (menu di file e cartella) aprono prima una lista dei file `.gitignore` o `.gitattributes` in cui può andare la regola. Clicca una riga e premi `Enter` (o usa le frecce); `Esc` o un clic fuori annulla senza scrivere nulla.
- **Ctrl+clic** su una riga di file o di cartella la aggiunge alla selezione o la toglie, **Shift+clic** (**Alt+clic**) prende un intervallo; lo stesso nei file modificati di un commit e in Vedi tutti i file. Un clic semplice ricomincia. **Clic destro su una riga selezionata**: il menu di tutti i file selezionati.
- **Clic destro sulla parte vuota** di una lista offre anche **Apri la cartella del repository**; File non in stage aggiunge **Stage di tutto** e **Scarta tutte le modifiche...**, File in stage aggiunge **Unstage di tutto**.
- **Converti** (menu di file, cartella e selezione) riscrive codifica, interruzioni di riga o rientro dei file su disco; un `Ctrl+Z` annulla il lotto.

### Il modulo di commit

- **Clicca** il Riepilogo o la Descrizione per scriverci.
- **Clicca Modifica il commit precedente** per attivare o disattivare l'amend.
- **Clicca Commit delle modifiche** per fare il commit. Quando il pulsante è grigio,
  un clic ne dice il motivo (per esempio *Metti in stage delle modifiche per fare il
  commit*).
- **Clicca `✦`** a destra del Riepilogo per scrivere il messaggio con l'IA.
- **Clicca `#▾`** alla fine del Riepilogo per l'elenco delle issue. Cliccane una per
  inserirne il riferimento.
- **Clicca Componi commit con l'IA** per aprire il compositore di commit.

### Dettagli del commit

- **Clicca** un file modificato per aprirne il diff. **Cliccalo di nuovo** per
  tornare al Log.
- **Trascina** il bordo inferiore del messaggio per ridimensionarlo. La rotella
  sopra il messaggio fa scorrere una descrizione lunga.
- **Clic destro** sul riepilogo, sulla descrizione o sul blocco dell'autore per *Modifica
  il riepilogo...*, *Modifica la descrizione...* o *Modifica l'autore...*; **doppio clic** su uno di essi (o sulla data dell'autore) per
  aprire la finestra di modifica completa con il cursore in quel campo. Gli equivalenti da tastiera sono `Ctrl+G Shift+E` (tutti i campi) e `Ctrl+G Shift+A` (l'autore); riepilogo e descrizione sono nella palette.
- **Clic destro** su un file modificato, su una riga di cartella, sulla parte vuota della lista o su una selezione per il suo menu (apri, cronologia, blame, apri in un programma, copia percorsi, ignora, tratta come binario o testo, converti, ripristina dal commit, elimina). **Ctrl+clic** e **Shift+clic** selezionano più file.

Vedi [Stage e commit](staging-and-committing.md).

## Il diff e la Vista file

- **Clicca un pulsante** nella barra sopra il diff: Vista file, Vista diff, Blame,
  il selettore di colonna `File │ Tutti`, Modifica, `↑` `↓`, Inline, Affiancato, Hunk, WS, A capo, Mappa, Codifica. Un pulsante
  attivo è evidenziato. Un clic su `File` o `Tutti` mostra quell'elenco nella colonna dei
  commit; un clic sul segmento già scelto la nasconde (vedi
  [La colonna dei commit](interface-tour.md#la-colonna-dei-commit)).
- **Clicca `✕`** nel titolo per chiudere il file.
- **Passa il mouse su un hunk** per mostrare **Stage dell'hunk** o **Unstage
  dell'hunk**, e **Scarta l'hunk**.
- **Passa il mouse su una riga modificata** per mostrare i suoi pulsanti **+ Stage**,
  **− Unstage** e **✗ Scarta** all'estremità destra della riga.
- **Clic destro su una riga** per il menu dell'hunk: stage, unstage o scarta l'hunk
  o le righe selezionate, cronologia del file, blame, copia.
- **Clicca la minimappa** per portare la vista su quella riga. Tieni premuto il
  pulsante e muovi per scorrere.

Ogni scarto chiede prima. Quando uno scarto elimina file non tracciati, vanno via con loro anche le cartelle che restano completamente vuote (mai una che contiene ancora un file tracciato, ignorato o nascosto, né una che era già vuota); Annulla riporta i file con le loro cartelle.

### Blame e cronologia del file

- **Blame.** Lascia il puntatore su un'annotazione per vedere la scheda dell'hunk; un clic seleziona un hunk, un clic destro apre il suo menu. Trascina il bordo della colonna di annotazioni per ridimensionarla (salvato). Clicca o trascina sulla minimappa per scorrere.
- **Cronologia del file.** Un clic seleziona un commit; `Shift+clic` seleziona un intervallo da confrontare; un doppio clic mostra il commit nel Log; un clic destro apre il menu della riga; trascina il bordo tra elenco e diff per ridimensionarlo; la ✕ sull'elenco lo nasconde (clicca la striscia `›` per riportarlo); clicca `Vista file` / `Vista diff` / `Blame` della barra per cambiare ciò che mostra il lato destro, e il selettore `File │ Tutti` per scegliere l'elenco della colonna. La rotella scorre la metà sotto il puntatore. Ognuno ha il suo equivalente da tastiera ([Blame](log-and-history.md#blame), [Cronologia del file](log-and-history.md#cronologia-del-file)).

### La colonna dei commit (elenco Tutti)

- **Passa il mouse su una riga** per il suggerimento del Log (il messaggio intero, nome ed e-mail dell'autore, la data completa, secondo la parte sotto il puntatore).
- **Clic destro su una riga** per il suo menu: mostrala nel Log, il diff del file in quel commit, il file in quel commit, il suo blame, poi il menu del Log per il commit. Il tasto menu fa lo stesso da tastiera.
- **Trascina un bordo dell'intestazione** (`ui.column_resize`: per impostazione predefinita la cella di separazione di una cella tra due parti, segnata da un `│` attenuato nell'intestazione e disegnata come linea d'accento lungo la colonna mentre il puntatore ci passa sopra, e un trascinamento da qualsiasi punto della linea ridimensiona; con `header`, la prima lettera del nome successivo, accesa quando il puntatore ci passa sopra) per dimensionare la parte alla sua sinistra: corsie, messaggio, autore, data, ref, modifiche o SHA. La cella subito dopo l'ultima parte è il bordo dell'ultima parte.
- **Doppio clic su un bordo dell'intestazione** per adattare la parte alla sua sinistra al suo contenuto, come fanno i bordi del Log: la larghezza che mostra intera la sua riga più larga, mai più larga della colonna (da tastiera: clicca una riga o `Ctrl+N` per dare i tasti alla colonna, `Tab` per scegliere la parte, poi `=`; `+` adatta ogni parte; la palette ha `Adatta la parte al contenuto` e `Adatta tutte le parti al contenuto`). Il messaggio tiene ciò che le altre parti lasciano.
- **Trascina un nome dell'intestazione** su un altro per spostare lì la parte: stesso fantasma, stessa origine attenuata e stessa barra d'accento dell'intestazione del Log, e la parte si sposta al rilascio (`Esc` annulla; i tasti `<` / `>`, dopo `Tab`, fanno lo stesso un posto alla volta).
- **Clic destro sull'intestazione** per scegliere quali parti mostrare (Ref, Grafo, Autore, Data, Modif., SHA; il messaggio è sempre visibile); l'elenco di caselle ha una ✕ nel titolo. Sotto: `Adatta <parte> al contenuto` (la parte cliccata) e `Adatta tutte le colonne al contenuto`. La palette ha `Scegli le colonne` (il menu da tastiera, sulla parte scelta con `Tab`) e un comando di attivazione per ogni parte.
- **Shift+rotellina** o una rotellina orizzontale scorre le righe in orizzontale quando sono più larghe della colonna.
- **Trascina il bordo destro** della colonna per ridimensionarla. **Clicca la striscia `›`** (o la ✕ sulla colonna, o **clicca il segmento scelto** del selettore `File │ Tutti` della barra) per riportarla o nasconderla.

### Selezionare e copiare il testo

Puoi selezionare e copiare il testo del diff, della Vista file, di Blame e di
Cronologia del file.

- **Premi e trascina** per selezionare il testo. La selezione resta sullo stesso
  testo quando scorri.
- **Doppio clic** per selezionare una parola.
- **Un semplice clic** non seleziona il testo. Sposta il cursore o seleziona la
  riga, e cancella la selezione del testo.
- **`y` o `Ctrl+C`** copia la selezione. Senza selezione, copiano la riga corrente.
- **Clic destro** nel diff per **Copia la selezione**, **Copia la riga** e **Copia
  l'hunk**.

GitPoulpi copia solo il testo del file. I numeri di riga, i segni `+` e `-`, le
intestazioni degli hunk e le colonne di Blame non vengono copiati. In Split, una
selezione resta sul lato da cui hai iniziato il trascinamento.

Nel diff, un trascinamento seleziona anche le righe che attraversa, come fa `v`.
`Enter` fa poi lo stage di quelle righe. `Esc` cancella prima la selezione del
testo, poi quella delle righe, poi chiude il diff.

Selezionare del testo colora anche ogni altro punto in cui quel testo compare nel
diff e nell'editor.

## Campi di testo

Vale per ogni campo di testo: filtri, la barra di ricerca, la palette, il Riepilogo
e la Descrizione, i campi dei dialoghi e i valori delle Impostazioni.

- **Clic** per mettere lì il cursore.
- **Premi e trascina** per selezionare il testo.
- **Doppio clic** per selezionare una parola.
- Scrivi per sostituire la selezione. `Backspace` o `Delete` la rimuovono.

I tasti per i campi di testo sono in [Scorciatoie da tastiera](keybindings.md).

## L'editor

- **Clic** per mettere lì il cursore. **Shift+clic** estende la selezione.
- **Trascina** per selezionare. Trascina oltre il bordo superiore o inferiore per
  scorrere.
- **Doppio clic** per selezionare una parola.
- **La rotella** scorre senza spostare il cursore.
- **Clicca Salva, Annulla, Ripeti o Cerca** nella barra. In modalità cerca, **`↑`**
  e **`↓`** vanno al risultato precedente e successivo.
- **Clicca `✕`** per chiudere. GitPoulpi chiede prima quando ci sono modifiche
  non salvate.

## Immagini

Per il mouse nella vista immagine vedi [Immagini](image-view.md).

## Spostare e ridimensionare le finestre

Le finestre si possono spostare, e quelle più grandi ridimensionare:

- **Trascina la barra del titolo** per spostare una finestra (la croce di chiusura conserva il suo clic). La palette dei comandi e i selettori non hanno una barra del titolo separata: trascina il bordo superiore.
- **Trascina il bordo sinistro, destro o inferiore**, oppure un angolo in basso (la maniglia `◢`), per ridimensionare una finestra. La finestra resta sullo schermo e non diventa mai più piccola del suo minimo (60 x 16 per la Cronologia annullamenti, meno per le altre). Una finestra ridimensionata usa lo spazio: le aree di testo e gli elenchi crescono, il testo va a capo alla nuova larghezza.
- Il trascinamento continua quando il puntatore esce dalla finestra, e rilasciare fuori non la chiude mai.
- **Trascina i divisori** all'interno della vista Reflog / Lavoro perso della Cronologia annullamenti; il doppio clic su uno lo reimposta.
- Gli equivalenti da tastiera sono `Ctrl+N` poi `m` (le frecce spostano, `Maiusc`+frecce ridimensionano, `0` reimposta, `Invio` o `Esc` escono) e `Alt`+frecce per i divisori; la palette ha `Finestra: Sposta o ridimensiona` e `Finestra: Reimposta posizione e dimensione`. Agiscono sulla finestra in primo piano. `Ctrl+N` funziona allo stesso modo su ogni superficie: mostra un badge sui controlli della finestra, del dialogo o del popup in primo piano (campi, elenchi, schede, pulsanti e la croce di chiusura), il tasto che segue preme quel controllo, `m` avvia la modalità di disposizione e `Esc` annulla; la barra di stato indica `<badge>`, `m=arrange` e `Esc`. Vale anche per la palette dei comandi, i selettori, la ricerca nei workspace e la guida in sovrimpressione, dove `Ctrl+N` non scende più nell'elenco. `Alt+G` (`palette.arrange`) è un altro modo, valido su ogni dialogo e popup, per disporre quello in primo piano senza il prefisso.
- **Un'unica impostazione** (`ui.dialog_geometry`, Impostazioni > Aspetto > Disposizione) vale per tutte le finestre: ognuna ricorda la propria posizione e dimensione per questa sessione, su disco per il prossimo avvio, oppure mai.

| Dialogo | Sposta | Ridimensiona |
|---|---|---|
| Cronologia annullamenti / Reflog / Lavoro perso, Registro attività, pannello dei job, selettore di cartelle | sì | sì |
| Palette dei comandi, selettori di scheda / tema / lingua, ricerca nei workspace | sì | sì (l'elenco cresce con l'altezza, le righe con la larghezza) |
| Clona (l'elenco dei repository), Pull request e Nuova issue (la descrizione), «Cosa viene inviato» (il pannello), Mermaid (l'anteprima), opzioni di revisione (i campi, solo la larghezza) | sì | sì |
| Editor di testo lungo delle Impostazioni (modelli, hook, prompt IA), la casella IA del modulo del workspace | sì | sì (l'area di testo) |
| Riscrittura della cronologia: i due dialoghi di aiuto IA | sì | sì (la descrizione, il pannello inviato) |
| Informazioni, pannello degli aggiornamenti, controllo di sicurezza, conflitti previsti, «Oltre il limite IA», eliminazione di tag, file sensibili | sì | no (la dimensione segue il testo) |
| Init, Worktree, Sparse checkout, dialogo Flow, dialoghi del flusso delle issue, modulo del messaggio di commit (reword), modulo del Workspace, selettori di etichetta / milestone | sì | no |
| Conferme (ogni «Sei sicuro?») | sì | no |
| Dialoghi di Revisione e Board: i loro piccoli dialoghi (commento, termina, scelta, colonne) | sì | no |
| Popup della pagina Scorciatoie (conferma di ripristino, cattura del tasto), il popup della risposta IA | sì | no |
| Impostazioni: i risultati e la conferma dell'autenticazione del repository | sì | no |
| Riscrittura della cronologia: dialoghi di eliminazione, identità e domande | sì | no |
| Popup di spostamento di un file del compositore di commit, avvisi di merge | sì | no |

Non si possono spostare: le schermate a pagina intera (Impostazioni, Aiuto, le pagine Revisione e Board, che riempiono già la finestra), i menu contestuali e gli elenchi a discesa (restano agganciati a ciò che li ha aperti), i tooltip e i piccoli popup dentro una pagina (il modulo dei remote, l'editor di divisione del commit).

Il puntatore mostra una forma di spostamento sopra la barra del titolo e una di ridimensionamento sopra i bordi e gli angoli (terminali con `OSC 22`).

## Forma del puntatore del mouse

Alcuni terminali permettono a GitPoulpi di cambiare il puntatore del mouse: un
puntatore di ridimensionamento sopra i divisori, sopra un bordo di colonna del Log, del Radar e della colonna dei commit (anche mentre lo trascini), una mano sopra gli elementi
cliccabili. Funziona in kitty, foot, Ghostty, mintty e in una build di WezTerm con la PR a monte #8092 (la sequenza
`OSC 22`). Viene inviata solo quando la forma cambia, dentro tmux solo avvolta per
il passthrough (`allow-passthrough on`; kitty e foot la mostrano, Ghostty no), e mai in screen,
zellij o herdr. Il puntatore predefinito viene ripristinato all'uscita di GitPoulpi.
Negli altri terminali (Rio, Alacritty, Tabby, iTerm2, Terminal.app, Windows Terminal, xterm) il puntatore non cambia; vedi
[Scegliere un terminale](terminals.md#forma-del-puntatore-del-mouse-osc-22). Il divisore o il pulsante
sotto il puntatore si illumina in ogni terminale.

Lo stesso tipo di componente dà lo stesso puntatore in tutta l'app: i pulsanti di una revisione mostrata sopra il Radar (schede, la croce di chiusura, i pulsanti dei popup) e i pulsanti Azioni del Radar prendono la mano come ogni altro pulsante.

- **I pulsanti sono veri pulsanti**, in Impostazioni (Integrazioni: Modifica, Disconnetti,
  Rimuovi, Sfoglia, Elimina, i pulsanti dei frammenti dell'editor delle istruzioni...)
  come nelle finestre di dialogo: etichette con margine su uno sfondo pieno che si schiarisce sotto il
  puntatore, attenuate quando disabilitate, mai testo tra `[ ]`. Le uniche parentesi rimaste
  sono i segni delle caselle `[x]` / `[ ]`, che sono caselle di controllo e prendono la mano
  come ogni altro controllo.

- **La mano che indica** è sui controlli: pulsanti, caselle di controllo e interruttori, caselle combinate (in Impostazioni, nei pannelli delle issue e delle pull request, nel Radar, nelle finestre di dialogo), schede e le loro croci, i pulsanti `+`, link e riferimenti (`#12`, `!45`), intestazioni di gruppo e di cartella con la loro freccia di espansione, pillole dei ref, barre di scorrimento e i pulsanti Attività, job e versione della barra di stato. Una casella di controllo o un pulsante dentro una riga mantiene la mano.
- **La freccia** è sulle righe. Una riga si illumina per intero quando il puntatore è su di essa (la stessa luce in ogni elenco) e non è di per sé un controllo: le righe dei pannelli Refs, Worktree, Stash e Sottomoduli, il Log e la colonna dei commit (solo una pillola di ref prende la mano), i margini di un diff, i file modificati, gli elementi delle pull request, delle issue e del Radar, e le righe degli elenchi nelle finestre di dialogo e nei pannelli (clone, selettore di cartelle, Workspace, le schede della bacheca, il pannello Job, i selettori di assegnatario, etichetta e milestone, passaggi del rebase, file e commit della revisione, scelte dei menu e delle caselle combinate).
- **Il cursore di testo (I-beam)** è sulle caselle di input, su tutta la casella: le caselle di ricerca (ricerca globale, Impostazioni, palette dei comandi, Home), i filtri (Refs, file, Radar, Log delle attività, ricerca del pannello delle issue), le righe di testo, numero e area di Impostazioni (anche mentre le modifichi), i campi delle finestre di dialogo, le caselle dei commenti di revisione, il compositore di commit, il testo dell'editor di file e il risultato modificabile dell'editor di merge (i suoi pannelli di sola lettura mantengono la freccia, perché un trascinamento lì non seleziona nulla). Una regola copre anche il testo di sola lettura: dovunque un trascinamento del mouse possa selezionare del testo, il puntatore è l'I-beam. Sono il codice di un diff (la vista Diff, Inline e Split, e il pannello diff della revisione), la Vista file e il codice del Blame. I loro numeri di riga, segni e intestazioni degli hunk mantengono la freccia, e un pulsante su una riga mantiene la mano. Lo stesso vale per la conversazione di una issue o di una richiesta (anche la Panoramica della revisione e il suo blocco della bozza), il messaggio dei dettagli del commit, le Note di rilascio, il Log delle attività (entrambe le schede), il popup di aiuto, l'anteprima Markdown dell'editor dei commenti e i commenti, le note in sospeso e i suggerimenti dentro il diff della revisione: un trascinamento seleziona il testo (un semplice clic mantiene il suo significato: un link si apre, un pulsante agisce, una voce del Log delle attività o una discussione si apre o si chiude), `Ctrl+C` lo copia, `Esc` o un clic lo scarta. Una selezione segue il suo testo quando scorri. Anche il messaggio del commit è modificabile (il doppio clic lo corregge), e l'intero blocco si illumina leggermente.
- **Nei dettagli del commit** le righe di informazione sono controlli: clicca `commit: <id>` per copiare l'id breve, doppio clic sulla riga dell'autore o della data per correggerla (la mano e un lieve segno le indicano). La riga delle statistiche informa soltanto e mantiene la freccia.

I puntatori di ridimensionamento restano sui divisori e sui bordi di colonna, e la mano aperta compare solo mentre trasporti un ref.

## Copiare negli appunti

Le azioni di copia mettono il testo negli appunti di sistema della macchina su cui gira GitPoulpi
e, dove il terminale è noto per rispettarlo, lo inviano anche al terminale
(OSC 52). Via SSH, o senza display, viene inviato solo OSC 52: se il tuo terminale
non lo supporta (iTerm2 finché il suo permesso per gli appunti non è attivo, Terminal.app,
GNU screen), non viene copiato nulla e la riga di stato dice che il testo è stato solo inviato
al terminale. Vedi [Scegliere un terminale](terminals.md#appunti-azioni-di-copia).

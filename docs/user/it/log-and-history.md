[English](../log-and-history.md) · [Čeština](../cs/log-and-history.md) · [Deutsch](../de/log-and-history.md) · [Español](../es/log-and-history.md) · [Français](../fr/log-and-history.md) · **Italiano** · [日本語](../ja/log-and-history.md) · [한국어](../ko/log-and-history.md) · [Nederlands](../nl/log-and-history.md) · [Polski](../pl/log-and-history.md) · [Português (Brasil)](../pt-BR/log-and-history.md) · [Português (Portugal)](../pt-PT/log-and-history.md) · [Русский](../ru/log-and-history.md) · [Svenska](../sv/log-and-history.md) · [Türkçe](../tr/log-and-history.md) · [Українська](../uk/log-and-history.md) · [简体中文](../zh-CN/log-and-history.md)

# Lavorare con il Log

Il Log è il centro di GitPoulpi: l'intera cronologia come grafo, con ogni branch,
tag e stash. Questa pagina spiega come leggerlo, come cercarci dentro e tutto ciò
che puoi fare a commit e ref. Le aree sono nominate nel
[tour dell'interfaccia](interface-tour.md); i tasti esatti sono in
[Scorciatoie da tastiera](keybindings.md) e i gesti del mouse in
[Mouse e gesti](mouse-and-gestures.md).

![Il Log con un commit di merge selezionato: i suoi dettagli e i suoi file a destra](../images/log-graph.png)

## Leggere il grafo

Ogni riga è un commit, dal più recente. Un punto sta sulla corsia del commit; le
linee uniscono un commit ai suoi genitori; un merge mostra due linee che si
incontrano. Le pillole a sinistra nominano i branch e i tag di quel commit, nel
colore della corsia, con una sottile linea fino al punto. Un'icona di laptop
indica un branch locale, un globo uno remoto e `✓` il branch di cui hai fatto il
checkout. `+N` significa che altri ref puntano a quel commit; passaci sopra per
elencarli, oppure usa il menu qui sotto.

La prima riga sono le **modifiche in corso** (un cerchio tratteggiato: il tuo
lavoro non committato, unito al commit su cui poggia). Gli stash compaiono come
piccoli quadrati su una corsia, accanto al commit su cui sono stati creati.

### Colori, tinta di riga e avatar

I colori delle corsie vengono dal tema. *Impostazioni > Aspetto > Tema colori del
grafo* combina qualsiasi tema con qualsiasi tavolozza di corsie, e il selettore
della palette *App: Cambia tema colori del grafo...* (`Ctrl+N Shift+Y`) mostra
l'anteprima dal vivo di ogni tavolozza; vedi [Temi](themes.md). *Sfondo delle
corsie del grafo* impostato su **Tinta di riga** dipinge una tenue fascia del
colore della corsia dietro la colonna del grafo e una sottile barra sul suo bordo.
*Intensità della tinta di riga* (**Forte**, **Media** (predefinita), **Sottile**)
imposta quanto è sbiadita la fascia.

| Tavolozza di corsie Serie con tinta di riga | Tavolozza di corsie Material con tinta di riga |
|---|---|
| ![Il grafo del Log nel tema colori Serie con tinta di riga](../images/theme-serie.png) | ![Il grafo del Log nel tema colori Material con tinta di riga](../images/theme-material.png) |

Gli *Avatar degli autori* (un'immagine nel punto del grafo o nella colonna Autore)
sono disattivati di base. Attivarli invia a Gravatar un hash dell'indirizzo e-mail
di ogni autore, e interroga il tuo provider di hosting. *Fonti degli avatar* e
*Avatar di riserva*, nella stessa pagina, scelgono da dove vengono le immagini e
cosa si disegna quando non ce n'è una.

Se il tuo terminale non sa mostrare immagini, il grafo viene disegnato con
caratteri. *Rendering del grafo* e *Protocollo immagini* impongono l'uno o l'altro;
vedi [Risoluzione dei problemi](troubleshooting.md).

L'evidenziazione della riga selezionata o sotto il mouse parte dal cerchio del
commit. Le corsie alla sua sinistra mantengono lo sfondo normale.

### Colonne

Clic destro sull'intestazione per scegliere le colonne: **Ref**, **Grafo**,
**Messaggio** (sempre attiva), **Autore**, **Data**, **Modif.** e **SHA**. La
colonna facoltativa **Modif.** mostra, per ogni commit, `n file +aggiunte
-rimosse`. Trascina il bordo tra due intestazioni per ridimensionare una colonna;
la scelta viene ricordata, in celle del terminale. Con il valore predefinito `ui.column_resize = "line"` il bordo è lo spazio di una cella alla fine di una colonna: nella riga d'intestazione vi sta un piccolo segno e, mentre il puntatore è sullo spazio (sull'intestazione o in qualsiasi punto della tabella), una linea sottile d'accento viene disegnata su tutta l'altezza della tabella e un trascinamento da qualsiasi suo punto ridimensiona (il puntatore diventa un puntatore di ridimensionamento nei terminali che lo supportano). Con `"header"` non ci sono né segno né linea: il primo carattere della cella d'intestazione dopo il bordo si accende al passaggio e il trascinamento parte da lì, solo sulla riga d'intestazione (Impostazioni > Log e grafo > Ridimensionamento delle colonne). Le righe hanno lo stesso aspetto nelle due modalità: lo spazio fa parte della colonna alla sua sinistra, quindi la selezione e le tinte di riga lo attraversano. Ref, Grafo e le altre colonne mantengono la loro larghezza quando la finestra viene ridimensionata o ingrandita: solo **Messaggio** si allarga o si restringe (su una finestra molto stretta le altre colonne cedono temporaneamente spazio, Grafo per prima, e tornano alle tue larghezze quando la finestra si riallarga). Trascina la colonna Grafo fino alla larghezza minima, un solo punto di commit, e il grafo diventa compatto: ogni commit sta su un'unica corsia, uno sotto l'altro, senza linee di branch o di merge. Una cella più larga e l'albero torna.

Trascinare un'intestazione mostra un fantasma del suo nome che segue il puntatore e una barra d'accento dove la colonna andrebbe a finire; l'ordine cambia al rilascio e `Esc` annulla. Il selettore aperto dal clic destro è un elenco di caselle (`[x]` / `[ ]`) con una ✕ nel titolo; si raggiunge anche dalla palette (`Vista: Log: Scegli le colonne da mostrare`), si muove con le frecce e si attiva con `Spazio` / `Enter`.

Dalla tastiera, `Shift+C` (o `Vista: Colonna del Log: Scegli una colonna da spostare o ridimensionare` della palette) sceglie una colonna e la sua cella d'intestazione si illumina. Poi `Tab` / `Shift+Tab` scelgono un'altra colonna, `<` / `>` la spostano di un posto a sinistra o a destra tra le colonne mostrate (l'equivalente da tastiera del trascinare la sua intestazione), `]` / `[` la allargano o la restringono (l'equivalente del trascinare il suo bordo; l'ultima colonna sposta il proprio bordo sinistro), `0` riporta ogni larghezza all'automatico, e `Esc` o `Enter` esce. Nel frattempo le frecce continuano a muoversi tra i commit. L'ordine e le larghezze vengono ricordati come dopo un trascinamento e sono propri del Log: la colonna dei commit accanto a un diff ha un elenco suo.

*Adatta al contenuto.* Un **doppio clic** sul bordo di una colonna adatta la colonna alla sua sinistra al suo contenuto visibile più largo e alla sua intestazione (i caratteri larghi e le emoji sono contati per le celle che occupano). Nella disposizione adattata il Log riempie sempre il suo pannello, perciò una colonna si ferma a ciò che il pannello può darle una volta che **Messaggio** ha tenuto la sua parte e ogni altra colonna la sua larghezza minima: la riga di stato dice allora `Autore richiede 34 celle, ne entrano 20.` Lo stesso da tastiera: con una colonna scelta (`Shift+C`, `Tab`), `=` la adatta e `+` adatta ogni colonna; la palette ha `Vista: Colonna del Log: Adatta la colonna al contenuto` e `... Adatta tutte le colonne al contenuto`; il selettore del clic destro elenca `Adatta <colonna> al contenuto` (la colonna sotto il puntatore), `Adatta tutte le colonne al contenuto` e `Reimposta le larghezze delle colonne` sotto le sue caselle. **Messaggio** è la colonna flessibile e non ha una larghezza propria da adattare: prende lo spazio che le altre lasciano.

*Scorri lateralmente.* Per impostazione predefinita ogni colonna sta nel pannello. Nella disposizione **scroll** (la casella `Scorri lateralmente (le colonne mantengono le larghezze)` del selettore, Impostazioni > Log e grafo > Disposizione delle colonne, `Vista: Log: Attiva o disattiva lo scorrimento laterale delle colonne` della palette, oppure `s` mentre una colonna è scelta) le colonne mantengono le loro larghezze, **Messaggio** compreso (qui ha una larghezza propria, e adattarlo prende il suo riepilogo più largo), nessuna cede quando la finestra è stretta e la tabella può essere più larga del pannello. Una barra sul bordo inferiore la scorre: trascina il cursore, premi la traccia libera per portare il cursore sotto il puntatore, oppure usa le frecce alle estremità. Da tastiera `Shift+Sinistra` / `Shift+Destra` scorrono di quattro celle (la palette ha `Vista: Log: Scorri le colonne a sinistra` / `a destra`); la rotella orizzontale o `Shift`+rotella sopra la tabella fanno lo stesso, e scegliere una colonna (`Shift+C`, `Tab`) la porta in vista. Una tabella più stretta del pannello lascia spazio vuoto a destra, e la banda di una riga selezionata arriva fino al bordo. Ogni disposizione tiene le proprie larghezze (`ui.log_column_widths` e `ui.log_column_widths_scroll`); il primo trascinamento, tasto o adattamento nella disposizione scroll memorizza ogni colonna alla larghezza che ha, così si muove solo quella che cambi. Nella disposizione adattata un adattamento che non sta nel pannello lo dice e rimanda a questa disposizione.

*La maniglia dell'ultima colonna.* Anche l'ultima colonna ha una maniglia, sulla sua ultima cella (non il bordo del pannello, che ridimensiona ancora i pannelli): un trascinamento la sposta della distanza percorsa dal puntatore, e **Messaggio** prende la differenza. **SHA** è larga 8 celle per impostazione predefinita (le 7 cifre e quella cella di maniglia).

![Il Log con la colonna Modif. visibile: ogni commit elenca il numero di file e le righe aggiunte e rimosse](../images/log-changes-column.png)

### Autori e `.mailmap`

Quando il repository ha un `.mailmap` (il file di git che assegna un solo nome e una
sola e-mail a una persona che ha fatto commit con più identità), la colonna **Autore**
del Log, i dettagli del commit, Blame e la Cronologia del file mostrano l'identità
**canonica**, come fa `git log`. I commit in sé non vengono toccati. L'impostazione
`log.use_mailmap` (Impostazioni > Log e grafo, attiva di base) la disattiva per mostrare
i nomi salvati. La ricerca per autore corrisponde al nome e all'e-mail come mostrati,
quindi cercare il nome canonico trova tutti i commit della persona. La scheda
**Identità** del pannello della cronologia elenca i nomi e le e-mail della cronologia e
scrive per te le righe del `.mailmap`, vedi `history-rewrite.md`; da terminale:
`gitpoulpi mailmap list|check|add`.

## Cercare

- **Cerca** (`f`, il pulsante **Cerca** della barra degli strumenti, o
  *Navigazione: Cerca commit...* nella palette): cerca nell'intero messaggio del
  commit, nel nome e nell'e-mail dell'autore, nell'hash e nei nomi dei ref.
  Attenua ciò che non corrisponde e mantiene la forma del grafo. `Enter` /
  `Shift+Enter` saltano al risultato successivo / precedente.
- **Filtro dei Refs** (`/` nella barra laterale): restringe per nome gli elenchi
  di branch, tag e remoti.
- **Solo** (il tasto `o`, o il menu del ref, su un ref, una cartella o un remoto):
  mostra solo i commit dei ref in solo; metti in solo più ref per vederne l'unione.
  `Shift+O` termina il solo.
- **Nascondi** un ref (dal suo menu) per toglierlo dal grafo: la sua etichetta
  sparisce e con essa i commit raggiunti solo da lui (il commit attivo resta
  sempre, con la riga dell'area di lavoro). *Nascosti n* nel titolo di Refs li
  conta, **Mostra tutto** (`Shift+H`) li riporta. Il totale della riga di stato
  conta la cronologia visibile.

## Branch, tag e la barra laterale Refs

Doppio clic su un branch locale o su un tag per fare il checkout. Clic destro su un
ref (nella barra laterale, o sulla sua pillola nel Log) per il suo menu. Le voci
più utili:

| Voce | Fa |
|---|---|
| Mostra nel grafo | Scorre il Log fino al commit del ref (caricando prima la cronologia più vecchia, se serve) e fa lampeggiare la riga; la selezione resta. Un singolo clic su un ref fa lo stesso, a meno che *Un clic su un ref lo mostra nel grafo* non sia disattivato nelle Impostazioni. Un ref nascosto dal grafo (o da Solo) lo dice invece nella barra di stato |
| Checkout | Passa al branch o si scollega sul tag |
| Merge, Rebase su | Fa il merge di questo ref nel branch attivo, o il rebase del branch attivo su questo ref |
| Fast-forward di `<branch>` a `<current>` / di `<current>` a `<branch>` | Sposta in avanti un branch quando è un fast-forward; altrimenti è grigia con il motivo (`divergente, non è un fast-forward`) |
| Crea un worktree da... | Apre un worktree collegato in una scheda propria |
| Pull, Push, Fetch, Imposta l'upstream... | Operazioni di rete su quel branch |
| Nascondi / Mostra nel grafo, Solo | Solo visualizzazione; nascondono o mettono in solo un ref (nascondere toglie anche i commit raggiunti solo da quel ref) senza toccare il repository |
| Rinomina, Elimina | Elimina chiede prima; si può eliminare anche il gemello remoto |
| Copia il nome del branch, Copia il link del branch sul remoto | Negli appunti |

**Dove si trova un tag.** Un tag può stare su questo computer, su un remote, o su entrambi, e le righe TAG della barra
laterale (e la pillola del tag nel Log) lo dicono con un indicatore dopo il nome. Passa il mouse su un tag per
leggerlo a parole.

| Indicatore | Significato |
|---|---|
| freccia in su | Solo qui: nessun remote lo ha. Il suo menu offre *Push* |
| freccia sinistra-destra | Qui e su un remote, allo stesso commit |
| freccia in giù (riga attenuata) | Solo su un remote: non è in questo repository (mai scaricato, oppure eliminato qui). Il suo menu offre *Fetch di `<tag>` da `<remote>`* ed *Elimina `<tag>` su `<remote>`...*; nella palette `Git: Fetch del tag selezionato` |
| freccia su-giù | Anche su un remote, ma a un commit diverso (un push verrebbe rifiutato) |

Git non tiene ref di tracciamento remoto per i tag, quindi GitPoulpi viene a sapere cosa ha ogni remote dal tuo ultimo
**fetch** (qualsiasi fetch, anche quello automatico) e dai propri push ed eliminazioni di tag. Mentre navighi non si
interroga la rete; se qualcun altro ha cambiato i tag di un remote, fai un fetch per vederlo. Le stesse informazioni
sono in `gitpoulpi tag list` (campi `local`, `remote`, `differs_on`, `where`).

**Eliminare un tag.** Quando un remote ha il tag, il pulsante rosso **Elimina** apre un dialogo con tre righe numerate
(`1`-`9`, oppure Su/Giù ed `Enter`; `Esc` annulla), e la riga evidenziata si spiega qui sotto:

1. **Elimina in locale**: lo rimuove solo da questo repository. Parte subito, e il messaggio dice che il remote lo ha
   ancora. **Il prossimo fetch lo riporta** (git segue i tag del remote), a meno che tu non lo elimini anche sul
   remote.
2. **Elimina su `<remote>`**: lo rimuove lì per tutti (`git push <remote> --delete`). Chiede una conferma che nomina
   il remote. Il tag locale resta. Non si può annullare.
3. **Elimina in locale e su `<remote>`**: entrambe le cose, dopo la stessa conferma, quindi un fetch non può
   riportarlo. **L'annullamento ripristina solo il tag locale** (lo stesso oggetto tag); torna a essere solo locale e
   *Push* lo rimette sul remote.

![Il dialogo Elimina tag con le sue tre righe, Elimina in locale, Elimina su origin ed Elimina in locale e su origin, sopra la sezione Tag del pannello Refs, dove ogni tag ha un indicatore che dice se è solo locale o anche su origin](../images/tag-delete.png)

Con più remote, ognuno ha le proprie righe. Un tag solo qui mantiene la semplice conferma; un tag solo su un remote va
direttamente alla conferma di quel remote. CLI: `gitpoulpi tag delete <name> [--local | --remote <r> | --both [--remote <r>]]`.

**Fetch e tag.** Un fetch aggiorna anche ciò che ha ogni remote (gli indicatori) e il normale seguire i tag di git
continua a valere: un tag eliminato solo qui viene scaricato di nuovo. Un tag che qualcuno ha **eliminato sul remote**
sparisce dal lato remote dell'indicatore dopo qualsiasi fetch, ma resta qui come tuo tag locale (ora *solo qui*),
perché git per impostazione predefinita non elimina i tag. **Impostazioni > Sincronizzazione > Prune automatico dei
tag** (disattivato per impostazione predefinita; `gitpoulpi fetch --prune-tags`) elimina anche un tag locale così, ma
solo uno che era stato sul remote: un tag che non hai mai inviato, o che hai spostato, non viene mai toccato. Richiede
git 2.41 o più recente per sapere cosa è stato eliminato.

**Nuovo branch o tag.** I pulsanti **Branch** e **Tag** della barra degli
strumenti, `Ctrl+G b` e `Ctrl+G t`, o *Crea un branch qui...* di un commit chiedono
il nome in una casella inline nella colonna Ref, sulla riga del commit di
destinazione. `Enter` lo crea, `Esc` annulla, e un nome che git rifiuterebbe viene
segnalato in rosso senza chiudere la casella.

Su un commit senza ref, passare il puntatore sulla sua riga mostra anche, nella colonna Ref, una pillola sbiadita con il nome del branch a cui appartiene (altrimenti il branch remoto), dedotto dalla cronologia; i tag non vengono mai mostrati come fantasma. Un branch eliminato dopo un merge compare comunque, barrato, con il nome indicato nell'oggetto del commit di merge (`Merge branch 'x'`, `Merge pull request #N from owner/x`...); squash merge, rebase e fast-forward non lasciano questa traccia. Passando sulla pillola se ne vede il tipo. Non crea nulla: usa il comando di creazione branch qui sopra. **Mostra un branch fantasma sul commit evidenziato** in Impostazioni > Log e grafo la disattiva; non viene disegnata dove la colonna Ref è troppo stretta.

**Remoti.** Il `+` nel titolo REMOTI ne aggiunge uno; clic destro sul nome di un
remoto per fare il fetch, copiare il suo link web, renderlo il remoto predefinito,
nasconderlo dal grafo, modificare i suoi URL o rimuoverlo. **Nascondere un remoto**
(l'occhio della sua riga, la voce di menu *Nascondi il remoto dal grafo*, o
`Remoto: Nascondi il remoto nel grafo...` nella palette) nasconde ogni branch che
ha ora o che avrà in seguito, e i commit raggiunti solo da lui escono dal Log: il
modo per sfogliare una cronologia senza quella di un remoto d'archivio non
correlato; lo riporti con *Mostra il remoto nel grafo*. Mostrare un remoto mostra
anche i suoi branch che erano stati nascosti uno per uno. Un branch di un remoto
nascosto lo dice sulla riga di stato se si usa il suo occhio: mostra prima il
remoto.

**Stash.** **Stash** mette in stash tutto (compresi i file non tracciati);
`Ctrl+G m` chiede prima un messaggio (con l'IA attiva, `Ctrl+G` dentro quel prompt
ne suggerisce uno). Clic destro su una riga di stash, nel Log o nella sezione
STASH, per **Pop**, **Applica** (mantiene lo stash) o **Elimina**.

**Worktrees.** *Crea un worktree da...* (su un branch, un tag o un commit) apre un
dialogo che propone una cartella accanto al repository, il nome facoltativo di un
nuovo branch e «Apri il nuovo worktree come scheda». I worktree compaiono nella
sezione WORKTREES; doppio clic su uno per aprirlo, e la sua scheda si chiude da
sola quando il worktree viene rimosso.

## Trascinare i ref sui commit

Puoi premere su un branch (nella barra laterale o sulla sua
pillola nel Log) e trascinarlo. Un'etichetta fantasma segue il puntatore e i
bersagli validi si illuminano; rilascia per ottenere un menu:

- un branch locale **su un altro branch locale**: Merge, Rebase, Rebase
  interattivo, Fast-forward;
- un branch locale **su un branch remoto**: Push verso quel remoto;
- un branch remoto **su un branch locale**: Merge, Rebase, Fast-forward, Pull.

`Esc` annulla un trascinamento. L'equivalente da tastiera è il comando della
palette *Refs: Rilascia il ref selezionato sul branch attivo...*.

**Merge o fast-forward.** Quando la destinazione non ha commit propri (il branch trascinato è semplicemente avanti), un
merge potrebbe limitarsi a spostare avanti la destinazione. Di base (Impostazioni > Git > **Merge fast-forward** = *never*,
o la scelta propria di un repository nella sua pagina Opzioni del merge) la riga del menu dice *Merge di x in y (merge commit)* e crea un vero
commit di merge, così la cronologia conserva il branch; usa la riga separata *Fast-forward di y a x* quando vuoi lo
spostamento semplice (per una linea retta, usa invece *Rebase*). Con *allowed* la riga Merge dice *(fast-forward)*, e con *only*
*(solo fast-forward)*, in grigio con il motivo quando i branch sono divergenti. Le stesse righe e le stesse etichette sono nel menu
del branch, e `gitpoulpi merge --ff | --no-ff | --ff-only` sceglie per ogni merge.

## Il menu del commit

Clic destro su un commit (o `.` su di esso) per il suo menu:

![Il menu del commit: cherry-pick, revert, reset, modifica del messaggio, spiegazione con l'IA, rebase interattivo, crea branch, tag, checkout, worktree, patch e azioni di copia](../images/context-menu.png)

| Voce | Fa |
|---|---|
| **Cherry-pick su `<branch>`** | Riapplica il commit sul branch attivo |
| **Revert del commit...** | Crea un commit che lo annulla. Su un merge commit, un menu a discesa chiede prima quale genitore è la linea principale (il lato che resta; il genitore 1 è il branch su cui è stato fatto il merge), poi la solita conferma. Annulla ripristina lo stato precedente |
| **Reset di `<branch>` a questo commit** | Un sottomenu per il tipo di reset; un reset hard chiede conferma |
| **Modifica il messaggio del commit...** | Vedi sotto |
| **Modifica...** | Cambia il messaggio, il nome, l'e-mail e la data dell'autore del commit in un'unica finestra; vedi [Modificare un commit](#modificare-un-commit-messaggio-autore-date) |
| **Dividi commit...** | Spezza il commit in più commit; vedi [Dividere un commit](#dividere-un-commit) |
| **Dividi commit (IA)...** | Lo stesso, con un raggruppamento proposto dal modello; solo quando l'[IA](ai.md) è attiva |
| **Spiega le modifiche (IA)** | Un popup con una spiegazione dell'IA; solo quando l'[IA](ai.md) è attiva |
| **Rebase interattivo da qui...** | Vedi sotto |
| **Crea un branch qui...**, **Crea un tag qui...**, **Crea un tag annotato qui...** | Casella inline per il nome |
| **Checkout di questo commit**, **Crea un worktree qui...** | HEAD scollegato, oppure un worktree |
| **Crea una patch da questo commit...**, **da qui a `<branch>`...** | Scrive file `.patch`; *Applica una patch...* sta nel menu di un ref |
| **Copia il link del commit sul remoto**, **Copia** > | Negli appunti (hash, messaggio e altro) |

## Modificare il messaggio di un commit (reword)

*Modifica il messaggio del commit...* (o `Ctrl+G e`) apre il messaggio completo: un
riepilogo di una riga con un contatore, una descrizione sotto, **Annulla** e
**Aggiorna il messaggio**. Si può modificare il messaggio di qualsiasi commit del
branch attivo (HEAD o un antenato); i commit sopra di esso vengono ricreati con i
propri alberi, autori e date, i merge restano merge e le tue modifiche in stage e
non in stage non vengono toccate. Se il commit è già stato inviato, il popup
avverte in ambra che servirà un force push (con lease) e chiede una seconda
conferma. `Ctrl+Z` (Annulla della barra degli strumenti) annulla il reword; vedi
[Annulla e ripeti](undo-redo.md). La barra di stato nomina anche la punta
precedente.

## Modificare un commit (messaggio, autore, date)

*Modifica...* nel menu del commit (o `Ctrl+G Shift+E`, o *Commit: Modifica...* nella palette) apre un'unica finestra con le proprietà del commit: il riepilogo, la descrizione e il nome, l'e-mail e la data dell'autore, ciascuno precompilato con i valori del commit. Cambia ciò che vuoi e premi **Applica**; vengono inviati solo i campi modificati. Una data si scrive `YYYY-MM-DD HH:MM:SS +HH:MM` e si mostra in UTC finché non la modifichi. Il committer sei tu, adesso (come in un reword). Qualsiasi commit del branch corrente può essere modificato, con le stesse regole, lo stesso avviso per un commit già pubblicato e la stessa seconda conferma di *Modifica il messaggio del commit...*; i commit sopra vengono ricreati con i propri autori e date, e le tue modifiche in stage e non in stage restano com'erano. `Ctrl+Z` (Annulla nella barra degli strumenti) annulla l'operazione. Nelle build con la riscrittura della cronologia, **Modifica la cronologia in profondità...** (`Alt+H`) in basso a sinistra nella finestra la chiude senza applicare nulla e apre il [pannello di riscrittura della cronologia](history-rewrite.md) con l'ambito da questo commit al più recente; se hai modificato un campo chiede prima conferma (`Esc` conferma). `Ctrl+N` mostra un badge su ogni campo e pulsante della finestra; premilo per portarvi il cursore o per premere quel pulsante (vedi [Badge](keybindings.md#badge)).

Ogni modo di aprire la finestra la apre completa, con tutti i campi visibili e modificabili; cambia solo la posizione del cursore. Nel pannello dei dettagli del commit (a destra), clic destro sul riepilogo, sulla descrizione o sul blocco dell'autore e scegli *Modifica il riepilogo...*, *Modifica la descrizione...* o *Modifica l'autore...*, oppure doppio clic sul riepilogo, sulla descrizione, sull'autore o sulla data dell'autore: la finestra si apre con il cursore in quel campo. Gli equivalenti da tastiera funzionano dal Log e dal pannello dei dettagli: `Ctrl+G Shift+E` (tutti i campi), `Ctrl+G Shift+A` (l'autore) e, nella palette, *Commit: Modifica...*, *Commit: Modifica l'autore...*, *Commit: Modifica il riepilogo...* e *Commit: Modifica la descrizione...*. Entrambe le scorciatoie si possono riassegnare (`git_prefix.amend`, `git_prefix.edit_author`; vedi [Scorciatoie da tastiera](keybindings.md)). Da uno script o da un agente: `gitpoulpi amend <commit>` con `--message` (o `--summary` / `--description`), `--author-name`, `--author-email`, `--author-date` e le opzioni `--committer-*` corrispondenti (lo strumento MCP è `amend`).

## Dividere un commit

*Dividi commit...* nel menu del commit (o la palette: *Git: Dividi il commit selezionato...*)
spezza un commit in due o più, dal più vecchio al più recente. Si può dividere qualsiasi
commit del branch attivo che non sia un merge (HEAD o un antenato, anche un commit radice).
L'editor di divisione ha due pannelli: a sinistra i **Nuovi commit** (ciascuno con il suo
messaggio) e a destra le **Modifiche** del commit, una riga per file e per hunk, ognuna con
un campo `[N v]` che indica il nuovo commit a cui va.

| Tasto | Mouse | Fa |
|---|---|---|
| `Su` / `Giù` (`k` / `j`) | clic su una riga | si sposta sulle modifiche (`Shift+Su` / `Shift+Giù` estende la selezione) |
| `1`-`9` | il campo `[N v]` della riga, poi una scelta | assegna la modifica selezionata a quel nuovo commit; `0` la lascia non assegnata |
| `Enter` / `m` | il campo `[N v]` | apre l'elenco dei commit per la modifica selezionata |
| `n` | `+ Nuovo commit` | mette la modifica selezionata in un nuovo commit |
| `v` | | **modalità righe**: divide un hunk riga per riga (trascina sulle righe per selezionare un intervallo) |
| `Tab` | clic su un pannello | passa dai commit alle modifiche e viceversa |
| `e` / `Enter` (nel pannello dei commit) | la matita | modifica il messaggio di quel commit |
| `d` / `Delete` | la croce | rimuove un nuovo commit (le sue modifiche diventano non assegnate) |
| `Alt+Su` / `Alt+Giù` | le frecce | sposta un nuovo commit in su o in giù |
| `s` | la casella | firma i nuovi commit |
| `Shift+R` | `Reimposta` | torna al piano iniziale |
| `Ctrl+S` | `Dividi in N` | divide |
| `Esc` | la croce, `Annulla` | esce; con modifiche in sospeso serve un secondo `Esc` per confermare |

Ogni modifica deve andare a un solo commit; l'editor conta quelle ancora *Non assegnate* e
il pulsante resta rifiutato finché non sono zero. Un file binario, che cambia solo la
modalità, che nel commit è nuovo o eliminato, un sottomodulo o un file LFS si può assegnare
solo per intero. Il primo commit parte con il messaggio originale; dai un messaggio a ogni
nuovo commit. Un nuovo commit mantiene l'autore e la data originali; chi ha fatto il commit
sei tu, adesso.

**Nel repository cambia solo il branch.** L'ultimo nuovo commit ha esattamente il contenuto
del commit originale, quindi i commit sopra di esso vengono ricreati con gli stessi alberi,
autori e messaggi (i merge restano merge) e le tue modifiche in stage e non in stage restano
dove sono: una directory di lavoro con modifiche va bene e non può esserci alcun conflitto.
Gli hook non vengono eseguiti. La divisione è bloccata finché c'è un auto-stash in sospeso.

**Firma.** La casella *Firma i commit* parte dal valore di `commit.gpgSign` del repository.
Se la togli, le firme dei commit riscritti (quelli nuovi e quelli sopra di essi) vanno perse,
e l'editor lo dice sotto la casella. Se la spunti, vengono firmati tutti; se la firma
fallisce, non cambia nulla.

**Annulla.** Un solo `Ctrl+Z` (Annulla della barra degli strumenti) rimette il commit
originale com'era, anche con la directory di lavoro modificata; vedi
[Annulla e ripeti](undo-redo.md). Un commit già inviato richiede poi un force push (con
lease).

**Dal rebase interattivo.** Ogni riga della revisione del rebase ha un piccolo pulsante
**dividi...** (`t` sulla riga): apre lo stesso editor per quel commit e, a divisione
conclusa (o annullata), torni alla schermata di revisione, sulla nuova cronologia. Se avevi
modificato il piano, premi dividi una seconda volta per confermare che quelle modifiche
vanno perse.

Per script e agenti: `split-source` e `split-commit` nel
[riferimento della CLI](cli-reference.md); un agente può anche aprire questo editor già
compilato perché tu lo riveda (`ui propose split`, vedi [MCP e agenti](mcp-and-agents.md)).

## Rebase interattivo

Scegli **Rebase interattivo da qui...** su un commit che è un antenato del branch
attivo, oppure trascina un branch su un altro e scegli *Rebase interattivo*. Il Log
viene sostituito da una schermata di revisione: una riga per commit, il più
recente in cima, ognuna con una maniglia di trascinamento e un menu a tendina
dell'azione.

![La schermata del rebase interattivo: quattro commit, il secondo impostato su Squash e il terzo su Drop, con i pulsanti Reimposta, Annulla il rebase e Avvia il rebase](../images/rebase.png)

| Azione | Tasto | Effetto |
|---|---|---|
| Pick | `p` | Mantiene il commit |
| Reword | `r` | Lo mantiene con un nuovo messaggio |
| Squash | `s` | Lo unisce al commit sotto, combinando i messaggi |
| Fixup | `f` | Lo unisce al commit sotto, scartando il suo messaggio |
| Drop | `d` | Lo esclude |
| Edit | `e` | Si ferma lì perché tu possa modificarlo |

Trascina una riga (o `Ctrl+Up` / `Ctrl+Down`) per riordinare. `a` mostra
l'anteprima dell'autosquash (i commit `squash!` e `fixup!` si spostano accanto ai
loro bersagli). Non succede nulla finché non premi **Avvia il rebase**
(`Ctrl+S`); **Reimposta** rimette il piano e **Annulla il rebase** (`Esc`) esce senza
cambiare nulla. Quando un passo si ferma su un conflitto o su un `edit`, un banner
sopra il Log offre **Continua**, **Salta** e **Interrompi**. Un intervallo che
contiene un commit di merge non può essere ribasato in modo interattivo; la voce di
menu ne spiega il motivo.

## Cherry-pick di più commit

`Ctrl+clic` attiva o disattiva un commit, `Shift+clic` (o `Alt+clic`) seleziona un intervallo,
`Space` attiva o disattiva quello corrente, `Esc` svuota. Clic destro su un commit
selezionato per **Cherry-pick di N commit su `<branch>`...**: si apre la stessa
schermata di revisione, con le stesse azioni e lo stesso riordino, e **Avvia il
cherry-pick** li riapplica in ordine.

## Copiare i commit come Mermaid gitGraph

![La finestra Copia come Mermaid gitGraph su tre commit selezionati: scelte di ambito, dettaglio e direzione, l'opzione fence e l'anteprima in tempo reale](../images/log-mermaid.png)

Seleziona dei commit nel Log (`Ctrl+clic`, `Shift+clic`, `Space`; oppure lascia semplicemente il cursore su uno)
e apri **Copia come Mermaid gitGraph...**: `Shift+Y` nel Log, il sottomenu **Copia**
del menu del commit, oppure `Log: Copia come Mermaid gitGraph...` nella palette. Una finestra mostra lo script
[Mermaid](https://mermaid.js.org/syntax/gitgraph.html) generato, in tempo reale e scorrevole; `Enter`
(o il pulsante **Copia**) mette negli appunti esattamente quel testo. Incollalo in un README, nella descrizione di una pull
request o su [mermaid.live](https://mermaid.live).

La finestra ha tre elenchi a discesa, una casella e l'anteprima (`Tab` si sposta tra loro; ogni
elenco si apre con `Enter`, `Space` o `Giù`):

- **Commit**: la selezione, oppure - quando è selezionato un commit e HEAD discende da esso - **Dalla
  selezione a HEAD**, la serie di commit tra i due (risposta dalla parte caricata del Log).
- **Dettaglio**: `structure` (commit, branch e merge; gli id sono SHA brevi; nell'interfaccia **Struttura**), `refs` (aggiunge i veri
  nomi dei branch come branch Mermaid, i tag come `tag:` e il commit HEAD come `type: HIGHLIGHT`; **Ref**),
  `subjects` (l'id del commit diventa `<sha> <oggetto>`, con escape e tagliato a `ui.mermaid_subject_width`
  caratteri; **Oggetti**) e `full` (aggiunge all'id l'autore e la data UTC; **Completo**).
- **Direzione**: da sinistra a destra (`gitGraph LR:`) o dall'alto in basso (`gitGraph TB:`).
- **Includi il blocco di codice ```mermaid** racchiude lo script in un blocco di codice Markdown.

Dettaglio, direzione e blocco di codice vengono ricordati (`ui.mermaid_detail`, `ui.mermaid_orientation`,
`ui.mermaid_fence`). Lo stesso generatore funziona dalla riga di comando:
`gitpoulpi log --format mermaid [--detail structure|refs|subjects|full] [--orientation lr|tb] [--fence]
[--range A..B | --limit N]` (vedi il [riferimento CLI](cli-reference.md)); il server MCP lo espone come
l'argomento `format` dello strumento `log`.

Il `gitGraph` di Mermaid è un linguaggio di riproduzione (commit, branch, checkout, merge), non una descrizione di
grafo, quindi il generatore fa alcune scelte che vale la pena conoscere:

- Un commit continua il branch del suo primo genitore selezionato; gli altri figli aprono un nuovo
  branch con il nome di un vero branch su di esso (`b1`, `b2`... a `structure`). Il primo branch è `main`
  (o il nome reale, tramite un'intestazione `mainBranchName`) e segue la catena dei primi genitori di HEAD.
- Un merge si unisce al branch dell'altro genitore. Quando quel branch è andato avanti dopo il genitore, il
  generatore crea un piccolo branch puntatore (`<branch>-<sha>`) al genitore e fa il merge di quello, così il
  merge punta comunque al commit giusto.
- Un genitore fuori dalla selezione viene scartato: un commit rimasto senza genitori selezionati apre una linea
  tutta sua, quindi i buchi in una selezione appaiono come linee scollegate.
- Un octopus merge (tre o più genitori) diventa una serie di merge; quelli intermedi hanno il
  suffisso ` (octopus 1/2)`, l'ultimo porta il vero commit.
- I cherry-pick non vengono rilevati; le virgolette doppie negli oggetti diventano apostrofi (Mermaid non
  può fare l'escape).

## Confrontare i commit

Seleziona due o più righe del Log: `Ctrl+clic` attiva o
disattiva una riga, `Shift+clic` (o `Alt+clic`, `Shift+Up/Down`,
`Alt+Up/Down`) prende un intervallo, `Space` attiva o disattiva la riga
corrente. Si possono selezionare commit, la riga **WIP** e le righe di **stash**.
Il pannello destro diventa un pannello **Confronto**:

- l'intestazione ha un pulsante **Inverti** e sotto di esso ogni riga
  selezionata è elencata nell'ordine del Log (la più recente per prima) come una
  **scheda** su due righe: l'avatar
  dell'autore (secondo le impostazioni degli avatar del Log: un'immagine nei
  terminali che le supportano, le iniziali altrove), il nome, quanto tempo fa,
  l'id breve e il riepilogo. La riga **WIP** è «Directory di lavoro / Modifiche
  non committate», uno stash mostra il glifo dello stash, `stash@{n}` e il suo
  messaggio. L'estremità più vecchia porta un'etichetta **Da** e la più recente
  un'etichetta **A** (Inverti le scambia); con tre o più righe una nota dice che
  le modifiche sono combinate. L'elenco delle schede occupa al massimo un terzo
  del pannello, poi scorre (rotella, barra di scorrimento, o trascina il
  divisore sotto per ridimensionarlo);
- clicca una scheda per portare il cursore del Log su quella riga e farla
  lampeggiare nel grafo; la selezione resta com'è, quindi `Space` poi la toglie
  dalla selezione;
- sotto, il solito elenco dei file modificati (vista a percorso o ad albero,
  filtro, ordinamento) elenca i file che differiscono tra loro. Clicca un file,
  o premi `Enter`, per aprire il suo diff al centro, negli stessi layout unificato
  o affiancato, con Blame, Cronologia del file e vista file sull'estremità
  più recente.

![Tre righe del Log selezionate: il pannello Confronto le elenca come schede con avatar ed etichette Da / A, sopra i file modificati](../images/compare-cards.png)

Ciò che viene confrontato dipende dalla selezione:

| Selezione | Confrontato |
|---|---|
| due commit (o uno stash e un commit) | il più vecchio con il più recente, nell'ordine del Log, qualunque sia l'ordine dei clic |
| la riga WIP e un commit o uno stash | quel commit con la directory di lavoro: modifiche in stage e non in stage dei file tracciati (i file non tracciati non fanno parte di un diff) |
| tre o più commit | le loro modifiche combinate: da subito prima del più vecchio al più recente (i merge seguono il primo genitore) |

Una rinomina appare come una cancellazione più un'aggiunta. **Inverti** (`x`)
scambia le estremità di un confronto a due righe; con la riga WIP, che è sempre
l'estremità più recente, o con un intervallo di tre o più, non c'è nulla da
invertire. Un confronto con la riga WIP segue la directory di lavoro mentre salvi i file.

Tastiera: `v` (o **Log: Confronta la selezione** nella palette dei comandi) porta
il focus nell'elenco dei file del confronto; `x` (o **Log: Inverti gli estremi
del confronto**) inverte; `n` (o **Log: vai alla riga confrontata successiva**) è
il gemello da tastiera del clic su una scheda: il cursore del Log va alla
successiva riga selezionata, ricominciando dall'inizio alla fine. `Esc` o un
semplice clic su una riga torna al singolo commit. La voce **Cherry-pick** conta
solo i commit della selezione; la riga WIP e gli stash vengono ignorati lì.

Gli stessi elenchi sono disponibili per script e agenti:
`gitpoulpi diff --from <rev> [--to <rev|worktree>] [--name-only]` (la CLI e lo
strumento MCP `diff`; `--to` è la directory di lavoro per impostazione predefinita).

## Conflitti e altri banner

Quando un merge, rebase, cherry-pick o revert si ferma su un conflitto, sopra il
Log compare un banner e i file in conflitto portano un `!` rosso nel pannello
Stato. Risolvili nell'editor di merge, poi usa **Continua** o **Esegui il commit
del merge** del banner, oppure **Interrompi** per tornare indietro. Vedi
[Risolvere i conflitti di merge](merge-conflicts.md).

Altri banner sopra il Log:

| Banner | Pulsanti | Palette |
|---|---|---|
| *Rebase di `<branch>` su `<commit>` · passo/totale · fermo a ...* | **Continua**, **Salta**, **Interrompi** | *Rebase: Continua / Salta / Interrompi* |
| *Applicazione delle patch (git am) · patch n di m* (un `git am` che si è fermato) | **Risolvi i conflitti...** (quando una patch va in conflitto), **Continua**, **Salta**, **Interrompi** (chiede prima) | *git am: Continua / Salta questa patch / Interrompi* |
| *Un index.lock rimasto (n s) blocca ogni modifica a questo repository* (un comando git è morto e ha lasciato `.git/index.lock`) | **Rimuovi index.lock** (chiede prima; offerto solo quando il file ha almeno 10 secondi) | *Repository: Rimuovi index.lock obsoleto* |
| *Auto-stash in sospeso: ...* | Vedi [Auto-stash](#auto-stash) | *Banner: ...* |

## Blame

Blame mostra l'intero file con numeri di riga e colori di sintassi, e una colonna di annotazioni a **destra** del codice (`ui.blame_side = "left"` la sposta a sinistra). Aprilo dalla barra di un file (`Blame`), dal menu del file, dalla palette o da una riga della cronologia (`b`). Dalla directory di lavoro le righe non ancora committate sono mostrate a parte (vedi Strisce).

- **Hunk.** Una serie di righe consecutive modificate per ultime da un solo commit è un hunk. La sua prima riga mostra l'avatar dell'autore (un'immagine se il terminale disegna immagini, altrimenti il badge con le iniziali, niente se *Avatar degli autori* (`ui.avatars`) è disattivato), l'autore, quanto tempo fa e il riepilogo del commit; le altre righe mostrano solo la barra colorata dell'autore. Una linea sottile separa gli hunk. Se un hunk inizia sopra lo schermo, la sua annotazione è ripetuta sulla prima riga visibile.
- **Strisce.** Accanto all'annotazione, la barra colorata dell'autore e una **striscia di calore dell'età**: calda per le righe recenti, fredda per quelle vecchie, rispetto alla riga più vecchia e più recente del file stesso (la legenda è nel titolo). Le righe non ancora committate sono punteggiate e dicono «Non ancora committato».
- **Scheda.** Lascia il puntatore un attimo su un'annotazione, o premi `i`, per la scheda di dettaglio dell'hunk: ID completo del commit, autore ed e-mail, data, riepilogo e le sue azioni. L'hunk selezionato è colorato.

  ![Blame con la minimappa attiva e la scheda aperta sull'hunk selezionato](../images/blame-card.png)
- **Azioni** (righe della scheda, menu contestuale su un hunk, tasti e `Blame: ...` nella palette): mostrare il commit nel Log (`Enter`), copiarne l'ID (`Shift+Y`), mostrare il diff del commit per questo file (`d`), Blame sul genitore del commit (`b`, seguendo le ridenominazioni), aprire la Cronologia del file (`Shift+H`).
- **Spostarsi.** `j` / `k`, `PageUp`/`PageDown`, `Home`, `Shift+G` e la rotella come ovunque; `n` / `p` hunk successivo / precedente, `Shift+N` / `Shift+P` hunk successivo / precedente dello stesso commit, `g` vai a un numero di riga.
- **Hunk alti.** Sotto il riepilogo, un hunk di tre o più righe mostra l'ID breve del commit e la data completa (attenuati), così la colonna non resta vuota.
- **Minimappa.** Con `ui.minimap` attivo (il pulsante `Mappa`, `M`), la minimappa del file sta accanto al codice, colorata con lo stesso calore dell'età della striscia; clicca o trascina lungo di essa per scorrere.
- **Larghezza.** Trascina il bordo della colonna di annotazioni, o premi `>` / `<`; la larghezza è salvata come `ui.blame_width`.

## Cronologia del file

Aprila dal segmento `File` del selettore `File │ Tutti` della barra del diff, da un menu del file o dalla palette (`Cronologia del file: ...`). Il centro si divide in due: a sinistra i commit del file, dal più recente, seguendo le ridenominazioni; a destra il diff del file per il commit selezionato, con tutte le modalità della vista Diff (hunk / file intero, inline / affiancato, diff per parola, minimappa, a capo).

- **L'elenco.** Due righe per commit. La prima: la lettera della modifica (A aggiunto, M modificato, D eliminato, R rinominato), il riepilogo e i `+aggiunte -rimosse` colorati. La seconda, attenuata: l'avatar dell'autore (esattamente ciò che la colonna Autore del Log mostra secondo *Avatar degli autori*), l'autore, la data relativa, l'ID breve del commit e `vecchio -> nuovo` sul commit che ha rinominato il file. Un elenco stretto toglie prima la ridenominazione, poi l'ID, poi la fine del nome dell'autore, ma mai l'autore né la data. Le pagine da 100 si caricano avvicinandosi alla fine; nel frattempo compare una riga `caricamento di altri...`.
- **Spostarsi.** `j` / `k`, frecce, `PageUp`/`PageDown`, `Home` / `End`, la rotella e un clic; il diff segue subito (i diff si leggono in background, in cache per riga, e un tasto tenuto premuto ne chiede uno solo). Una riga precedente a una ridenominazione è confrontata con il nome che il file aveva allora.
- **Diff / File / Blame.** I pulsanti **Vista file**, **Vista diff** e **Blame** della barra della vista scelgono ciò che il lato destro mostra per il commit selezionato: la sua **Vista diff**, la **Vista file** (il file com'era in quel commit) o il suo **Blame**, tutto dentro la divisione con l'elenco a sinistra, e il segmento `File` del selettore della colonna resta acceso. La modalità scelta resta quando cambi riga e il lato destro segue la selezione. Tasti `Alt+1` / `Alt+2` / `Alt+3` (nell'elenco anche `d` / `o` / `b`), da entrambe le metà, e le righe `Cronologia del file: ...` della palette fanno lo stesso; `Esc` torna al Diff.
- **La colonna.** L'elenco è la *colonna dei commit* che mostra il suo elenco **File**: come nasconderla, mostrarla, dimensionarla e usare il selettore `File │ Tutti` è descritto una volta sola in [La colonna dei commit](interface-tour.md#la-colonna-dei-commit). Un clic su `Tutti` lascia la Cronologia del file e tiene la vista sul commit selezionato.

  ![Cronologia del file con un commit centrale selezionato: il suo diff a destra](../images/file-history.png)

  ![Cronologia del file con il lato destro in modalità Blame, il blame del file al commit selezionato](../images/file-history-blame.png)
- **Confrontare due commit.** `Shift+Su` / `Shift+Giù` o `Shift+clic` selezionano un intervallo; il lato destro mostra il diff del file tra i due estremi, ciascuno con il proprio percorso. `Esc` scarta l'intervallo.
- **Focus.** `Tab` (o un clic) dà la tastiera al diff e la restituisce; il diff mantiene i propri tasti.
- **Azioni** (menu contestuale, tasti e `Cronologia del file: ...` nella palette): mostrare nel Log (`Enter`, doppio clic; il Log carica quanto serve), copiare l'ID (`y`) o il riepilogo (`Shift+Y`), mostrare il diff (`d`), il file a quel commit (`o`) o il suo blame (`b`) a destra, confrontare con la directory di lavoro (`c`, di nuovo per uscire), ripristinare il file a quella versione (`Shift+R`, con conferma).
- **Annulla.** Ripristinare una versione è un passo registrato: Annulla (`Ctrl+Z`) riporta i vecchi byte del file.
- **Divisore.** Trascina il bordo tra le metà, o `Alt+Right` / `Alt+Left`, `Alt+0` per reimpostare; la larghezza è ricordata per sessione.

## Fetch, pull e push

- **Pull** (`Ctrl+G p`) fa il pull come stabilito da
  *Impostazioni > Git e codifica > Pulsante Pull* (`git.pull_mode`: merge, solo
  fast-forward, rebase; un'impostazione per repository). Il suo `▾` esegue una volta sola *Fetch All*,
  *Pull (fast-forward se possibile)*, *Pull (solo fast-forward)* o *Pull (rebase)*,
  senza cambiare il pulsante. La modalità viene sempre passata a git in modo
  esplicito, quindi il `pull.rebase` della tua config git non può sorprenderti.
- **Push** (`Ctrl+G Shift+P`) invia il branch attivo al remoto predefinito,
  impostando l'upstream la prima volta. Il suo `▾` fa il push una volta sola verso
  un altro remoto a tua scelta. Se il remoto ha commit che tu non hai, un dialogo
  offre **Pull (merge), poi push**, **Force push (con lease)...** (mai un
  `--force` nudo) o **Annulla**.
- `Ctrl+G f` fa il fetch di ogni remoto. In *Impostazioni > Sincronizzazione*,
  *Auto-prune* decide se i fetch fanno il prune, e *Intervallo di auto-fetch
  (minuti)* fa il fetch in background.

Tutto questo è disponibile anche come `gitpoulpi fetch`, `pull` e `push` nella
[CLI](cli-reference.md).

## Auto-stash

Quando modifiche non committate bloccano un checkout, pull, merge o rebase,
GitPoulpi può metterle da parte per te: le mette in stash, esegue l'operazione e
poi fa il pop dello stash. *Impostazioni > Sincronizzazione > Auto-stash prima di
checkout, pull, merge o rebase* (`general.auto_stash`):

- **chiedi** (predefinito): un dialogo offre *Stash delle modifiche e continua*,
  *Fai sempre lo stash automaticamente* (lo stesso, e imposta
  `general.auto_stash = "always"` così non ti viene più chiesto; lo riporti com'era
  in Impostazioni > Sincronizzazione) o *Annulla*. La CLI e MCP falliscono invece
  e nominano `--autostash`.
- **sempre**: eseguito senza chiedere. **mai**: l'operazione viene rifiutata.

*Includi i file non tracciati nell'auto-stash* prende anche i file non tracciati.
Un albero pulito non vede mai uno stash. Lo stash, l'operazione e il pop sono
ciascuno annullabile.

Se lo stash non può essere rimesso a posto (va in conflitto con il risultato, o un
merge o rebase si è fermato su un suo conflitto), resta nell'elenco degli stash. Un
**banner sopra il Log** dice *Auto-stash in sospeso: le tue modifiche precedenti al
checkout sono in stash@{n}*. I suoi pulsanti:

| Pulsante | Fa |
|---|---|
| **Risolvi i conflitti...** | Apre l'editor di merge (solo dopo un pop in conflitto) |
| **Mantieni risultato** | Mantiene la directory di lavoro com'è e scarta la copia nello stash (solo dopo un pop in conflitto, quando nessun file è più in conflitto) |
| **Pop ora** (**Riprova pop** dopo un pop fallito) | Rimette a posto lo stash |
| **Mostra lo stash** | Mostra le modifiche dello stash |
| **Ricomincia** | Vedi sotto |
| **Drop** | Scarta lo stash (chiede prima) |

Ogni pulsante ha un comando della palette (*Banner: Pop dell'auto-stash in
sospeso*, *Banner: Ricomincia...*, ...). Nulla viene riprovato o scartato per te.

**Ricomincia** chiede prima. Poi:

1. conserva una copia dello stato attuale dei file messi in stash sotto
   `refs/gitpoulpi-backup/autostash-start-over-<time>`;
2. scarta le modifiche solo su quei file (rifiuta, nominandoli, se altri file hanno
   modifiche locali che potrebbero andare perse);
3. annulla l'operazione che ha fatto l'auto-stash, tramite il journal di annullamento
   (quindi quell'operazione deve essere l'ultima);
4. fa il pop dello stash sul commit originale e cancella il banner.

Se un passo fallisce, si ferma con lo stash intatto e dice dove. Finché il banner
è visibile e l'ultima operazione è quella che ha fatto l'auto-stash, **Annulla**
(`Ctrl+Z`, `gitpoulpi undo`) fa lo stesso, dopo una conferma. Gli script usano
`gitpoulpi autostash start-over [--dry-run] [--yes]` (codice di uscita 3 senza
`--yes`).

**Mentre un auto-stash è in sospeso, le operazioni che spostano la cronologia
vengono rifiutate:** checkout di un branch o di un commit, merge, rebase (anche
interattivo), pull, fast-forward, cherry-pick, revert, reword, composizione dei
commit con l'IA e ripeti. La finestra rende grigi questi pulsanti e queste voci
della palette e ne dà il motivo. La CLI e MCP ricevono un errore `PendingAutoStash`
(codice di uscita 1). Tutto il resto resta consentito: letture, stage, scarto,
commit, branch, tag, remoti, fetch, push, continua / salta / interrompi del rebase,
risoluzione dei conflitti, `stash push`, `reset` e `stash pop` / `apply` / `drop`.
Anche Annulla è consentito.

Quando l'auto-stash aspetta un merge o un rebase fermo sui conflitti, viene
ripristinato da solo non appena quell'operazione finisce (in GitPoulpi, oppure con
git puro seguito da un aggiornamento).

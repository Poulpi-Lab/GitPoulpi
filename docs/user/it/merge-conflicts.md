[English](../merge-conflicts.md) · [Čeština](../cs/merge-conflicts.md) · [Deutsch](../de/merge-conflicts.md) · [Español](../es/merge-conflicts.md) · [Français](../fr/merge-conflicts.md) · **Italiano** · [日本語](../ja/merge-conflicts.md) · [한국어](../ko/merge-conflicts.md) · [Nederlands](../nl/merge-conflicts.md) · [Polski](../pl/merge-conflicts.md) · [Português (Brasil)](../pt-BR/merge-conflicts.md) · [Português (Portugal)](../pt-PT/merge-conflicts.md) · [Русский](../ru/merge-conflicts.md) · [Svenska](../sv/merge-conflicts.md) · [Türkçe](../tr/merge-conflicts.md) · [Українська](../uk/merge-conflicts.md) · [简体中文](../zh-CN/merge-conflicts.md)

# Risolvere i conflitti di merge

Quando un merge, cherry-pick, revert o rebase si ferma sui conflitti, il Log mostra un
banner (*Merge in corso · feature in main · 2 file in conflitto*) e i file in conflitto
compaiono nel pannello Stato con un `!` rosso. Aprine uno nell'**editor di merge**: A (il lato corrente) e B (il lato
in entrata) affiancati, con il **Risultato**, il file che stai scrivendo, sotto. Questa pagina è la
versione breve; l'elenco completo dei tasti è in [Scorciatoie da tastiera](keybindings.md).

## Aprirlo

- `Enter`, o doppio clic, su una riga `!` del pannello Stato; il suo menu del clic destro
  inizia con **Apri nell'editor di merge**.
- Il pulsante **Risolvi i conflitti...** del banner (apre il primo file in conflitto).
- La vista Diff di un file in conflitto: il pulsante **Editor di merge** (`Shift+E`).
- La palette: **Conflitti: Apri l'editor di merge**.

Niente si apre da solo: un'operazione sospesa mostra solo il banner e le righe
`!`.

![Il Log di un repository fermo su un merge in conflitto: il banner Merge in corso con Risolvi i conflitti e Interrompi il merge, e due file in conflitto segnati con un ! rosso](../images/conflict-banner.png)

L'editor occupa l'intera larghezza della finestra: la barra laterale Refs e il
pannello destro sono nascosti finché è aperto e tornano quando lo chiudi (`F9`
li mostra o nasconde nel frattempo). A e B sono affiancati ogni volta che l'editor è
largo almeno `merge.side_by_side_min_width` colonne (100 per impostazione predefinita, Impostazioni >
Diff ed editor; `0` significa sempre); se è più stretto vedi un lato alla volta dietro delle schede. Le righe
lunghe scorrono lateralmente, A e B insieme (`Shift+←` / `Shift+→`, Shift+rotella o una
rotella laterale), e non vanno mai a capo.

![L'editor di merge: A (corrente) e B (in entrata) affiancati sopra il Risultato, conflitto 1 preso da A e conflitto 2 da B, conflitto 3 ancora da scegliere](../images/merge-editor.png)

## Scegliere

Ogni conflitto è un blocco incorniciato in A e in B, con una casella di controllo nell'intestazione e un
`+` / `✓` davanti a ogni riga.

| Per... | Tasto | Mouse |
|---|---|---|
| prendere la riga sotto il cursore (lato attivo) | `Space` | clic sulla riga |
| prendere l'intero blocco del lato attivo | `Enter` | clic su `[ ]` nella sua intestazione |
| prendere solo A / solo B per questo conflitto | `a` / `b` | clic destro sul blocco |
| prendere entrambi, A poi B / B poi A | `c` / `Shift+C` | clic destro sul blocco |
| non prendere nulla | `x` | clic destro sul blocco |
| scambiare l'ordine di due lati scelti | `s` | clic sul badge `1st` / `2nd` |
| prendere tutto A / tutto B (ogni conflitto) | `Shift+A` / `Shift+B` | barra degli strumenti |
| annullare la scelta di questo conflitto (torna non risolto) | `u` in A / B, `Alt+U` ovunque | clic destro sul blocco, **Annulla la scelta (non risolto)** |
| riportare l'intero file ai conflitti originali (chiede prima conferma se hai scritto nel Risultato; `Ctrl+Z` lo recupera) | `Shift+U` in A / B, `Alt+R` ovunque | **Ripristina** accanto a Prendi tutto |
| conflitto successivo / precedente | `F7` / `Shift+F7`, `Alt+Down` / `Alt+Up` (`n` / `Shift+N` in A / B) | `▼` / `▲`, o clic su un segno nella barra di scorrimento |
| un numero ripete un movimento | `3j`, `2k`, `2n`, `3N` (in A / B) | |
| file in conflitto successivo / precedente | `Alt+PageDown` / `Alt+PageUp` | `›` / `‹` nell'intestazione |
| mostrare o nascondere il pannello Base (antenato comune) | `Alt+D` (`d` in A / B) | `Base` |
| mostrare l'intero file in A e B (**File intero**) | `Alt+F` (`f` in A / B) | `File intero` |
| dare ad A o B più larghezza / tornare a larghezze uguali | `Alt+Right` / `Alt+Left`, `Alt+0` | trascina il divisore; doppio clic su di esso |
| scorrere A / B / Base lateralmente | `Shift+←` / `Shift+→` (`Shift+H` / `Shift+L`) | Shift+rotella, rotella laterale |
| mostrare o nascondere i pannelli Refs e Stato | `F9` | |
| dare il focus ad A, B, al Risultato | `F6` / `Shift+F6` | clic |
| saltare a un pannello, sul conflitto in cui sei | `Alt+1` A, `Alt+2` B, `Alt+3` Risultato, `Alt+4` Base; oppure `Ctrl+N` poi `1` / `2` / `3` | |
| passare al pannello vicino (A, Base, B, Risultato), stesso conflitto | `h` / `l` oppure `←` / `→` in A / B | |
| ciclare l'altezza del Risultato | `z` | trascina il suo titolo |
| annulla / ripeti (scelte e digitazione, un'unica pila) | `Ctrl+Z` / `Ctrl+Y` | |
| scrivere il file, lasciandolo in conflitto | `Alt+S` | **Salva** |
| scrivere il file e segnarlo come risolto | `Ctrl+S` | **Salva e segna come risolto** |
| chiudere | `Esc` | `✕` |

Ogni barra di scorrimento (A / B / Base, e il Risultato) porta un segno per conflitto:
arancione finché richiede ancora una scelta, verde una volta gestito, blu per quello corrente.
Un clic su un segno salta a quel conflitto (un segno sotto il cursore della barra di scorrimento è
già sullo schermo, quindi lì il clic trascina il cursore).

**File intero** mostra l'intero file in A e B con ogni differenza colorata, non solo i conflitti:

![Modalità File intero: l'intero file in A e B, con le modifiche colorate](../images/merge-editor-full-file.png)

Il Risultato è un vero editor: fai clic al suo interno e scrivi, seleziona, taglia e incolla, cerca con
`Ctrl+F`. Una scelta su un conflitto che hai modificato a mano sostituisce la tua modifica (`Ctrl+Z`
la recupera). Salvare con un conflitto in cui non è stato scelto nulla, o con una riga
che sembra ancora un marcatore di conflitto, chiede prima conferma.

### ✦ Suggerisci una risoluzione (IA)

Con l'IA attiva (Impostazioni > IA; vedi [Assistenza IA](ai.md)) la barra degli strumenti ha un
pulsante **✦ Suggerisci**, il menu del clic destro di un conflitto ha *✦ Suggerisci una risoluzione (IA)*, e
`Alt+I` e la palette (`IA: Suggerisci una risoluzione del conflitto di merge`) fanno lo stesso. Agisce
solo sul conflitto corrente e parte solo quando lo chiedi. La risposta sostituisce
la parte di quel conflitto nel Risultato come un'unica modifica, segnata come modificata (`✎`), così `Ctrl+Z`
rimette il conflitto com'era. Non viene mai salvata né messa in stage per te. Mentre
è in corso la riga dei suggerimenti mostra il tempo trascorso; `Esc` annulla, e un errore viene mostrato
anch'esso lì. Con l'IA disattivata nessuno di questi controlli esiste.

Dopo **Salva e segna come risolto** il file è in stage e l'elenco dei file in conflitto è
tornato; apri tu il file successivo. `Ctrl+Z` (o il pulsante Annulla)
annulla l'ultimo "segna come risolto". Quando non resta nessun file, il banner offre
**Esegui il commit del merge** (o **Continua** per un cherry-pick, un revert o un rebase) e
**Interrompi il merge**, che chiede prima conferma: rimette HEAD, l'indice e la directory di lavoro
com'erano.

Un merge il cui risultato è uguale a HEAD (hai preso il nostro lato ovunque) non ha nulla
in stage e nessun file modificato, eppure può comunque essere concluso: **Esegui il commit del merge**
apre il prompt di commit con il messaggio del merge, e il commit ha entrambi i
genitori. Allo stesso modo il pulsante Commit del compositore è abilitato senza nulla in stage
mentre un merge è sospeso.

Un cherry-pick o un revert vuoto (la modifica è già applicata) non può essere
committato: git lo rifiuta, e **Continua** lo dice e indica **Salta**.
**Salta** (pulsante del banner, voce della palette *Git: Salta cherry-pick o revert*, CLI
`conflicts skip`) chiede prima conferma, poi termina l'operazione senza registrare nulla
per essa; HEAD, l'indice e la directory di lavoro tornano a HEAD. Salta non è
annullabile (non si perde nulla che non sia stato abbandonato di proposito); un merge non
ha nulla da saltare.

## Conflitti che non sono testo

Un file binario, un puntatore Git LFS, un file oltre 8 MiB, un file eliminato da un lato,
un residuo di rinomina, un collegamento simbolico, un cambio di modalità o un sottomodulo non ha righe
da scegliere. L'editor mostra una **scheda** con ciò che contiene ogni lato e un pulsante per ogni
scelta; scegliere risolve il file e lo mette subito in stage.

| Scheda | Scelte |
|---|---|
| File binario, puntatore LFS, troppo grande | **Tieni A** (`a`), **Prendi B** (`b`), **Prendi base** (`o`, quando c'è), **Apri nello strumento di merge** (`m`) |
| Eliminato da un lato, modificato dall'altro | **Tieni il file** (`k`), **Accetta l'eliminazione** (`Shift+D`) |
| Residuo di rinomina (il percorso esiste da un solo lato) | **Tieni il file** (`k`), **Elimina il file** (`Shift+D`) |
| Collegamento simbolico, sottomodulo, modalità del file | **Tieni A**, **Prendi B**, **Prendi base** |

Quando il file è un'immagine (PNG, JPEG, GIF, WebP, BMP, ICO), i riquadri mostrano anche
le immagini A, B e Base, adattate e in sola lettura, in una finestra abbastanza larga per
contenerle; vedi [Immagini](image-view.md#nelleditor-di-merge).

![La scheda di un conflitto binario: cosa contengono A, B e la base, e i pulsanti Tieni A, Prendi B, Prendi base e Apri nello strumento di merge](../images/merge-editor-card.png)

`Esc` chiude la scheda senza scegliere. Lo strumento di merge esterno è
`general.external_merge_tool` in Impostazioni > Diff ed editor (`{base}`, `{local}` = A,
`{remote}` = B, `{merged}`; vuoto significa `git mergetool`); il file viene messo in stage quando lo
strumento termina con successo.

## Prevenzione dei conflitti

GitPoulpi può avvisarti *prima* del merge. Quando il branch su cui ti trovi andrebbe in conflitto con il branch da cui
è partito (la sua **base**), sulla riga del bordo superiore del Log compare una striscia: **In conflitto con origin/main
in 2 file**, con un pulsante **Mostra i file**. Occupa la prima riga di bordo libera dopo i banner delle operazioni in
corso, quindi finché ne è visibile uno può dover aspettare di avere spazio. Non viene unito nulla: il daemon fa un
merge in memoria dei due commit (libgit2, oppure `git merge-tree --write-tree` con git 2.38 o successivo; un git più
vecchio disattiva la funzione con una riga nel registro), quindi i tuoi file, l'indice e i ref restano intatti.

- **La base** è il branch di destinazione della pull request del branch quando GitPoulpi lo conosce già (la cache che
  tiene la sezione Pull request; per questo non viene fatta nessuna chiamata di rete), altrimenti il branch, tra i
  pattern dei **Branch base** (Impostazioni > Prevenzione dei conflitti), con meno commit dal punto di biforcazione. A
  parità vince il branch la cui punta è più vicina al punto di biforcazione, poi un branch di tracciamento remoto su uno
  locale, poi il nome. Il branch stesso e la sua copia remota non sono mai la base.
- **Il popup** (clicca la striscia, oppure usa la voce di palette **Conflitti: Mostra i conflitti previsti**, che
  funziona da tastiera) elenca i file e ha **Apri l'anteprima del merge** e **Rebase sulla base**. Sono le righe del menu
  del branch base (*Merge di <base> in <branch>*, *Rebase di <branch> su <base>*) eseguite per te: registrate nel
  journal, quindi `Ctrl+Z` le annulla, e un merge che va davvero in conflitto si ferma nell'editor di merge come ogni
  altro. Sinistra e Destra si spostano tra i pulsanti, Enter ne preme uno, Esc chiude.
- **Quando gira**: in background, circa 2 secondi dopo che il repository si è calmato (un commit, un checkout, un fetch
  che ha spostato un branch, un rebase), una volta per ogni coppia `(HEAD, base)`. La striscia sparisce appena HEAD si
  sposta e torna, o no, con la nuova risposta. Un HEAD scollegato non viene controllato.
- **Liste dei branch base**: da dove vengono i Branch base di un repository è una scelta nella sua pagina **Prevenzione
  dei conflitti**. **Predefinita** segue il valore globale (il `**` incorporato, ogni branch, finché non scegli un'altra lista in
  Impostazioni > Generali > Prevenzione dei conflitti); **Lista: <nome>** segue una delle tue liste con nome; **Lista propria**
  è una lista tutta sua. Le liste sono condivise: modificane una e ogni repository che la segue usa la modifica al controllo
  successivo, senza riavvio. **Salva come lista...** trasforma i pattern propri di un repository in una lista con nome,
  **Ripristina il valore predefinito** elimina la scelta del repository. Eliminare una lista che un repository segue lo fa
  usare la lista predefinita.
- **Disattivarla**: **Attiva il rilevamento proattivo dei conflitti** in Impostazioni > Generali, oppure per un solo
  repository nella sua pagina **Prevenzione dei conflitti**. Da uno script, `gitpoulpi conflicts check` stampa la stessa
  risposta in JSON ([Riferimento della CLI](cli-reference.md)).

## Agenti

Un agente può mostrarti un conflitto (`gitpoulpi ui show conflict <path> [--conflict
N]`: l'editor si apre su di esso, non viene scelto nulla) o **proporre** una risoluzione
(`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba` oppure `--content-file F`,
richiede *Lascia che gli agenti IA propongano modifiche da confermare*). Una proposta apre
l'editor con le scelte (o l'intero testo) già nel Risultato e un banner viola
*✦ Proposto da claude-code*. Controllala, cambia ciò che vuoi e premi
**Ctrl+S**: solo quello scrive o mette in stage qualcosa. Chiudere l'editor rifiuta la
proposta. Vedi [MCP e agenti](mcp-and-agents.md).

Da uno script, le stesse operazioni sono `gitpoulpi conflicts list | show | resolve |
unresolve | state | abort` ([Riferimento della CLI](cli-reference.md)); `conflicts check` li prevede prima di un merge.

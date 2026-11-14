[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · [Français](../fr/undo-redo.md) · **Italiano** · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · [Polski](../pl/undo-redo.md) · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Annulla e ripeti

**Puoi sempre tornare indietro.** Ogni modifica che GitPoulpi fa al tuo repository
viene registrata (fino a 100 azioni per impostazione predefinita,
`general.undo_depth`), e così ogni posizione che un branch ha avuto, comprese le
mosse fatte da altri programmi. Se sbagli, uno di questi strumenti riporta indietro il lavoro:

| Vuoi | Usa |
|---|---|
| annullare l'ultima cosa | `Ctrl+Z` (ripeti: `Ctrl+Y` o `Ctrl+Shift+Z`), oppure i pulsanti **Annulla** e **Ripeti** della barra degli strumenti |
| annullare più cose insieme, o una sola di molto tempo fa | la **Cronologia annullamenti** (`Alt+Z`, la freccia accanto ad Annulla, `Modifica: Cronologia annullamenti...`) |
| riavere un branch, tag, stash, file o hunk rimosso diverse azioni fa, **senza annullare ciò che hai fatto dopo** | **Ripristina solo questa** nella Cronologia annullamenti |
| riportare un branch dov'era, anche se un altro programma lo ha spostato | il **Reflog** del branch (menu del ref, `Reflog...`) |
| trovare i commit che nessun branch raggiunge più | **Lavoro perso** (`Modifica: Recupera commit persi...`) |
| annullare ciò che una shell o un **agente IA** ha fatto a branch, tag, stash o HEAD | lo stesso `Ctrl+Z`: anche quelle modifiche sono nella cronologia, segnate come **esterne** (vedi [Modifiche fatte fuori da GitPoulpi](#modifiche-fatte-fuori-da-gitpoulpi)) |

Dopo ogni azione che distrugge qualcosa e può essere annullata (scarto, eliminazione
di un file, di un branch o di un tag, hard reset, drop di uno stash, rimozione di un
remoto), la barra di stato lo dice: «scarta: fatto - Ctrl+Z per annullare», con il
tasto come lo hai assegnato. Il tooltip di Annulla dice quante azioni puoi
ripercorrere all'indietro.

`Ctrl+Z` annulla l'ultima cosa che GitPoulpi ha fatto al tuo repository. `Ctrl+Y`
(o `Ctrl+Shift+Z`) la ripristina. La barra degli strumenti ha i pulsanti **Annulla** e **Ripeti**,
la palette ha `Modifica: Annulla` e `Modifica: Ripeti`, e il tooltip di ciascun
pulsante dice cosa annullerebbe ("Annulla Commit \"fix typo\" (a81b9e4)").

![La barra degli strumenti dopo Stage di tutto: Annulla è acceso e il suo tooltip nomina l'operazione che annullerebbe](../images/undo-toolbar.png)

Ripeti non esegue mai di nuovo l'operazione. Ripristina lo stato che era stato registrato,
quindi un commit ripetuto è esattamente lo stesso commit, con lo stesso id e lo stesso
timestamp.

`Ctrl+Z` è inerte mentre un campo di testo ha la tastiera (il messaggio di commit, la
casella del filtro, l'editor, che tiene il proprio annulla del testo).

## La Cronologia annullamenti

`Alt+Z` (riassegnabile), la freccia accanto al pulsante Annulla o
`Modifica: Cronologia annullamenti...` apre un pannello con ogni azione registrata,
dalla più recente: un'icona per il tipo di azione (commit, branch, tag, stash,
reset, merge e simili, scarto, stage, configurazione, modifica esterna), che cosa
ha fatto nella tua lingua, quanto tempo fa, e il suo stato (**ripristinabile**,
**bloccata** con il motivo, oppure **annullata** e pronta da ripetere). Quando le
voci coprono più giorni, un'intestazione sottile (Oggi, Ieri, poi la data) apre ogni
giorno. L'intestazione del pannello le conta («12 azioni possono essere annullate
(fino a 100)»).

Le azioni che hai annullato stanno **sopra** l'azione più recente, sotto un
divisore («annullate - Ripeti le riporta indietro»), attenuate, con quella che
Ripeti riporta indietro per prima più vicina al divisore. La selezione parte
dall'azione più recente, non da loro. Le frecce si spostano (le intestazioni e il
divisore vengono saltati), `1` / `2` / `3`
(o `Ctrl+Tab` / `Ctrl+Shift+Tab`) passano tra le tre viste, `Esc` chiude.
`Invio` non esegue mai un'azione: ogni pulsante ha la sua lettera, mostrata nella
riga di suggerimenti sotto i pulsanti. Per la riga selezionata:

- **Annulla fino a qui** (`u`): annulla quell'azione e tutte le più recenti,
  dalla più recente, dopo un'unica conferma che le elenca tutte. Un passaggio che il
  daemon giudica rischioso chiede di nuovo prima di partire.
- **Ripristina solo questa** (`o`): riporta indietro ciò che quella singola azione
  ha rimosso e conserva tutto ciò che è stato fatto dopo. Un branch o un tag eliminato
  torna sul commit che aveva (se il suo nome ora è occupato ti viene chiesto un altro,
  con uno libero già inserito); uno stash scartato torna in cima all'elenco degli
  stash; un file scartato o eliminato viene riscritto al suo posto se da allora non
  è stato toccato, altrimenti accanto a esso come `nome.restored` (nulla viene mai
  sovrascritto); un remoto, un upstream, una riga di ignora o una riga di attributi
  rimossi vengono aggiunti di nuovo. Un reset, amend, rebase, merge o commit ottiene
  un branch `restore/<branch>-<sha>` sulla vecchia punta: il tuo branch corrente e i
  tuoi file non si muovono mai. La conferma mostra esattamente cosa farà, e il
  ripristino è una nuova azione: `Ctrl+Z` lo annulla.
- **Ripeti fino a qui** (`p`) su una riga annullata.
- **Ripristina main dove l'aveva lasciato Commit** (`d`, oppure `Alt+Shift+Z` /
  `Modifica: Ripristina il branch dove l'aveva lasciato l'azione bloccata` da fuori
  dal pannello): quando un altro programma ha spostato il branch e ha bloccato
  Annulla, il reflog sa ancora dove l'azione lo aveva lasciato.

Clic destro su una riga (o `.`) per le stesse azioni come menu.

![La Cronologia annullamenti dopo aver eliminato un branch e fatto due commit: la riga del branch eliminato è selezionata e può essere ripristinata da sola](../images/undo-history.png)

## Il Reflog

Il menu del ref di un branch locale ha **Reflog...**; `Modifica: Reflog...` mostra
quello di HEAD, e la Cronologia annullamenti ha una scheda per esso. Elenca le
posizioni che il branch ha avuto, dalla più recente: SHA breve, il motivo
registrato, quanto tempo fa, e chi, comprese le mosse fatte in una shell. Il
messaggio e i file modificati della posizione selezionata sono mostrati accanto
all'elenco.

Il reflog viene letto 200 posizioni alla volta. Uno più lungo continua da solo:
spostare il cursore (o scorrere) vicino alla fine dell'elenco carica la pagina
successiva, e l'ultima riga dice **... Carica altro** finché ce n'è una. **Carica
altro** (`m`, o `Reflog: Carica altro` nella palette) la carica subito e scrive «posizioni caricate: N» sulla
barra di stato. I repository che usano il formato reftable di git sono letti
come tutti gli altri.

![Il Reflog di main: ogni posizione che ha avuto, dalla più recente, con quella selezionata descritta a destra](../images/reflog.png)

**Mostra nel Log** (`l`), **Crea branch qui** (`b`), **Crea tag qui** (`t`),
**Applica alla directory di lavoro** (`a`), **Salva come stash** (`s`) (gli ultimi
tre funzionano come in Lavoro perso qui sotto) e
**Ripristina `<branch>` a questa posizione** (`r`). Il ripristino è annullabile.
Quando il branch è quello attivo e hai modifiche locali che non potrebbe spostare
senza sovrascriverle, un menu a discesa chiede cosa farne: mantenerle, soft, mixed
oppure hard. Hard scarta le modifiche ai tuoi file ma le salva prima, quindi
`Ctrl+Z` le riporta indietro (lo dice in rosso).

## Lavoro perso

`Modifica: Recupera commit persi...` (anche nel menu dei commit del Log) trova i
commit che nessun branch, tag o stash raggiunge: quelli che un reflog nomina ancora
(i commit di un branch eliminato), quelli che solo un pin di annullamento tiene in vita
e quelli orfani. L'elenco compare subito e la ricerca più lenta dei commit orfani si
completa mentre gira un indicatore di attività; non blocca mai l'interfaccia.

![Lavoro perso: i due commit di un branch eliminato (trovati tramite il reflog) e i commit orfani](../images/lost-work.png)

**Crea branch qui** (`b`) recupera un commit con un nome che scegli, **Crea tag qui**
(`t`) lo fissa con un tag (nome predefinito `restored-<id breve>`), **Cherry-pick**
(`c`) lo applica al branch corrente. Un commit perso non è nel Log, quindi questa vista
non ha «Mostra nel Log» (la vista Reflog lo tiene per i commit ancora raggiungibili).

**Recupera le modifiche** rimette ciò che il commit ha cambiato senza fare un commit:
**Applica alla directory di lavoro** (`a`) riapplica le modifiche del commit ai file,
tutto o niente (indice e HEAD restano; un commit di merge viene rifiutato). Quando non
si applica in modo pulito, nulla viene toccato e la finestra propone **Salva come stash**
(`s`), che conserva invece le stesse modifiche come nuovo stash; quell'azione è sempre
disponibile anche da sola. Entrambe sono registrate nel journal: `Ctrl+Z` rimette a posto
i file, o elimina di nuovo lo stash. Le stesse azioni sono `gitpoulpi apply-commit <rev>`
e `gitpoulpi stash from-commit <rev> [-m M]`, e la vista Reflog le ha anch'essa.

### Pannelli, hunk e la finestra

Reflog e Lavoro perso mostrano a sinistra l'elenco dei commit e, a destra, l'intestazione
del commit selezionato, **ogni file che ha cambiato** e gli hunk del file selezionato
(solo hunk, a capo nel pannello). L'elenco dei file è quello del pannello destro della
finestra principale: `Percorsi` o `Albero` (`v`, o clic sull'interruttore), una cartella
si apre e si chiude con `Invio`, `Sinistra` o `Destra`, un clic o le frecce scelgono un
file e i suoi hunk compaiono sotto (si chiede al daemon fuori dal thread
dell'interfaccia; gli ultimi file vengono tenuti). L'elenco dei commit, quello dei file
e gli hunk hanno ciascuno una barra di scorrimento che puoi premere e trascinare.

- `Tab` / `Shift+Tab` passano tra i pannelli (elenco, file, hunk); le frecce, `PageUp`,
  `PageDown`, `Home` e `End` agiscono sul pannello che ha la tastiera (il suo
  bordo è acceso). `Invio` su un file passa ai suoi hunk.
- **Divisori**: trascina quello tra l'elenco e il pannello destro, o quello tra i file e
  gli hunk; il doppio clic su un divisore lo reimposta. Da tastiera:
  `Alt+Sinistra` / `Alt+Destra`, `Alt+Su` / `Alt+Giù`, `Alt+0` reimposta entrambi.
  La palette ha `Reflog / Lavoro perso: Reimposta le dimensioni dei pannelli`,
  `Pannello successivo` e `Elenco file a percorsi o ad albero`.
- `Ctrl+N` mostra i suoi badge sulle tre schede delle viste e sui tre pannelli, come
  nella finestra principale; un tasto del badge salta lì.
- **Sposta e ridimensiona la finestra**: trascina la barra del titolo per spostarla,
  trascina i bordi sinistro, destro e inferiore o un angolo in basso (la maniglia `◢`)
  per ridimensionarla (mai sotto 60 x 16, sempre sullo schermo). La croce di chiusura
  conserva il suo clic e rilasciare fuori dalla finestra non la chiude mai. L'equivalente
  da tastiera è `Ctrl+N` poi `m`: le frecce spostano, `Maiusc`+frecce ridimensionano, `0`
  reimposta, `Invio` o `Esc` escono (il bordo lo dice finché è attivo). La palette ha
  `Finestra: Sposta o ridimensiona` e `Finestra: Reimposta posizione e dimensione`. Se la
  posizione viene mantenuta lo decide `ui.dialog_geometry` (*Impostazioni > Aspetto >
  Layout > Ricorda posizione e dimensione delle finestre*): `session` (predefinito),
  `saved` (ripristinata al prossimo avvio) o `never`.

## Dalla palette dei comandi

Ogni azione di riga delle tre viste è anche una riga della palette dei comandi, quindi il
pannello non richiede né il mouse né una lettera da ricordare. Premi `Ctrl+P` con il pannello
aperto: la palette si apre sopra di esso e le sue righe **Cronologia annullamenti:**,
**Reflog:** e **Lavoro perso:** agiscono sulla riga selezionata della vista visibile.

| Vista | Righe della palette |
|---|---|
| Cronologia annullamenti | `Cronologia annullamenti: Annulla fino a qui...`, `Ripeti fino a qui...`, `Ripristina solo questa...`, `Ripristina il branch dove si trovava...`, `Aggiorna`, `Chiudi` |
| Reflog | `Reflog: Mostra nel Log`, `Crea branch qui...`, `Crea tag qui...`, `Applica le modifiche alla directory di lavoro`, `Salva le modifiche come stash`, `Ripristina a questa posizione...`, `Carica altro`, `Aggiorna` |
| Lavoro perso | `Lavoro perso: Crea branch qui...`, `Crea tag qui...`, `Cherry-pick`, `Applica le modifiche alla directory di lavoro`, `Salva le modifiche come stash`, `Cerca di nuovo` |

Le righe sono grigie (con il motivo) finché la loro vista non è aperta; i tre comandi di
apertura, `Modifica: Cronologia annullamenti...`, `Modifica: Reflog...` e `Modifica: Recupera
commit persi...`, fanno anche passare a quella vista un pannello già aperto. Una riga che la
selezione non può eseguire ne spiega il motivo nella barra di stato invece di non fare nulla:
"questa azione è già annullata: usa Ripeti fino a qui", il motivo per cui una voce non può
essere ripristinata da sola, "si può riportare indietro solo un branch o HEAD", e così via. Le
righe con `...` aprono la stessa conferma o lo stesso prompt del nome del pulsante.

## Cosa si può annullare

| Area | Operazioni |
|---|---|
| Commit | commit, amend, reword, il compositore di commit con IA (tutti i suoi commit come un unico passaggio) |
| Branch e tag | creare, eliminare, rinominare, impostare o rimuovere l'upstream, creare un branch di tracking, creare / eliminare / annotare un tag, fast-forward, fare il checkout di una pull request come branch |
| Spostarsi | checkout di un branch o di un commit (compreso un HEAD staccato) |
| Cronologia | merge (fast-forward o merge commit), rebase (semplice e interattivo, una volta terminato), cherry-pick, revert, reset (soft, mixed e hard), la parte locale di un pull |
| Modifiche in corso | stage e unstage (file, cartella, hunk, righe, tutto), scarto (file, cartella, hunk, righe, un checkout di file), applicare una patch o le modifiche di un commit (`apply-commit`), **eliminare un file dal disco**, **ignorare** (file, estensione o cartella: la riga del `.gitignore` che hai scelto), **trattare come binario o testo** (la riga di `.gitattributes`) |
| Stash | push, apply, pop, drop, salvare le modifiche di un commit come stash (`stash from-commit`) |
| Remoti e configurazione | aggiungere, rimuovere, rinominare, cambiare l'URL o l'URL di push di un remoto, track e untrack di Git LFS |
| Su un server | **eliminare un branch o un tag su un remoto** (dal menu del riferimento, dalla palette, `branch delete-remote`, `tag delete-remote`, `tag delete --both`, o un finish del workflow che elimina il branch remoto): vedi sotto |

Mentre un merge, un cherry-pick o un revert è fermo sui conflitti, anche **Salva e segna come
risolto**, lo stage e l'unstage sono annullabili (ognuno viene annullato prima di
quello sotto di lui), e lo è anche **Interrompi il merge** (o il cherry-pick, il revert):
annullarlo riporta indietro l'intera operazione sospesa, con i suoi
conflitti (le tre versioni di ogni file irrisolto), i file già
risolti e in stage, i marcatori di conflitto nella directory di lavoro e il messaggio
del merge; ripetere interrompe di nuovo. Stage e risoluzioni smettono di essere annullabili una volta
terminata l'operazione (con commit o interrotta). *Unstage* e *Unstage di tutto* non
toccano mai un file ancora in conflitto: non ha alcuna modifica in stage da annullare.

Tutto ciò che uno scarto o un hard reset elimina viene prima conservato, quindi annullarlo
riporta indietro i byte esatti, compresi i file non tracciati, i bit di esecuzione e i
collegamenti simbolici. Annullare uno stash eliminato restituisce lo stesso stash, con lo stesso
id di commit (finisce in cima all'elenco degli stash, come `stash@{0}`, non nella sua vecchia
posizione). Annullare la rimozione di un remoto restituisce i suoi branch
di tracking remoto, il suo alias `origin/HEAD` e le impostazioni di upstream dei branch che
lo tracciavano.

### Eliminare un branch o un tag su un remoto

Prima dell'eliminazione GitPoulpi chiede al server cosa contiene in questo momento il branch o
il tag e lo registra: il remoto, il nome e il commit esatto (per un tag annotato, l'oggetto
tag stesso, così la sua annotazione viene conservata). L'eliminazione viene poi eseguita solo
se il server contiene ancora quel valore.

**Annulla lo rimanda** con lo stesso nome, sullo stesso commit, senza mai forzare. Poiché è
una modifica su un server che altre persone vedono, Annulla **chiede sempre prima** e lo
dice («Annullare questa operazione ricrea il branch topic su origin a 3f2a1c0...»); lo dicono
anche i passaggi della voce (`gitpoulpi undo --dry-run`). Usa le stesse credenziali di un push.
Il branch di tracking remoto torna insieme. **Ripeti** lo elimina di nuovo, con la stessa
sicurezza.

Annulla rifiuta, senza cambiare nulla, quando:

- il branch o il tag è stato **creato di nuovo sul remoto** da allora, o spostato: è su un
  altro commit rispetto a quello registrato, e un push sovrascriverebbe il lavoro di qualcun
  altro (il rifiuto nomina entrambi i commit);
- il commit (o l'oggetto tag) **non è più nel tuo repository** (è stato rimosso dal garbage
  collector): l'annullamento non si può fare.

«Elimina in locale e sul remoto» è **una sola voce**: Annulla ripristina insieme il branch o il
tag locale e quello remoto, Ripeti li elimina di nuovo entrambi. Eliminare solo il branch
remoto lascia stare quello locale; annullarlo fa lo stesso. Un finish del workflow che elimina
anche il branch remoto è annullabile nel suo insieme.

## Cosa non si può annullare, e perché

- **Quasi tutto ciò che è su un server**: push, invio di un tag, push LFS, blocco e sblocco,
  apertura di una pull request. Quello stato è condiviso con altre persone; annullare un push
  sarebbe un force-push o un'eliminazione sul loro lato, e GitPoulpi non lo farà al posto tuo.
  (Eliminare un branch o un tag remoto è l'eccezione: l'annullamento rimette soltanto ciò che
  il server conteneva, e rifiuta se lì qualcosa è cambiato.)
  Annulla non si blocca mai nemmeno a causa di una simile operazione.
- **Fetch**: sposta solo i riferimenti di tracking remoto, che rispecchiano il server. Il
  fetch successivo lo rifarebbe.
- **Interrompere un rebase**: l'annullamento non è offerto (un rebase sposta i branch
  e conserva un proprio stato; l'interruzione è essa stessa la via del ritorno). Interrompere un
  **merge, cherry-pick o revert** *viene* registrato, vedi sopra.
- **Creare un worktree**, **rimuovere un worktree** (ciò che ha salvato prima resta:
  il commit, lo stash; il branch eliminato insieme è registrato nel journal, quindi
  Annulla riporta indietro il branch, non la cartella), **creare un file patch**, clone, init, azioni di hosting
  (pull request, issue) e chiamate all'IA: non fanno parte della cronologia del
  repository.
- **Creare un nuovo file vuoto** dal menu del file, la rimozione automatica dei file
  `*.orig` dopo un merge e il salvataggio di un file nell'editor integrato
  non vengono registrati. Neppure la modifica del **comando SSH** (`core.sshCommand`), né quella del **checkout parziale** (non si perde nulla; *Disattiva* riporta ogni file).
- **Scrivere in `.git/info/exclude` o `.git/info/attributes`** (la scelta «solo questo clone» delle liste di ignora e di tratta come binario/testo): questi file sono fuori dalla directory di lavoro, quindi non vengono registrati.
- **Le modifiche ai file nella directory di lavoro fatte fuori da GitPoulpi** (un
  editor, uno script, un agente IA che scrive codice) non vengono mai catturate:
  non ne esiste alcuno snapshot, quindi un `git reset --hard` o `git checkout --
  file` fatto in una shell perde per sempre le modifiche non committate. Ciò che
  di un lavoro in shell *viene* catturato è ciò che vive nei ref: vedi la sezione
  seguente.

## Modifiche fatte fuori da GitPoulpi

Quando lavori con un agente IA (o con una shell accanto a GitPoulpi), la maggior
parte del lavoro git avviene fuori dalla finestra: commit, nuovi branch e worktree,
branch eliminati, reset, rebase, stash. GitPoulpi vede anche questi. Il daemon
tiene gli ultimi ref che ha visto (branch locali, tag, HEAD, l'elenco degli stash,
l'elenco dei worktree) e, ogni volta che il repository è cambiato e non per mano
di GitPoulpi, registra **una voce per ogni modifica coerente** nella Cronologia
annullamenti, segnata come **esterna**, con il motivo che git ha registrato
(`commit: fix login`, `rebase (finish)`, `reset: moving to HEAD~1`, `checkout:
moving from a to b`) e il nome che git ha registrato per chi l'ha fatta.
GitPoulpi non distingue una shell da un agente IA e non tira mai a indovinare: la
voce dice solo che è stata fatta fuori.

- **Una raffica è una sola voce.** Venti commit fatti tra due controlli sono una
  voce dal primo valore all'ultimo; un rebase è una voce, registrata quando
  finisce (nulla viene registrato mentre un rebase o un merge è fermo).
- **I ref eliminati vengono fissati.** Un branch, tag o stash sparito viene
  tenuto in vita sotto `refs/gitpoulpi/undo/` sul suo vecchio commit prima che
  qualcosa possa eliminarlo, così **Ripristina solo questa** (o `Ctrl+Z`) riporta
  il branch che l'agente ha eliminato, anche dopo il reflog e un garbage
  collection.
- **Mentre GitPoulpi era chiuso.** Lo snapshot è salvato accanto al journal
  (`.git/gitpoulpi/refs-snapshot.json`), quindi ciò che è successo mentre nessun
  GitPoulpi girava viene trovato al successivo avvio e segnato «mentre GitPoulpi
  era chiuso».
- **`Ctrl+Z` funziona su di esse come su qualsiasi voce**: i ref tornano
  indietro, un branch eliminato viene ricreato, uno stash scartato viene
  conservato di nuovo, un checkout esterno rifà il checkout del vecchio branch.
  Un branch attivo torna indietro come `git reset --keep`: i file seguono, e
  quando una delle tue modifiche locali è d'intralcio **rifiuta e non cambia
  nulla**. Ripeti la applica di nuovo. Anche **Ripristina solo questa**
  funziona: un branch eliminato torna e le azioni successive restano; un branch
  spostato torna come `restore/<branch>-<sha>`.
- **La barra di stato lo dice una volta**: «modifica esterna registrata - Ctrl+Z
  per annullare» quando arriva una nuova voce esterna e la finestra è in primo
  piano.
- Poiché la modifica esterna è ora la voce più recente, il vecchio rifiuto
  «bloccata da una modifica esterna» sparisce per lo più: annulli prima la
  modifica esterna.

Ciò che non viene registrato: gli spostamenti di soli fetch dei branch di
tracking remoto (rispecchiano il server), i ref nascosti di GitPoulpi, un worktree
aggiunto o rimosso senza alcuna modifica di branch (la voce del branch che porta
dice «worktree aggiunto <percorso>»; annullare sposta solo ref, non rimuove il
worktree) e tutto ciò che sta dentro i file. `general.journal_outside = false`
(Impostazioni > Generali, «Registra le modifiche fatte fuori da GitPoulpi») lo
disattiva: nulla di esterno viene registrato e una modifica fatta fuori blocca la
voce più recente, come prima.

## Quando Annulla rifiuta

Prima di agire, GitPoulpi controlla che tutto ciò che l'operazione ha toccato sia ancora
esattamente come l'operazione l'ha lasciato. In caso contrario **non cambia nulla** e dice cosa
si è spostato:

- `main` è a 3f2a1c0, non ad a81b9e4 dove l'aveva lasciato Commit (un commit fatto in una
  shell, per esempio);
- un file, l'indice, uno stash o una sezione di configurazione è stato modificato nel frattempo;
- è in corso un merge, un rebase o un cherry-pick;
- l'operazione non è mai terminata perché GitPoulpi è stato fermato mentre la eseguiva (la
  voce è segnata *interrotta*, non viene mai annullata automaticamente, e `gitpoulpi
  undo --list` mostra cosa stava per cambiare così puoi sistemarla a mano).

Annulla è rigorosamente last-in, first-out: una voce più recente bloccata blocca anche
quelle più vecchie, perché annullare un'operazione più vecchia sotto una più nuova che
è ancora in vigore sarebbe tirare a indovinare. Se rimetti a posto le cose da solo, la voce
torna valida. Le modifiche che non toccano nulla di ciò che una voce ha toccato (un commit
da shell su un altro branch, un fetch) non bloccano nulla.

### Una voce bloccata che non riesci a sbloccare: Dimentica

Quando la voce più recente è bloccata per sempre (il repository è andato avanti fuori da
GitPoulpi, oppure la voce non è mai terminata), non si può annullare nemmeno nulla di ciò che sta sotto.
Il tooltip di Annulla e il rifiuto lo dicono allora e indicano la via d'uscita:
**Modifica: Dimentica la voce di annullamento bloccata** nella palette (chiede prima conferma), oppure
`gitpoulpi undo-forget` da riga di comando. Dimenticare rimuove solo quella singola
voce dal journal; il repository non viene mai toccato, e la voce non può più essere
annullata né ripetuta. La voce sotto diventa allora la più recente.

Due casi si risolvono da soli:

- un merge o rebase in conflitto che concludi con git semplice: la voce viene
  chiusa alla successiva lettura dello stato del repository da parte di GitPoulpi;
- i file di un pop o apply di uno stash in conflitto che risolvi con **Salva e
  segna come risolto**: Annulla rimette il conflitto finché il file risolto è ancora
  in stage e non ne è stato fatto il commit.

## Conferme

Annulla chiede conferma solo quando è rischioso:

- il commit (o merge, rebase, cherry-pick, revert) che stai annullando è
  già sull'upstream del branch: annullarlo riscrive una cronologia che hai pubblicato,
  e dovresti fare il force-push del risultato;
- hai modifiche non committate e l'annullamento sposta la directory di lavoro: le tue
  modifiche vengono conservate, ma l'annullamento si ferma, senza cambiare nulla, se una di esse è
  in un file che deve riscrivere.
- la voce ha eliminato un branch o un tag su un remoto: annullarla **lo rimanda al server**
  (vedi sopra), quindi chiede sempre, e lo stesso vale per la ripetizione che lo elimina di nuovo
  lì.

Ripeti, e ogni annullamento di un semplice spostamento di riferimenti, non chiede mai: la direzione opposta
è a un tasto di distanza.

## Quanto indietro

`general.undo_depth` (Impostazioni > Generali) è il numero di operazioni conservate.
Il valore predefinito è **100**; `0` significa illimitato. Abbassarlo dimentica subito le voci
più vecchie. Iniziare una nuova operazione dopo un annullamento scarta ciò che avresti potuto
ripetere, come in qualsiasi editor.

La cronologia è conservata per worktree (un worktree collegato ha la sua), nella directory
git (`.git/gitpoulpi/journal.json`), quindi sopravvive alla chiusura del programma
e non fa mai parte di `git status` né di un commit.

## Disattivarla

`general.undo_enabled = false` (Impostazioni > Generali, "Annulla attivo") disattiva
l'intera funzione (e con essa la registrazione delle modifiche esterne). Non viene registrato né conservato nulla, quindi stage,
scarto e commit sono un po' più veloci. I pulsanti Annulla e Ripeti
scompaiono e `Ctrl+Z` dice "Annulla è disattivato".

**Disattivarla cancella la cronologia**: il file journal e i riferimenti nascosti
descritti qui sotto vengono eliminati, la prima volta che una qualsiasi richiesta raggiunge il
daemon mentre è disattivata. Riattivarla riparte da una cronologia vuota.

## Cosa viene memorizzato

Per essere certi che un annullamento non possa mai perdere dati, ogni commit, oggetto tag, stash o
snapshot di cui una voce potrebbe avere bisogno è tenuto in vita da un riferimento nascosto sotto
`refs/gitpoulpi/undo/` finché la voce è nella cronologia. Sono
esclusi dagli elenchi di GitPoulpi, ma `git log --all` o
`git for-each-ref` da shell li mostrano, e `git push --mirror` li invierebbe (usa
invece `git push --all`). Vengono eliminati quando la loro voce esce dalla
cronologia o quando Annulla viene disattivato. Nient'altro viene mai eliminato.

## Da riga di comando e MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` descrive i passaggi e non cambia nulla.
- `--list` stampa il journal, la voce più recente per ultima (`--all` aggiunge le voci che non sono
  più ripetibili ma sono ancora conservate), con l'id e lo stato di ogni voce.
- `--expect <id>` fa rifiutare la chiamata come *obsoleta* a meno che quella voce sia ancora
  la più recente: uno script (o una seconda finestra) non annulla mai qualcosa di diverso
  da ciò che ha guardato.
- `undo-forget` elimina dal journal la voce più recente bloccata (`--redo`: la voce
  di ripetizione più recente). Rifiuta quando quella voce non è bloccata, e termina con codice **3**
  senza `--yes`. Per MCP è una mutazione, ma non distruttiva: il
  repository non viene toccato.
- Codice di uscita **0** fatto, **2** rifiutato (il JSON ne indica il motivo, per esempio
  `RemoteRefMoved` quando un branch remoto eliminato è stato creato di nuovo), **3** rischioso
  e richiede `--yes` (annullare l'eliminazione di un branch o tag remoto, che fa un push verso il server).

Tornare indietro senza annullare tutto ciò che è venuto dopo:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (anche: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (anche: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` mostra, per ogni voce, che cosa **Ripristina solo questa** potrebbe
  riportare indietro (`restore.items`) o perché non può riportare nulla
  (`restore.not_restorable`), e chi l'ha fatta: `"source": "gitpoulpi"`, oppure
  `"source": "outside"` con `outside.message` (il motivo di git) e
  `outside.committer` per una modifica fatta da una shell o da un agente IA.
  `undo`, `redo` e `undo-restore` trattano le voci esterne come qualsiasi altra.
- `undo-restore <id>` riporta indietro ciò che quella voce ha rimosso (un branch o
  tag eliminato, uno stash scartato, un file scartato o eliminato, un remoto rimosso
  con i suoi branch di tracciamento e upstream, una riga di ignora) e conserva ogni
  operazione successiva. Non sovrascrive mai: un file modificato da allora riceve
  la versione conservata accanto a sé come `<nome>.restored`; un nome di branch già
  occupato è un rifiuto (uscita 2) che ne suggerisce uno libero, da passare con
  `--name`; dopo un reset, amend o rebase crea un nuovo branch sulla vecchia punta
  e non sposta mai il tuo branch. Il ripristino è una sola voce del journal, quindi
  `gitpoulpi undo` lo annulla.
- `reflog` elenca dove sono stati `HEAD` o un branch, con il motivo di git, comprese
  le mosse fatte da una shell. `reflog-restore` sposta un branch (o `HEAD`) su una di
  quelle posizioni; un branch attivo con modifiche non committate richiede `--mode`.
  Anch'esso è registrato nel journal.
- `lost-commits` elenca i commit che nessun branch, tag, remoto o stash raggiunge. Per
  riavere il lavoro di uno senza fare un commit: `apply-commit <rev>` (nella directory
  di lavoro, tutto o niente) o `stash from-commit <rev>` (come stash); entrambi sono
  registrati nel journal.
- Quando una mossa fatta in una shell blocca la voce più recente, il rifiuto porta
  con sé la posizione del reflog che contiene ciò che la voce si aspetta
  (`reason.RefMoved.restore`), da passare a `reflog-restore`.

Esistono due comandi perché le azioni sopra siano annullabili da ogni superficie:
`gitpoulpi rm-file <paths>` (elimina file dalla directory di lavoro) e
`gitpoulpi ignore <pattern> [--file <path/.gitignore>]` (aggiunge una riga a un
`.gitignore`; non succede nulla se la riga c'è già; annullare rimette il file
com'era, o lo rimuove se l'ha creato il comando, e rifiuta se il
file è stato modificato nel frattempo).

Il server MCP ha uno strumento per comando, quindi anche `undo`, `redo`, `rm-file` e
`ignore` sono strumenti. `undo`, `redo` e `rm-file` sono classificati come distruttivi
(soggetti a controllo come `reset`, vedi [MCP e agenti](mcp-and-agents.md)); `ignore` è una
normale mutazione. Il journal è un file nel repository, quindi `gitpoulpi
undo` in una shell annulla ciò che ha fatto la finestra, e la finestra può annullare ciò che ha fatto
uno script.

[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · **Italiano** · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# Il Radar

Il Radar è una scheda a sé, subito dopo Home: un unico elenco di ciò che ti aspetta in
**più repository**: le pull (o merge) request che hai scritto, quelle che chiedono la tua
revisione, le issue assegnate a te o create da te, e i repository in cui hai lasciato del
lavoro in corso. È la finestra Home con la vista Radar sopra, così che nulla sotto possa
rubare un clic o un tasto.

Richiede un profilo di host di codice (vedi [Integrazioni di hosting e issue
tracker](hosting-integration.md); gli elenchi di un repository sono in [Pull request e issue](pull-requests-and-issues.md), e la discussione di una riga si apre come [conversazione](conversations.md)) e dei repository collegati a esso, oppure un profilo con gli elenchi a livello di
account attivati (vedi «Oltre l'ambito» più sotto). Un repository non
collegato non viene mai interrogato: mostra invece una riga **Collega...** (vedi
«Repository con qualcosa da dire» più sotto).

![Il Radar su due repository: una richiesta fissata, poi In attesa della mia revisione, Le mie pull request, Assegnate a me, issue e lavoro in corso](../images/radar.png)

**Rivedi...** su una riga di pull o merge request (o `e`) apre il repository con la
revisione nell'app della richiesta ([Rivedere una pull request](review.md)).

## Aprirlo

- **`Alt+R`** (riassegnabile: `global.radar`), da qualsiasi punto.
- La palette: `Radar: Apri`.
- La scheda **Radar**, subito dopo Home nella barra delle schede.
- La riga **Radar** del menu **`+`**, finché la scheda è chiusa.

Da una scheda di repository ti porta alla finestra Home con il Radar aperto; `Esc`
o **Indietro** torna all'elenco dei repository, e il `⌂` fa lo stesso.

## Chiudere la scheda o disattivare il Radar

La scheda si chiude come una scheda di repository: con la sua **x**, con un clic centrale, con `Ctrl+W`
mentre è in primo piano o con la voce di palette `Radar: Chiudi scheda`. Home è l'unica scheda che non
si può chiudere. Una scheda chiusa resta chiusa per quella sessione e anche dopo un riavvio, finché non
riapri il Radar (`Alt+R`, `Radar: Apri` o il menu `+`): torna allora subito dopo Home. Chiuderla non
cambia nessuna impostazione.

Se non lo usi mai, disattivalo in **Impostazioni > Generali > Attiva il Radar**
(`general.radar_enabled`, vedi il [riferimento della configurazione](config-reference.md)). Se è
disattivato, la scheda, i suoi comandi della palette e `Alt+R` spariscono e GitPoulpi non carica nulla
per esso. Si applica subito, senza riavvio.

## Cosa copre: l'ambito

La casella combinata **Ambito** sceglie quali repository vengono interrogati:

- un'**area di lavoro**, per nome (i suoi repository, anche quelli senza una scheda
  aperta);
- **Schede aperte**: i repository aperti come schede di questa sessione;
- **Cartella di clonazione predefinita** e **Cartella di esplorazione predefinita**: ogni repository trovato nella
  cartella impostata nelle [Impostazioni](config-reference.md) (`repos.clone_root`, `repos.browse_root`), che abbia
  o no una scheda.

Si apre sull'ultimo ambito che hai scelto qui (conservato in `radar.toml`), finché
esiste ancora; prima della tua prima scelta, o quando quell'area di lavoro è stata
eliminata, sull'area di lavoro attiva della sessione, altrimenti sulle schede aperte.
La scelta vale solo per questa vista: non cambia l'area di lavoro attiva della
finestra. Quando due repository hanno lo stesso
nome di cartella, la cartella superiore viene aggiunta al chip per distinguerli.

### I due ambiti di cartella

Un ambito di cartella cerca i repository **su due livelli**: le cartelle direttamente al suo interno e le cartelle
dentro quelle che non sono repository a loro volta (`~/src/work/app`). Le cartelle nascoste vengono saltate, un
repository non viene mai attraversato e ne vengono elencati al massimo 200. La ricerca gira in background, mai
sul thread che disegna, e il suo risultato resta finché non aggiorni: il pulsante Aggiorna, `r`, `F5` (il tasto di aggiornamento della finestra, riassegnabile) o
`Radar: Aggiorna` cercano di nuovo nella cartella (un repository clonato nel frattempo compare).

L'hosting resta attivabile repository per repository: un repository della cartella **non collegato** a un host non
viene mai interrogato. Contribuisce solo con la riga del lavoro in corso, ha una riga `non collegato` con
**Collega...** sotto Repository (apre la sezione delle richieste di quel repository, dove scegli il provider e il
progetto), e la barra di stato li conta (`3 non collegati (solo lavoro in corso)`).

Una cartella non impostata è in grigio nell'elenco (`Cartella di esplorazione predefinita (impostala nelle
Impostazioni)`); sceglierla apre la sua riga nelle Impostazioni (Impostazioni > Generali) e lascia l'ambito
com'era. La cartella di clonazione conta come impostata grazie al suo valore predefinito per sistema operativo (la
cartella da cui parte il dialogo Clona) finché quella cartella esiste; la cartella di esplorazione non ha un
valore predefinito. L'elenco degli ambiti viene riletto dalle impostazioni a ogni apertura, quindi una modifica
fatta lì vale subito.

## L'elenco

| Gruppo | Cosa contiene |
|---|---|
| Fissati | Tutto ciò che hai fissato, di qualsiasi tipo, nell'ordine in cui l'hai fissato |
| In attesa della mia revisione | Richieste che chiedono la tua revisione (e che non hai scritto tu) |
| Le mie richieste | Le richieste che hai scritto; il chip dice `bozza` per una bozza |
| Assegnate a me | Richieste assegnate a te |
| Che mi menzionano | Richieste e issue aperte che ti menzionano e non sono già in un gruppo sopra (una issue non assegnata a te né creata da te compare qui solo quando ti menziona) |
| Altre richieste | Le altre richieste aperte |
| Issue assegnate a me | Issue aperte assegnate a te |
| Issue create da me | Issue aperte che hai creato |
| Lavoro in corso | Repository dell'ambito con modifiche non committate o commit non inviati |
| Rimandati | Elementi che hai nascosto per un po'; ripiegati finché non lo apri |
| Repository | Repository con qualcosa da dire (offline, token da rinnovare, non collegato) |

Le parole «pull request» e «merge request» seguono l'impostazione della terminologia
abituale e l'host.

Ogni riga mostra il tipo di elemento (`⇄` richiesta, `●` issue, `✎` lavoro in corso),
il chip del repository (nel colore dell'area di lavoro), il numero, il titolo,
l'autore, l'età e, dove c'è spazio, le etichette. Una riga di lavoro in corso mostra il
branch e ciò che vi attende: `✎` file modificati, `+` file nuovi, `↑` commit da
inviare, `↓` commit da scaricare.

I conteggi sono in ogni intestazione. All'interno di un gruppo, l'elemento aggiornato
più di recente viene per primo.

La barra di stato sotto il filtro mostra un indicatore di attività quando gli host
impiegano più di un momento a rispondere. Gli elenchi in cache (la stessa cache usata
dalla barra laterale) compaiono subito e vengono sostituiti man mano che arrivano le
risposte.

### Colonne

Il chip del repository, il `#` (numero e segni di controllo), l'autore e l'età sono **colonne ridimensionabili**. Lasciata in automatico, la colonna del repository è
larga quanto il nome più lungo a schermo (fino a 40 celle, e al massimo un terzo della finestra), così `owner/name` e il
`group/sub/project` di un elenco di account non vengono tagliati su un terminale normale. Il titolo prende ciò che resta.
Un nome che non entra termina con i puntini (`group/sub...`), e:

- **Fermando il puntatore** su un repository, un titolo, un autore o un elenco di etichette tagliati, un tooltip mostra il
  testo intero. Selezionando la riga, anche la barra di stato sotto il filtro riporta il progetto e il titolo completi,
  così la tastiera vede la stessa cosa.
- **Trascinando il separatore** (la cella subito dopo il nome del repository, dopo il `#`, o appena prima dell'autore o dell'età) si ridimensiona
  la colonna; con il **doppio clic** la colonna torna adattata. L'elenco ha una **riga d'intestazione** (repository, `#`,
  titolo, autore, età) e i separatori seguono `ui.column_resize` come quelli del Log: con `line` (il valore
  predefinito) un `│` attenuato sta nel separatore sulla riga d'intestazione e, mentre il puntatore è sul separatore
  (sull'intestazione o su una qualsiasi riga dell'elenco), la colonna è scelta con i tasti o stai trascinando, una linea sottile d'accento viene
  disegnata lungo tutto l'elenco; un trascinamento da qualsiasi suo punto ridimensiona. Quando rilasci, la linea scompare a meno che il
  puntatore sia ancora sul separatore (allora se ne va con il puntatore): un trascinamento del mouse non lascia mai una colonna scelta. Con `header` non ci sono né segno né linea: la prima
  cella dopo il separatore, sulla riga d'intestazione, si accende ed è l'unico punto da cui trascinare. Il puntatore diventa
  un puntatore di ridimensionamento sopra un separatore nei terminali che lo supportano. Il chip di stato ha una
  larghezza fissa, e così l'età (la più larga mostrata, finché non la dimensioni; un'età tagliata da una colonna stretta mostra la
  data completa in un tooltip), quindi l'autore inizia nella stessa cella su ogni riga.
- Oppure con i tasti: **`C`** sceglie una colonna (di nuovo, o `Tab`, per la successiva), **`]`** la allarga di due
  celle, **`[`** la restringe, **`0`** rimette ogni colonna in automatico, `Esc` esce. La palette ha le stesse voci:
  `Radar: Scegli colonna`, `Radar: Allarga colonna`, `Radar: Restringi colonna`, `Radar: Reimposta larghezze colonne`.

Le larghezze sono conservate in `radar.toml` (`[columns]`, `repo`, `number`, `author` e `age`, in celle) e restano dopo un riavvio. Sono
gli stessi gesti di ridimensionamento delle colonne del Log.

### Controlli e conflitti

Una riga di richiesta porta due piccoli segni dopo il numero quando l'host ha detto
qualcosa:

| Segno | Significato |
|---|---|
| `✓` | i controlli del commit head sono superati |
| `✗` | un controllo è fallito |
| `●` | i controlli sono in coda o in corso |
| `!` (accanto) | la richiesta è in conflitto con il suo branch di destinazione |

Appoggia il puntatore sui segni e un tooltip lo dice a parole («Controlli falliti»,
«Conflitti con main»); seleziona la riga e la barra di stato sotto il filtro dice lo
stesso, così lo vede anche la tastiera. Una richiesta senza segni è una su cui l'host
non ha detto nulla (nessuna CI configurata, o non richiesta).

- **GitLab** invia lo stato della pipeline e il flag dei conflitti insieme all'elenco
  delle merge request: ogni riga ha i suoi segni.
- **GitHub** no: l'elenco non ha né l'uno né l'altro. GitPoulpi li chiede per ogni
  richiesta (la richiesta stessa per il suo stato di merge, lo stato combinato del
  commit head e le sue check run: tre chiamate), per al massimo **15 richieste
  aperte per repository**, le tue per prime. Se GitHub rifiuta una di quelle chiamate
  (un limite di frequenza), le altre restano senza segni fino al prossimo
  aggiornamento. Le risposte vengono salvate con la cache di hosting.

- **Gitea e Forgejo** si leggono come GitHub: l'elenco porta il flag dei conflitti (`mergeable`), lo stato del
  commit viene chiesto per ogni richiesta (al massimo 15 per repository, le tue per prime), e le issue sono gli stessi
  gruppi. Le menzioni usano la ricerca del server (`mentioned_by`).

### Menzioni

**Che mi menzionano** e il filtro **Mostra: Che mi menzionano** usano ciò che l'host
sa sulle menzioni. GitHub viene interrogato tramite la sua ricerca (`mentions:`),
GitLab tramite la tua lista di cose da fare: contano solo le menzioni ancora **in
sospeso** lì, quindi una voce che hai segnato come fatta esce dal gruppo.

Le richieste chiuse e unite e le issue chiuse non sono elencate.

## Viste e filtri

Tre caselle combinate e un campo di testo restringono l'elenco. Ogni scelta è un menu
a discesa: cliccalo, oppure usa il suo tasto.

- **Vista** (`v`): Tutto, richieste, Issue, WIP (lavoro in corso), Rimandati.
- **Mostra** (`f`): Tutti, Miei (richieste che hai scritto, issue che hai create),
  Assegnati a me, Revisione richiesta, Che mi menzionano.
- **Ambito** (`w`): vedi sopra.
- **Testo del filtro** (`/` o `Ctrl+F`): corrispondenza fuzzy su repository, numero,
  titolo, autore, branch ed etichette. `Enter` ridà i tasti all'elenco e conserva il
  testo, `Esc` lo cancella.

`Ctrl+N` mostra un badge sul filtro, su ogni casella combinata, sui pulsanti
Aggiorna e Indietro e su ogni titolo di sezione sullo schermo. Premi il carattere
di un badge: il filtro riceve la tastiera, una casella combinata si apre, un
pulsante viene eseguito e un titolo riceve la selezione (`Enter` o `Space` la
comprime poi). `Esc` annulla. Vedi [Badge](keybindings.md#badge).

## Azioni

Seleziona una riga e usa il suo menu (clic destro, il `⋮` a fine riga, `.` o `Menu`),
un tasto, o la palette (`Radar: ...`). Un'azione del mouse ha sempre un tasto e un
gemello nella palette.

| Azione | Tasto | Palette |
|---|---|---|
| Apri repository (la sua scheda si apre, o viene attivata) | `Enter` / doppio clic | `Radar: Apri repository` |
| Rivedi... (una richiesta; una richiesta non clonata chiede come) | `e` | `Radar: Rivedi la richiesta...` |
| Rivedi una richiesta tramite l'API dell'host (senza clone) | `a` nella domanda | `Radar: Rivedi tramite l'API` |
| Clona il repository, poi rivedi la richiesta | `c` nella domanda | `Radar: Clona e rivedi...` |
| Apri nel browser | `o` | `Radar: Apri nel browser` |
| Checkout del branch (una richiesta) | `c` | `Radar: Checkout del branch` |
| Crea worktree... (una richiesta) | `t` | `Radar: Crea worktree...` |
| Mostra nel repository (nel suo elenco) | `s` | `Radar: Mostra nel repository` |
| Inizia a lavorare su issue... (una issue) | `n` | `Radar: Inizia a lavorare su una issue...` |
| Fissa in alto, o Non fissare più | `p` / clic sulla `★` | `Radar: Fissa o non fissare` |
| Modifica assegnatari... / revisori... / etichette..., Imposta milestone... (GitHub, GitLab) | menu della riga | `Radar: Modifica assegnatari...`, `Radar: Modifica revisori...`, `Radar: Modifica etichette...`, `Radar: Imposta milestone...` |
| Rimanda... | `z` | `Radar: Rimanda...` |
| Non rimandare più | `u` | |
| Mostra o nascondi le richieste dei repository non clonati | `a` | `Radar: Mostra le richieste non clonate` |
| Mostra o nascondi le bozze delle righe a livello di account | `d` | `Radar: Mostra le bozze` |
| Attiva le richieste di revisione a livello di account di un host | `T` / `Enter` sulla sua riga di suggerimento / **Attiva** | `Radar: Attiva le richieste di revisione dell'account...` |
| Aggiorna | `r`, `F5` | `Radar: Aggiorna` |
| Il menu della riga | `.` | `Radar: Menu della riga...` |
| Torna alla Home | `Esc` | `Radar: Torna alla home` |

Le azioni che richiedono la finestra del repository (checkout, worktree, mostra,
inizia a lavorare, collega, i selettori dei metadati) passano prima alla sua scheda e vengono eseguite lì
appena è caricata:

- **Checkout** usa il checkout della barra laterale: il branch quando è noto in
  locale, altrimenti la testa della richiesta recuperata in un branch locale (quella
  di un fork, per esempio).
- **Crea worktree...** apre la finestra Crea worktree sul branch della richiesta.
- **Mostra nel repository** seleziona l'elemento nell'elenco PULL REQUEST o ISSUE.
- **Inizia a lavorare su issue...** apre il prompt Nuovo branch con un nome ricavato
  dall'issue (`12-fix-the-thing`).

### Fissare e rimandare

Un **pin** tiene un elemento in alto. Un **rinvio** lo nasconde per **1 ora**, **4
ore**, **Fino a domani** (le 08:00 della mattina dopo, nel tuo orario locale, anche quando adesso non sono ancora le 08:00) o **Fino alla prossima modifica** (l'host segnala un
aggiornamento dell'elemento o, per una riga di lavoro in corso, i suoi contatori
cambiano). Entrambi sono conservati in `radar.toml` nella tua cartella di
configurazione, indicizzati per host, repository, tipo e numero, e sono condivisi da
ogni sessione sulla macchina.

### Repository con qualcosa da dire

Un repository con un problema ha una riga tutta sua in fondo all'elenco, e i suoi
elementi in cache restano sullo schermo:

- **offline, 5m fa**: l'host non è raggiungibile;
- **Riconnetti**: il token manca o è rifiutato; il pulsante apre Impostazioni > Integrazioni;
- **in pausa per questo giro**: l'host ha chiesto di rallentare (o ha rifiutato il
  token), quindi GitPoulpi non lo ha interrogato di nuovo per gli altri repository di
  questo giro;
- **non collegato, suggerito GitHub**: il remoto del repository corrisponde a un
  profilo, ma l'hosting è opt-in per repository, quindi non si chiede nulla finché
  non lo colleghi. **Collega...** ti porta alla sezione PULL REQUEST del repository,
  dove scegli il provider e il progetto.

Al massimo quattro repository sono interrogati contemporaneamente, e mai sul thread
che disegna lo schermo.

## Oltre l'ambito: righe a livello di account

L'ambito copre i repository che hai clonato. Una richiesta che aspetta la tua revisione in un repository che
non hai mai clonato lì è invisibile. Le **righe a livello di account** interrogano l'host stesso, una volta per
profilo, ed entrano nell'elenco qualunque sia l'ambito: riguardano te, non una cartella.

Sono **disattivate per impostazione predefinita** (la ricerca di un host costa quota API). Finché un host di codice
connesso non ha nessuno dei suoi elenchi attivo, in cima all'elenco compare una riga per lui:

`GitHub: elenca le richieste in attesa della tua revisione, anche nei repository che non hai clonato  [Attiva]`

Clicca **Attiva**, premi `Enter` su quella riga, premi `T`, oppure esegui `Radar: Attiva le richieste di revisione
dell'account...`: scrive `integrations.radar.review_requests = true` per quel profilo in `config.toml` e
interroga subito l'host. Gli altri elenchi (ogni richiesta aperta dei tuoi namespace, issue assegnate a te,
bozze, namespace) si impostano con `gitpoulpi hosting integrations radar` o nelle impostazioni del profilo; vedi
il [riferimento della configurazione](config-reference.md) e il [riferimento della CLI](cli-reference.md). GitHub,
GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center e Gerrit rispondono; Bitbucket Cloud non ha un endpoint a
livello di account, quindi le sue richieste compaiono solo per i repository collegati dell'ambito.

I **Namespace** del profilo limitano tutti i suoi elenchi ad alcuni repository (Impostazioni > Integrazioni > Radar: un'area di testo con una voce per riga e una casella **Espressioni regolari**). Una riga corrisponde al percorso completo di un repository quando è uguale o è un prefisso di percorso che termina a un `/`: `ptm` mantiene `ptm/produits/x` e scarta `ptmx/y` e `formation/y`. Con la casella attiva, ogni riga è un'espressione regolare eseguita sul percorso completo così come lo scrivi (`^ptm/(produits|infra)/`); una riga non valida viene mostrata sotto il campo e non viene salvata. Vuoto significa nessun limite.

Come vengono collocate le righe:

- Una riga che l'ambito elenca già (stesso host, progetto, tipo e numero) compare una sola volta: **vince la
  riga dell'ambito**.
- Una riga il cui progetto hai clonato da qualche parte (repository recenti, aree di lavoro, le cartelle trovate
  dagli ambiti di cartella; abbinato sull'URL del remote) porta quel clone: **Enter**, checkout, worktree e gli
  altri la aprono come qualsiasi riga.
- Ogni altra riga è **non clonata**: il chip del repository è attenuato e mostra il progetto (`acme/app`), e
  la riga di stato dice `Non clonati` quando è selezionata. **Apri nel browser** funziona, **Rivedi** chiede come
  rivederla (sotto), e checkout, worktree, mostra nel repository e inizia a lavorare dicono
  `clona prima il repository`.

### Rivedere una richiesta

**Rivedi** (`e`) su una richiesta dei repository dell'ambito apre la scheda di quel repository con la revisione
al suo interno. Su una richiesta di una riga a livello di account il modo dipende dal fatto che un clone sia noto e
dall'impostazione **Richiesta di un repository non clonato** del profilo (Impostazioni > Integrazioni, `review_mode`
nel [riferimento della configurazione](config-reference.md)):

| Impostazione | Un clone è noto | Nessun clone è noto |
|---|---|---|
| Chiedi (predefinita) | chiede: **Apri il clone locale** (`l`) o **Rivedi tramite l'API** (`a`) | chiede: **Clonalo, poi rivedi** (`c`) o **Rivedi tramite l'API** (`a`) |
| Clonalo | apre il clone | apre la finestra Clona |
| Rivedi tramite l'API | rivede tramite l'API | rivede tramite l'API |

La domanda è una piccola finestra: le lettere scelgono, **Enter** sceglie la riga evidenziata, `Esc`, **Annulla**,
la `✕` e un clic fuori la chiudono, e **Ricorda per GitHub** (`r`) scrive l'impostazione del profilo così la
domanda non viene più posta (la riga delle Impostazioni la cambia di nuovo). `Ctrl+N` mette un badge ai suoi controlli.

- **Rivedi tramite l'API** apre la revisione nella finestra Home, sopra il Radar, con il codice preso
  dall'host (vedi [Rivedere senza un clone](review.md#rivedere-senza-un-clone)). `Esc` o la `✕` restituisce
  il Radar; le righe *Revisione: ...* della palette funzionano su di essa.
- **Clonalo, poi rivedi** apre la finestra Clona sull'account collegato da cui viene la riga (lo stesso elenco di
  repository, raggruppato per proprietario), con il repository della richiesta selezionato e la cartella di clone
  predefinita già inserita; cambia l'una o l'altra, poi clona, con lo stesso accesso di qualsiasi clone da quell'account.
  Se il repository non è nell'elenco dell'account (non ne sei membro), l'elenco resta aperto con la ricerca sul suo nome e
  una nota lo dice; per un host che non può elencare i repository, la finestra si apre su **Clona con URL** con
  l'indirizzo già inserito. Quando il clone è arrivato, il nuovo repository viene collegato all'host
  (la sua sorgente PULL REQUEST e la sua sorgente ISSUE quando l'host ha issue e nessuna è impostata), la sua scheda si apre e
  la revisione di quella richiesta è aperta al suo interno. Annullando la finestra, o con un clone che fallisce, non viene collegato nulla.
- Le richieste di Azure DevOps e Bitbucket non hanno una revisione nell'app: **Rivedi** apre la pagina della richiesta nel
  browser e lo dice sulla riga di stato.

Le righe della palette *Radar: Rivedi tramite l'API* e *Radar: Clona e rivedi...* saltano la domanda.

Due caselle accanto al filtro (compaiono quando viene chiesto un elenco a livello di account), conservate in
`radar.toml`:

| Casella | Tasto | Predefinito | Effetto |
|---|---|---|---|
| `[x] Non clonati` | `a` | attiva | Elenca le righe dei repository senza un clone noto. Disattivata, restano solo le righe dei cloni noti. |
| `[ ] Bozze` | `d` | disattiva | Elenca le richieste in bozza delle righe a livello di account. Le bozze dei repository dell'ambito sono sempre elencate. |

Entrambe hanno i badge di `Ctrl+N` e sono nella palette (`Radar: Mostra le richieste non clonate`, `Radar: Mostra le bozze`).

La risposta di un profilo viene conservata in `radar-account.json` per **5 minuti**: riaprire il Radar entro quel
tempo mostra le righe senza interrogare l'host. **Aggiorna** (`r` o `F5`) interroga sempre: ogni repository e ogni elenco a livello di account, qualunque sia l'età dell'ultima risposta. Un host che ha chiesto di rallentare
o ha rifiutato il token non viene interrogato di nuovo nello stesso giro. Al massimo 300 righe sono conservate per profilo e tipo,
e una riga sotto **Repository** dice quante altre ne esistevano, o perché un profilo non si è potuto leggere
(**Riconnetti** apre Impostazioni > Integrazioni).

## Limiti

- Le righe a livello di account vengono dai profili GitHub, GitLab (compreso GitLab self-managed), Gitea, Forgejo,
  Azure DevOps, Bitbucket Data Center e Gerrit (non Bitbucket Cloud). Le issue assegnate a te vengono dai primi
  cinque; Bitbucket e Gerrit non hanno issue.
- Gli elenchi per repository usano i profili GitHub, GitLab (compreso GitLab self-managed), Gitea, Forgejo, Azure DevOps,
  Bitbucket e Gerrit; controlli, conflitti e menzioni vengono letti solo per i primi quattro.
- Le menzioni di GitLab vengono solo dalle cose da fare in sospeso; i controlli di
  GitHub, Gitea e Forgejo vengono chiesti per 15 richieste per repository.

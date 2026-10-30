[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · **Italiano** · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS ("Large File Storage") tiene i file grandi (file di design, video, dataset) fuori dalla cronologia di un repository: git memorizza al loro posto un *puntatore* di tre righe di testo, mentre il contenuto vero risiede sul server LFS e in una cache locale sotto `.git/lfs`.

GitPoulpi non richiede alcun account per tutto questo. LFS usa le credenziali che il tuo remoto usa già (il tuo credential helper di git, oppure l'ssh-agent).

## Cosa richiede il programma `git-lfs` e cosa no

GitPoulpi esegue il programma standard `git-lfs` per tutto ciò che converte o sposta file (install, track, fetch, pull, push, blocchi).

Leggere lo *stato* di LFS non richiede alcun programma: se un repository usa LFS, quali file sono puntatori, quanto sono grandi e se l'oggetto vero è già memorizzato in locale.

| | richiede `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (segnala il programma come mancante invece di fallire) | no |
| `gitpoulpi lfs files` | no |
| install / track / untrack / prune | sì |
| fetch / pull / push | sì |
| blocchi | sì |

## Installare `git-lfs`

- **Windows**: è incluso in Git for Windows; oppure `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Altrove: https://git-lfs.com

Serve la versione 3.0 o successiva. Dopo l'installazione GitPoulpi se ne accorge entro un minuto (o subito con `lfs status`).

## Quando `git-lfs` manca

GitPoulpi non va mai in crash né termina per questo motivo:

- `gitpoulpi lfs status` riesce e dice `git-lfs: not installed`, con il link di installazione.
- Un'operazione che richiede il programma fallisce con un messaggio chiaro che termina con il link di installazione (`https://git-lfs.com`) e con uno stato di uscita diverso da zero; via MCP lo strumento restituisce quel messaggio.
- Un repository che usa LFS si apre comunque; i suoi file grandi compaiono semplicemente come puntatori finché il programma non viene installato.

## Guardare i file LFS

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # pointer files in the index, with size and local/not downloaded
gitpoulpi lfs files --at v1.0   # ... as of a revision
gitpoulpi show big.bin --at HEAD  # reports "Git LFS object, 12.3 MB, sha256:..." instead of pointer text
```

## Configurare un repository: install, track, untrack

```
gitpoulpi lfs install                 # filters + hooks for THIS repository (git lfs install --local)
gitpoulpi lfs install --user          # into your global git config instead (never done silently)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... and make them read-only until locked
gitpoulpi lfs track '*.bin' --renormalize  # ... and re-stage already-committed *.bin as pointers
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # delete old, already-pushed objects from the local store
```

- `install` scrive per impostazione predefinita nel `.git/config` e negli hook del repository stesso, quindi la tua configurazione git globale non viene mai toccata a meno che tu non passi `--user`.
- `track` e `untrack` modificano `.gitattributes` e lo lasciano **non in stage**: compare come una normale modifica della directory di lavoro di cui fai tu il commit. I file aggiunti *dopo* il tracciamento vengono memorizzati come puntatori dal commit successivo.
- I file di cui è **già stato fatto il commit** come contenuto normale restano normali nella cronologia. `--renormalize` li rimette in stage in modo che il prossimo commit li sostituisca con dei puntatori. Per spostarli in LFS anche in ogni commit passato, usa la regola di riscrittura della cronologia *Migra i file a LFS* (vedi [Riscrittura della cronologia](history-rewrite.md#spostare-file-dentro-o-fuori-da-git-lfs)); è propria di GitPoulpi, non è `git lfs migrate`.
- Metti il pattern tra virgolette, così la shell non lo espande.
- Via MCP sono normali mutazioni (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), soggette al controllo `[mcp]` di `config.toml`.

## Spostare gli oggetti: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # download objects into the local store; working tree untouched
gitpoulpi lfs pull  [--remote origin]           # fetch, then replace pointer files with the real content
gitpoulpi lfs push  [--remote origin] [--all]   # upload the current branch's objects (or every ref with --all)
```

- Il remoto è per impostazione predefinita quello predefinito del repository, come per `fetch` e `push`.
- Le credenziali sono quelle abituali del tuo remoto (credential helper, ssh-agent). GitPoulpi non chiede mai un account proprio.
- `pull` installa i filtri LFS per questo repository se mancano: senza di essi git-lfs scarica gli oggetti ma salta la sostituzione dei puntatori, e un checkout successivo rimetterebbe i puntatori.
- MCP tratta `lfs push` come `push`: uno strumento distruttivo (modifica il remoto).
- La finestra mostra l'avanzamento di un trasferimento (file e byte) nella barra di stato. La CLI attende il risultato.

### Fare il checkout del contenuto: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: write real content over the pointer files, from the local store
```

Non scarica mai nulla (quello è `lfs pull`); gli oggetti assenti dall'archivio locale restano puntatori. Come `pull`, installa prima i filtri LFS del repository se non ne è configurato nessuno. Come ogni operazione LFS, non è annullabile; vedi [Annulla e ripeti](undo-redo.md). Via MCP è `lfs_checkout` (una scrittura, non distruttiva).

### Il pulsante LFS della barra degli strumenti, e pull / push

La barra degli strumenti ha un pulsante **`LFS ▾`** dopo **Pop** (nascosto quando `git.lfs_enabled` è disattivato, e in un repository dove LFS non è inizializzato né usato dal suo `.gitattributes`: vedi [Impostazioni > LFS](settings.md#specifico-del-repository-lfs)). Un clic apre un menu a discesa: **Fetch di tutti i file LFS** (`lfs fetch --all`), **Checkout di tutti i file LFS** (`lfs checkout`), **Pull di tutti i file LFS** (`lfs pull`), **Push di tutti i file LFS** (`lfs push --all`, gli oggetti di ogni riferimento) e **Pulisci l'archivio LFS locale** (`lfs prune`). Ogni riga è anche un comando della palette (`LFS: Fetch di tutti gli oggetti`, `LFS: Checkout di tutti i file`, `LFS: Pull degli oggetti`, `LFS: Push di tutti gli oggetti`, `LFS: Ripulisci gli oggetti locali`). `LFS: Apri il menu` apre il menu a discesa. Ognuno ha una sequenza `Ctrl+G`: `l` per il menu, poi `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` e `Shift+V` per le righe. Puoi cambiarle in *Impostazioni > Tastiera e mouse*. Se `git-lfs` manca, ogni riga porta alla finestra di installazione.

![Il menu a discesa LFS della barra degli strumenti: fetch, checkout, pull e push di tutti i file LFS, e pulizia dell'archivio locale](../images/lfs-menu.png)

**Pull e push gestiscono LFS da soli.** In un repository che usa LFS, con `git-lfs` installato, un semplice `pull` controlla prima che i filtri LFS del repository siano installati, e un semplice `push` controlla il suo hook `pre-push`. Quando ne manca uno, GitPoulpi esegue `git lfs install --local` (solo questo repository) prima del trasferimento, e la barra di stato dice "filtri LFS installati". Senza l'hook, un push non caricherebbe alcun oggetto LFS. Si disattiva con `git.lfs_autofix_filters = false` (*Impostazioni > Git e codifica > Installa i filtri LFS prima di pull e push*). Un semplice `push` carica comunque solo gli oggetti del branch inviato, mai quelli di ogni riferimento; il prune non è mai implicito.

## Blocchi dei file

Alcuni file non si possono unire (un file Photoshop, un livello, una risorsa binaria). I *blocchi* di Git LFS permettono a una persona di dire "ci sto lavorando io": nessun altro può bloccarlo né inviarne modifiche finché non viene sbloccato.

```
gitpoulpi lfs locks  [--remote origin]              # who holds which file (yours are marked)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Il blocco è una funzione del **server LFS** (GitHub, GitLab e Gitea ce l'hanno). Vi si accede con le credenziali del remoto; GitPoulpi non ha un account proprio.
- Un server senza blocchi fa fallire `lock` con un messaggio che lo dice (il tipo `LfsLockingUnsupported`). `locks` su un server simile non elenca nulla, ed è ciò che segnala lo stesso git-lfs.
- Tracciare un pattern con `--lockable` (`lfs track '*.psd' --lockable`) rende quei file in sola lettura finché non li blocchi.
- `unlock` del blocco di un altro viene rifiutato; `--force` lo rompe, ed è per questo che MCP tratta `lfs_unlock` come uno strumento distruttivo. `lfs_locks` è in sola lettura.

## Clonare un repository LFS

`gitpoulpi clone <url>` (e la finestra Clona della TUI) fa il checkout del repository *senza* eseguire i filtri LFS, poi, quando il repository usa LFS e `git-lfs` è installato, scarica gli oggetti con `git lfs pull` (installando i filtri per quel repository se non ne è configurato nessuno) e lo mostra come fase a sé, *Download degli oggetti LFS*. `--no-lfs` salta quel passaggio e lascia i puntatori.

Un clone **non fallisce mai a causa di LFS**: con `git-lfs` mancante, o con un download che fallisce, il clone riesce comunque e i file grandi restano puntatori; lo stato LFS del repository ne spiega il motivo, e `gitpoulpi lfs pull` completa il lavoro una volta risolto il problema.

## Nell'interfaccia da terminale

- **Il tag `LFS`.** Un file memorizzato con Git LFS ha un piccolo tag `LFS` dopo il nome negli elenchi File non in stage e File in stage del pannello Stato e nell'elenco dei file modificati di un commit.
- **La scheda del diff.** Aprire il diff di un file LFS mostra una scheda invece del testo del puntatore: `Oggetto Git LFS`, `1.2 MB -> 1.4 MB`, gli oid brevi vecchio e nuovo, e se il nuovo oggetto è memorizzato in locale oppure `non scaricato`. Premi `r` per il diff grezzo del puntatore, di nuovo `r` per tornare indietro. A un file LFS si applica solo lo stage a livello di file (non ci sono hunk né righe da mettere in stage).
- **La Vista file** di un file LFS dice `oggetto Git LFS, 12.3 MB, sha256:ab12cd34...` e dove ottenere il contenuto.
- **Il banner.** Per un repository che usa LFS mentre `git-lfs` manca o è più vecchio della 3.0, un banner nel pannello Stato lo segnala. Fai clic su di esso (o usa la voce della palette *LFS: Apri la pagina di installazione*) per aprire https://git-lfs.com. Nient'altro cambia: la finestra si apre, i diff funzionano, i file grandi compaiono semplicemente come puntatori.
- **Il menu del file e la palette.** Clic destro su un file modificato nel pannello Stato: *Traccia con LFS: questo file* / *questa estensione* per un file normale; *Blocca il file (LFS)*, *Sblocca il file (LFS)* o *Sblocca con la forza (LFS, blocco di un altro)* per un file LFS, a seconda di chi detiene il blocco. La riga di un file bloccato dice `bloccato da te` / `bloccato da <name>`. Tutto è anche un comando della palette (`Ctrl+P`, digita `LFS`): inizializzare, tracciare o smettere di tracciare un pattern, fetch, pull o push degli oggetti, prune, mostrare i blocchi, aprire la pagina di installazione.
- **Un'azione fallita si spiega da sola.** Se un'azione LFS fallisce perché `git-lfs` manca, una finestra riporta il messaggio e un pulsante *Apri pagina di installazione*.
- **Impostazioni.** *Impostazioni > LFS* (per repository) mostra se LFS è inizializzato, elenca i pattern tracciati e ha **Configura LFS qui**. *Git e codifica* (per repository) ha l'interruttore **Git LFS**, una riga in sola lettura **Programma git-lfs** e un pulsante **Inizializza LFS per questo repository**.
- **Git LFS** disattivato (`git.lfs_enabled = false`, vedi il [riferimento della configurazione](config-reference.md)) nasconde tutto questo. I filtri LFS di git continuano comunque a funzionare.
- Lo stato LFS viene letto poco dopo l'apertura del repository, dopo ogni modifica, a ogni aggiornamento manuale, quando `.gitattributes` cambia e ogni dieci secondi per un repository che usa LFS.

## Limiti noti

- Un repository i cui pattern LFS risiedono solo in un `.gitattributes` *annidato* (non quello nella radice, né `.git/info/attributes`) viene rilevato come LFS solo quando ha un archivio di oggetti LFS (`.git/lfs`), non prima.
- `git lfs migrate` in sé non viene usato. Spostare file dentro o fuori da LFS lungo la cronologia spetta alle regole *Migra i file a LFS* / *Riporta i file fuori da LFS* della riscrittura della cronologia: lavorano sul clone locale, viene scritto solo il `.gitattributes` nella radice, e gli oggetti vanno poi comunque inviati (*Invia gli oggetti LFS* nel pannello del risultato, oppure `lfs push --all`).

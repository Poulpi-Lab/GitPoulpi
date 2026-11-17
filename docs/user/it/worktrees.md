[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · **Italiano** · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Un worktree è una seconda copia in checkout del repository: un'altra cartella,
su un altro branch, che condivide la stessa cronologia. La sezione WORKTREES del
pannello Refs li elenca; quello principale è segnato con `(main)`.

![Il pannello Refs di un repository con due worktree collegati: la sezione WORKTREES elenca main (main), hotfix/axis-labels e feature/legend, con la sezione SOTTOMODULI sotto](../images/worktrees.png)

## Mostrare la sezione

La sezione è nascosta in un repository che non ha ancora alcun worktree
collegato. Per vederla comunque (allora dice «Ancora nessun worktree: + per
crearne uno»), usa il comando della palette **Barra laterale: Mostra la sezione
Worktrees**, spunta **WORKTREES** nell'elenco mostra/nascondi della barra
laterale (clic destro sull'intestazione di una sezione), oppure esegui uno
qualsiasi dei comandi di creazione qui sotto. GitPoulpi se lo ricorda per quel
repository (vedi **Sezioni della barra laterale** in
[Impostazioni](settings.md)); un altro repository senza worktree parte comunque
con la sezione nascosta.

## Creare un worktree

Il **+** sull'intestazione WORKTREES (tastiera: `+` nella sezione; palette:
**Refs: Crea un worktree...**) apre un menu:

- **Dal branch corrente**: il branch è in checkout nella cartella principale,
  quindi il dialogo propone un nuovo branch su di esso (`worktree-<id>`).
- **Dal commit selezionato**: disattivato finché nel Log non è selezionato un
  commit. Il dialogo propone un nuovo branch `worktree-<id>` che parte da lì;
  svuota il campo del branch per un worktree con HEAD scollegato.
- **Scegli un branch...**: il dialogo si apre sul suo elenco **Branch del
  worktree**, per aprire un branch che esiste già.

La palette ha le stesse tre voci (**Refs: Crea un worktree dal branch
corrente**, **Git: Crea un worktree sul commit selezionato...**, **Refs: Crea
un worktree scegliendo un branch**). Le voci precedenti funzionano ancora:
**Crea un worktree da...** nel menu di un branch, di un tag o di un commit.

![Il dialogo Crea worktree: i campi Nome, Cartella, Ref e Branch del worktree, la riga "nuovo branch da main", la casella Apri il nuovo worktree come scheda e i pulsanti Annulla e Crea](../images/worktree-create.png)

Il dialogo propone tutto e ti lascia cambiarlo. Il **Nome** e il nuovo branch
partono entrambi come `worktree-<id>`, dove `<id>` sono 8 cifre esadecimali
casuali (le stesse per entrambi), così due worktree non collidono mai.

| Campo | Significato |
|---|---|
| **Nome** | La cartella prende il suo nome. **Nome dal branch** (`Alt+N`) lo imposta a `worktree-<branch>`: le barre diventano `-` e la parola non viene ripetuta (`worktree-foo` e `feature/worktree-x` restano come sono, `feature/login` dà `worktree-feature-login`). |
| **Cartella** | Una cartella accanto al repository, `<repository>-<nome>`. Segue il Nome finché non scrivi nel campo Cartella; **Sfoglia...** sceglie dove metterla. |
| **Ref** | Da dove parte un *nuovo* branch: un branch o un commit (un elenco; scrivi per cercare). Non viene usato, e lo dice, mentre è scelto un branch esistente. |
| **Branch del worktree** | Un solo campo, due significati, spiegati sotto di esso. Un nome che non è un branch locale crea un nuovo branch dal Ref (**nuovo branch da main**). Un branch locale non in checkout altrove (sceglilo dall'elenco o scrivi il suo nome) viene messo in checkout lì così com'è (**fa il checkout di questo branch**). La prima riga dell'elenco, o un campo vuoto, fa il checkout del branch del Ref (o scollega HEAD su un commit); viene rifiutato quando quel branch è già in checkout. |
| **Rigenera** | `Alt+R`. Sorteggia un nuovo `<id>` e sostituisce il Nome e il branch con esso, tranne ciò che hai scritto tu (un Nome o un branch scritto da te resta) e un branch esistente (mai sostituito). Un Nome da **Nome dal branch** o dall'IA conta come generato. |
| **Nome con l'IA** | Solo con l'IA attiva (vedi sotto). |
| **Apri il nuovo worktree come scheda** | Attivo per impostazione predefinita. |

Una cartella non vuota, un nome non valido, un nome di branch che git rifiuta o
un branch che ha un altro worktree vengono segnalati sulla riga del dialogo
stesso, e **Crea** resta disattivato.

### Dare un nome con l'IA

Con l'[IA](ai.md) attiva, **Nome con l'IA** (`Alt+A`) apre una casella dentro il
dialogo: *Descrivi la funzionalità, il bug o l'attività*. **Dai un nome** chiede
al modello un nome breve per il worktree e un nome di branch che segue i nomi
dei branch del repository (`feature/`, `fix/`, kebab-case minuscolo). Riempiono
il Nome (come `worktree-<nome>`) e il campo del branch, ancora modificabili; un
nome di branch che esiste già riceve il suffisso `-2`, così la risposta non fa
mai un checkout in silenzio. La casella mostra lo stato di attesa, **Ferma** (o
`Esc`) annulla e un errore compare dentro il dialogo. Vengono inviati solo la
tua descrizione e i nomi dei branch locali.

## Il menu della riga

Clic destro su un worktree (oppure selezionalo e premi il tasto menu,
`Shift+F10` o `.`):

- **Apri in una nuova scheda**: come un doppio clic o `Enter`.
- **Apri in**: le voci di [Apri in](open-in.md) la cui casella **Può aprire un
  worktree** è spuntata (Impostazioni > Apri in; ogni voce è spuntata per
  impostazione predefinita). Vengono eseguite con la cartella del worktree come
  `${cwd}`.
- **Merge di <branch> in...**: scegli il branch in cui fare il merge (per
  impostazione predefinita il branch del worktree principale). Il merge
  ordinario viene eseguito nel worktree in cui quel branch è in checkout, con
  le tue opzioni di merge; un conflitto si risolve lì con il solito flusso dei
  conflitti. Un branch che non è in checkout da nessuna parte viene rifiutato,
  con il motivo. Non offerto per un worktree con HEAD scollegato.
- **Rimuovi worktree...**: mai quello principale.

![Il menu del clic destro di una riga di worktree: Apri in una nuova scheda, Apri in, Merge di hotfix/axis-labels in... e Rimuovi worktree...](../images/worktree-menu.png)

**Apri in una nuova scheda** (o un doppio clic, o `Enter`) apre la cartella del worktree come
scheda propria, con il suo Log, il suo stato e il suo branch:

![Il worktree hotfix/axis-labels aperto come scheda propria accanto a quella principale, con il suo commit e il suo file modificato](../images/worktree-tab.png)

La palette ha le stesse azioni sulla riga selezionata (**Refs: Apri il
worktree in...**, **Refs: Merge del branch del worktree in...**, **Refs:
Rimuovi il worktree...**, **Refs: Apri il worktree in una scheda**).

## Rimuovere un worktree

Un worktree pulito riceve una semplice conferma. Se il suo branch è già
integrato con un merge (nel suo upstream, nel branch del worktree principale o
in `main` / `master`) la casella **Elimina anche il branch <b> (già integrato in
<into> tramite merge)** è spuntata per impostazione predefinita; un branch non
integrato dice che resta. **Merge in... poi rimuovi** fa prima il merge e
rimuove solo quando il merge è andato a buon fine senza conflitti.

Con modifiche non salvate (tracciate o non tracciate) il dialogo le elenca
(numero, scorrevole) e offre:

- **Commit e rimuovi**: un campo per il messaggio nel dialogo; ogni modifica
  viene messa in stage (rispettando `.gitignore`) e salvata in un commit sul
  branch del worktree, poi il worktree viene rimosso. Un worktree con HEAD
  scollegato dà un nome a un branch per il commit nello stesso dialogo.
- **Stash e rimuovi**: ogni modifica, file non tracciati inclusi, viene messa
  in stash con un messaggio modificabile (predefinito «WIP su <cartella>»),
  poi il worktree viene rimosso. L'elenco degli stash è condiviso da tutti i
  worktree: dopo, lo stash è in STASH del repository principale, pronto per il
  pop.
- **Apri per il commit**: apre il worktree in una scheda con il messaggio di
  commit in primo piano; nulla viene rimosso.
- **Rimuovi comunque**: scarta le modifiche dopo una seconda conferma
  esplicita.

## Annullare

Rimuovere un worktree non si può annullare: la cartella non c'è più. Ciò che è
stato salvato prima resta: il commit è sul suo branch, lo stash nell'elenco.
L'eliminazione del branch che aggiunge **Elimina anche il branch** è quella
ordinaria registrata nel journal: **Annulla** riporta indietro il branch (non la
cartella). Anche creare un worktree non fa parte della cronologia.

## Da uno script o da un agente

`gitpoulpi worktree changes <path>` elenca le modifiche non salvate di un
worktree. `gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` lo rimuove; gli
strumenti MCP sono `worktree_changes` e `worktree_remove`. `gitpoulpi ai
worktree-name --describe "<task>"` stampa i nomi di worktree e branch che l'IA
propone (`{ name, worktree, branch }`) e non crea nulla; lo strumento MCP è
`ai_worktree_name`. Vedi il [riferimento della CLI](cli-reference.md).

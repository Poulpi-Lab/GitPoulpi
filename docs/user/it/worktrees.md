[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · **Italiano** · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Un worktree è una seconda copia in checkout del repository: un'altra cartella,
su un altro branch, che condivide la stessa cronologia. La sezione WORKTREES del
pannello Refs li elenca; quello principale è segnato con `(main)`.

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
  quindi il dialogo propone un nuovo branch su di esso (`<branch>-wt`).
- **Dal commit selezionato**: disattivato finché nel Log non è selezionato un
  commit. Il worktree parte con HEAD scollegato lì, a meno che tu non dia un
  nome a un nuovo branch.
- **Scegli un branch...**: il dialogo si apre sul suo elenco **Ref**.

La palette ha le stesse tre voci (**Refs: Crea un worktree dal branch
corrente**, **Git: Crea un worktree sul commit selezionato...**, **Refs: Crea
un worktree scegliendo un branch**). Le voci precedenti funzionano ancora:
**Crea un worktree da...** nel menu di un branch, di un tag o di un commit.

Il dialogo propone tutto e ti lascia cambiarlo:

| Campo | Significato |
|---|---|
| **Nome** | La cartella prende il suo nome. Proposto dal branch o dall'id breve. |
| **Cartella** | Una cartella accanto al repository, `<repository>-<nome>`. Segue il Nome finché non scrivi nel campo Cartella; **Sfoglia...** sceglie dove metterla. |
| **Ref** | Il branch o il commit da cui parte il worktree (un elenco; scrivi per cercare). Cambiarlo propone un nuovo Nome. |
| **Nuovo branch** | Se è vuoto, fa il checkout lì del branch del Ref (o scollega HEAD su un commit). Obbligatorio quando il branch è già in checkout. |
| **Apri il nuovo worktree come scheda** | Attivo per impostazione predefinita. |

Una cartella non vuota, un nome non valido o un branch che esiste già vengono
segnalati sulla riga del dialogo stesso, e **Crea** resta disattivato.

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
strumenti MCP sono `worktree_changes` e `worktree_remove`. Vedi il
[riferimento della CLI](cli-reference.md).

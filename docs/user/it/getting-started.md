[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · **Italiano** · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Per iniziare

Questa pagina ti porta da un'installazione appena fatta al tuo primo commit. Il
[tour dell'interfaccia](interface-tour.md) dà un nome a ogni area della finestra.
Le [Impostazioni](settings.md) elencano ogni opzione.

## 1. Installa e avvia

Segui l'[Installazione](installation.md). Poi apri un terminale in un
repository Git ed esegui:

```
gitpoulpi
```

GitPoulpi apre il repository che
contiene la cartella corrente. Fuori da un repository apre la scheda **Home**,
dove puoi sfogliare, clonare o inizializzare un repository (vedi il
[passo 7](#7-apri-clona-e-inizializza-i-repository)).

Le icone richiedono un Nerd Font o un font con le emoji. Se vedi dei riquadri,
consulta la [Risoluzione dei problemi](troubleshooting.md).

![La finestra principale: la barra laterale Refs a sinistra, il grafo del Log al centro e le modifiche in corso con il modulo di commit a destra](../images/hero.png)

## 2. Tre tasti per imparare tutto il resto

| Tasto | Apre |
|---|---|
| `?` | L'**aiuto**: ogni tasto e gesto del mouse. Scrivi per filtrarlo. `Ctrl+D` apre questa documentazione nel browser. |
| `Ctrl+P` | La **palette dei comandi**: scrivi per trovare un comando, `Enter` lo esegue. Ogni riga mostra la sua scorciatoia. |
| `Ctrl+,` | Le **Impostazioni**. Si aprono anche con l'ingranaggio nella barra del titolo, a sinistra della ×. |

Puoi fare tutto con la tastiera o con il mouse. La palette raggiunge ogni
comando. Vedi [Scorciatoie da tastiera](keybindings.md) e
[Mouse e gesti](mouse-and-gestures.md).

![La palette dei comandi filtrata sulla parola stage, con i comandi di stage e le loro scorciatoie](../images/palette.png)

## 3. Guarda la cronologia

La parte centrale della finestra è il **Log**: una riga per commit, dal più
recente. Mostra branch e tag come etichette colorate, il grafo, il messaggio,
l'autore, la data e l'hash breve.

- Clicca un commit. I suoi dettagli e i file modificati compaiono a destra.
- Clicca un file modificato. Il suo diff si apre al centro. `Esc` lo chiude.
- `j` / `k` o le frecce spostano la selezione. `f` cerca nei commit.
- Clic destro su un commit per il suo menu.

![Il Log con un commit di merge selezionato: il suo messaggio, l'autore e i due file che aggiunge, a destra](../images/log-graph.png)

## 4. Fai una modifica e mettila in stage

Modifica un file nel tuo editor. La **riga in cima al Log** ora mostra le
modifiche in corso: una riga tratteggiata con il numero di file modificati.
Cliccala. Il pannello destro mostra **File non in stage** e **File in
stage**.

- Passa il mouse su un file e clicca il suo pulsante **Stage del file**. Oppure
  clicca **Stage di tutto**.
- Clicca il nome di un file per aprirne il diff. Lì puoi mettere in stage un
  **hunk** (**Stage dell'hunk**) o una **riga**.

Vedi [Stage e commit](staging-and-committing.md).

![Il diff di un file modificato, con i pulsanti Stage dell'hunk e Scarta l'hunk sopra ogni hunk](../images/diff.png)

## 5. Fai il commit

Scrivi un riepilogo, e se vuoi una descrizione, nel modulo di commit in basso a
destra. Poi clicca il pulsante verde **Commit delle modifiche a N file** o premi
`Ctrl+Enter` (oppure `Ctrl+G Enter`, da qualsiasi punto). Spunta **Modifica il commit precedente** per aggiungere le tue
modifiche all'ultimo commit invece di crearne uno nuovo.

Se hai attivato l'[IA](ai.md), il pulsante `✦` accanto al riepilogo scrive un
messaggio a partire dalle modifiche in stage. GitPoulpi non fa mai il commit
finché non clicchi il pulsante di commit.

## 6. Condividi il tuo lavoro

La barra degli strumenti ha i pulsanti **Pull** e **Push**, ciascuno con un
elenco `▾`. Da tastiera premi `Ctrl+G` e poi una lettera: `Ctrl+G p` fa il pull,
`Ctrl+G Shift+P` il push. Dopo `Ctrl+G`, la barra di stato elenca le lettere.

GitPoulpi usa l'autenticazione che il tuo `git` usa già (credential helper,
ssh-agent). Non ha bisogno di un account suo. Per vedere e creare le pull
request, collega una volta un account di hosting. Vedi
[Integrazioni di hosting e issue tracker](hosting-integration.md).

## 7. Apri, clona e inizializza i repository

Clicca l'icona della casa all'estremità sinistra della riga delle schede, o
premi `Ctrl+O`. La scheda **Home** elenca i tuoi repository aperti, preferiti e
recenti, con i pulsanti **Sfoglia**, **Clona** e **Init**. Ogni repository si
apre in una scheda propria. `Ctrl+W` chiude la scheda corrente.

![La scheda Home con l'elenco dei repository aperti e recenti e i pulsanti Sfoglia, Clona e Init](../images/home.png)

**Clona** chiede un URL e una cartella, oppure ti lascia scegliere un repository da un account connesso (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; vedi [Clonare da un account connesso](hosting-integration.md#clonare-da-un-account-connesso)). **Init** trasforma una cartella in un
nuovo repository.
In tutte e tre le finestre (e nel selettore di cartelle che aprono), `Ctrl+N` mostra un badge su ogni campo e pulsante: premi il carattere per portare il cursore nel campo o premere il pulsante.

| ![La finestra Clona: dove clonare, l'URL, il nome della cartella e i pulsanti Annulla e Clona](../images/clone.png) | ![La finestra Inizializza un repository: la cartella, il nome e il branch iniziale](../images/init.png) |
|---|---|

### Checkout parziale

Un repository grande non deve finire per intero sul disco. Nella finestra Clona
spunta **Checkout parziale (solo alcune cartelle)** e scrivi le cartelle che
vuoi, separate da spazi o virgole (`src docs/guide`). Viene fatto il checkout
solo di quelle cartelle e dei file nella radice del repository; la cronologia
completa viene comunque scaricata, quindi Log, blame e branch funzionano come
sempre. Finché la casella è spuntata serve almeno una cartella.

![La finestra Checkout parziale con due cartelle scritte e i pulsanti Annulla, Disattiva e Applica](../images/sparse-checkout.png)

Qualsiasi repository può essere cambiato in seguito: apri la palette dei comandi
(`Ctrl+P`) ed esegui **Repository: Checkout parziale...**, oppure usa
**Checkout parziale...** nel menu della scheda del repository. La finestra
elenca in un solo campo le cartelle di cui è stato fatto il checkout: aggiungi o
togli nomi e premi **Applica** (`Enter`), oppure **Disattiva (checkout
completo)** per riportare ogni file. Finché la directory di lavoro è parziale, la barra di stato
mostra un piccolo badge `parziale`. Gli stessi controlli sono una pagina di
*Impostazioni > Checkout parziale* (un elenco di percorsi, uno per riga, con i pulsanti **Avvia**,
**Interrompi** e **Aggiorna**).

Note:

- È la modalità cone di git (cartelle, non pattern); `*`, `..` e i percorsi
  assoluti vengono rifiutati.
- Annulla non registra la modifica dell'insieme: non si perde nulla e
  **Disattiva** ripristina ogni file. Git può rifiutarsi di togliere una cartella
  che contiene modifiche non committate (l'errore compare nella barra di stato);
  fai prima commit o stash.
- La stessa cosa da script e agenti: `gitpoulpi clone <url> --sparse src docs` e
  `gitpoulpi sparse-checkout status | set | add | disable` (vedi il
  [riferimento della CLI](cli-reference.md)); il server MCP ha gli stessi
  strumenti.

## Per proseguire

- [Tour dell'interfaccia](interface-tour.md): il nome di tutto ciò che vedi.
- [Lavorare con il Log](log-and-history.md) e
  [Stage e commit](staging-and-committing.md): il lavoro di ogni giorno.
- [Temi](themes.md) e [Impostazioni](settings.md).
- [Assistenza IA](ai.md) e [Integrazioni di hosting e issue tracker](hosting-integration.md):
  le due integrazioni facoltative.
- [Riferimento della CLI](cli-reference.md) e [MCP e agenti](mcp-and-agents.md):
  le stesse operazioni da script e agenti.
- [Risoluzione dei problemi](troubleshooting.md).
